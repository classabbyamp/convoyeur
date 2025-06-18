// Convoyeur
//
// Copyright (c) 2024-2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use std::env;

use actix_web::error::ErrorInternalServerError;
use actix_web::middleware::{from_fn, Logger};
use actix_web::web::Data;
use actix_web::Error;
use actix_web::HttpMessage;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::info;

use crate::attrs::FileAttrs;
use crate::config::Config;
use crate::err::AppError;
use crate::host::Host;
use crate::middleware::{check_headers, get_file_attrs, strip_exif};
use crate::site::Site;

mod attrs;
mod config;
mod err;
mod host;
mod middleware;
mod site;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

async fn index(req: HttpRequest) -> Result<impl Responder, Error> {
    let conf = match req.app_data::<Data<Config>>() {
        Some(c) => c,
        None => return Err(ErrorInternalServerError("could not load configuration")),
    };
    let body = format!(
        "{}\n\n{} users and {} hosts\ndefault host: {:?}\nstrip exif: {}\nupload limit: {:?} MiB\n",
        USER_AGENT,
        conf.users.len(),
        conf.hosts.len(),
        conf.default_host,
        conf.strip_exif,
        conf.upload_limit
    );
    Ok(HttpResponse::Ok().body(body))
}

async fn upload(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<reqwest::Client>,
) -> Result<impl Responder, AppError> {
    let (host, attrs) = {
        let exts = req.extensions();
        if let Some(h) = exts.get::<Host>() {
            let attrs = exts.get::<FileAttrs>().unwrap().clone();
            (h.clone(), attrs)
        } else {
            return Err("no upload host specified".into());
        }
    };

    info!("uploading file {} to host {}", attrs, host);
    let url = match host
        .upload(client.get_ref(), body, &attrs.name, &attrs.mime)
        .await
    {
        Ok(u) => u,
        Err(e) => return Err(format!("failed to upload to host: {}", e).into()),
    };

    Ok(HttpResponse::Created()
        .insert_header(("Location", url.trim()))
        .finish())
}

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info")).init();

    let conf = Config::from_env()?;
    info!(
        "CONF: {} users and {} hosts",
        conf.users.len(),
        conf.hosts.len()
    );
    info!("CONF: default host: {:?}", conf.default_host);
    info!("CONF: strip exif: {}", conf.strip_exif);
    info!("CONF: upload limit: {:?} MiB", conf.upload_limit);
    let conf_data = web::Data::new(conf.clone());
    let client = web::Data::new(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap(),
    );

    actix_web::rt::System::new().block_on(
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::clone(&conf_data))
                .app_data(web::Data::clone(&client))
                .app_data(web::PayloadConfig::new(
                    conf.upload_limit.unwrap_or(25) * 1024 * 1024,
                ))
                .wrap(Logger::default())
                .service(web::resource("/").route(web::get().to(index)))
                .service(
                    web::resource("/upload")
                        .route(web::post().to(upload))
                        .wrap(from_fn(strip_exif))
                        .wrap(from_fn(get_file_attrs))
                        .wrap(from_fn(check_headers)),
                )
                .default_service(web::to(HttpResponse::NotFound))
        })
        .bind(conf.bind)?
        .run(),
    )
}
