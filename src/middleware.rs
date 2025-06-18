// middleware.rs: part of convoyeur
//
// Copyright (c) 2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorInternalServerError, ErrorNotFound},
    http::header::ContentDisposition,
    middleware::Next,
    web::{Bytes, Data},
    Error, HttpMessage,
};
use little_exif::{filetype::FileExtension, metadata::Metadata};
use log::debug;

use crate::{attrs::FileAttrs, config::Config};

pub async fn check_headers(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let headers = req.headers();
    let mut maybe_host_id = match headers.get("X-Upload-Host") {
        Some(h) => match h.to_str() {
            Ok(s) => {
                debug!("Found X-Upload-Host header: {}", s);
                Some(s.to_owned())
            }
            Err(e) => {
                debug!("Failed to decode host ID from X-Upload-Host header: {}", e);
                None
            }
        },
        None => {
            debug!("X-Upload-Host header not found");
            None
        }
    };

    let username = if maybe_host_id.is_none() {
        match headers.get("Soju-Username") {
            Some(u) => match u.to_str() {
                Ok(s) => {
                    debug!("Found Soju-Username header: {}", s);
                    Some(s)
                }
                Err(e) => {
                    debug!("Failed to decode username from Soju-Username header: {}", e);
                    None
                }
            },
            None => {
                debug!("Soju-Username header not found");
                match headers.get("X-Username") {
                    Some(u) => match u.to_str() {
                        Ok(s) => {
                            debug!("Found X-Username header: {}", s);
                            Some(s)
                        }
                        Err(e) => {
                            debug!("Failed to decode username from X-Username header: {}", e);
                            None
                        }
                    },
                    None => {
                        debug!("X-Username header not found");
                        None
                    }
                }
            }
        }
    } else {
        None
    };

    let host = {
        let conf = match req.app_data::<Data<Config>>() {
            Some(c) => c,
            None => return Err(ErrorInternalServerError("could not load configuration")),
        };

        if let Some(uname) = username {
            maybe_host_id = match conf.users.get(uname) {
                Some(s) => {
                    debug!("Found host {:?} for user {:?}", s, uname);
                    Some(s.to_owned())
                }
                None => {
                    debug!("no upload host found for user");
                    None
                }
            };
        };

        let host_id = match &maybe_host_id {
            Some(h) => h,
            None => {
                debug!("using default upload host");
                match &conf.default_host {
                    Some(h) => h,
                    None => return Err(ErrorNotFound("default upload host not defined")),
                }
            }
        };

        match conf.hosts.get(host_id) {
            Some(h) => h.clone(),
            None => {
                return Err(ErrorNotFound(format!(
                    "host {:?} does not exist in configuration",
                    host_id
                )))
            }
        }
    };

    req.extensions_mut().insert(host);

    next.call(req).await
}

pub async fn get_file_attrs(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let disp = match req
        .headers()
        .get("Content-Disposition")
        .map(ContentDisposition::from_raw)
    {
        Some(Ok(d)) => d,
        None | Some(Err(_)) => {
            return Err(ErrorInternalServerError(
                "missing or malformed Content-Disposition header",
            ))
        }
    };

    let file_name = disp.get_filename();
    let mime_type = req
        .headers()
        .get("Content-Type")
        .and_then(|x| x.to_str().ok());

    req.extensions_mut()
        .insert(FileAttrs::from(file_name, mime_type));

    next.call(req).await
}

pub async fn strip_exif(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let conf = match req.app_data::<Data<Config>>() {
        Some(c) => c,
        None => return Err(ErrorInternalServerError("could not load configuration")),
    };

    if conf.strip_exif {
        let extension = {
            let exts = req.extensions();
            let file_attrs = exts.get::<FileAttrs>().unwrap();

            match file_attrs.mime.as_str() {
                "image/png" => Some(FileExtension::PNG {
                    as_zTXt_chunk: true,
                }),
                "image/jpeg" => Some(FileExtension::JPEG),
                "image/jxl" => Some(FileExtension::JXL),
                "image/tiff" => Some(FileExtension::TIFF),
                "image/webp" => Some(FileExtension::WEBP),
                _ => None,
            }
        };

        if let Some(ext) = extension {
            debug!("clearing EXIF metadata");
            let body = req.extract::<Bytes>().await?;
            let mut buf = body.to_vec();

            Metadata::clear_metadata(&mut buf, ext)?;

            if ext == FileExtension::JPEG {
                Metadata::clear_app12_segment(&mut buf, ext)?;
                Metadata::clear_app13_segment(&mut buf, ext)?;
            }

            let body: Bytes = buf.into();
            req.set_payload(body.into());
        }
    }

    next.call(req).await
}
