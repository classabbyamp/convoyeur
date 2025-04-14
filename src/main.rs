use std::env;

use actix_web::{
    get, http::header::ContentDisposition, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder
};
use anyhow::{anyhow, Context};
use log::{debug, info};

use crate::config::Config;
use crate::err::AppError;
use crate::site::Site;

mod config;
mod err;
mod host;
mod site;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static DEFAULT_MIME: &str = "application/octet-stream";

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(USER_AGENT)
}

#[post("/")]
async fn upload(
    req: HttpRequest,
    body: web::Bytes,
    conf: web::Data<Config>,
    client: web::Data<reqwest::Client>,
) -> Result<impl Responder, AppError> {
    let mut maybe_host_id = match req.headers().get("X-Upload-Host") {
        Some(h) => match h.to_str() {
            Ok(s) => {
                debug!("Found X-Upload-Host header: {}", s);
                Some(s.to_owned())
            }
            Err(e) => {
                return Err(anyhow!("Failed to decode host ID from X-Upload-Host header: {}", e).into());
            }
        }
        None => {
            debug!("X-Upload-Host header not found");
            None
        }
    };

    let username = if maybe_host_id.is_none() {
        match req.headers().get("Soju-Username") {
            Some(u) => match u.to_str() {
                Ok(s) => {
                    debug!("Found Soju-Username header: {}", s);
                    Some(s)
                }
                Err(e) => {
                    return Err(anyhow!("Failed to decode username from Soju-Username header: {}", e).into());
                }
            }
            None => {
                debug!("Soju-Username header not found");
                match req.headers().get("X-Username") {
                    Some(u) => match u.to_str() {
                        Ok(s) => {
                            debug!("Found X-Username header: {}", s);
                            Some(s)
                        }
                        Err(e) => {
                            return Err(anyhow!("Failed to decode username from X-Username header: {}", e).into());
                        }
                    }
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

    if let Some(uname) = username {
        maybe_host_id = match conf.users.get(uname) {
            Some(s) => Some(s.to_owned()),
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
                None => return Err(anyhow!("default upload host not defined").into()),
            }
        }
    };

    let host = match conf.hosts.get(host_id) {
        Some(h) => h,
        None => return Err(anyhow!("host {:?} does not exist in configuration", maybe_host_id).into()),
    };

    let disp = match req.headers().get("Content-Disposition").map(ContentDisposition::from_raw) {
        Some(Ok(d)) => d,
        None | Some(Err(_)) => return Err(anyhow!("missing or malformed Content-Disposition header").into()),
    };
    let file_name = disp.get_filename().unwrap_or("file");
    let mime_type = req
        .headers()
        .get("Content-Type")
        .map_or(DEFAULT_MIME, |x| x.to_str().unwrap_or(DEFAULT_MIME));
    let file_size = body.len();

    info!("uploading file with file_name={:?}, mime_type={:?}, size={:?} to host {:?} for user {:?}",
           file_name, mime_type, file_size, host_id, username.unwrap_or("-"));
    let url = host
        .upload(client.get_ref(), body, file_name, mime_type)
        .await.context("failed to upload to host")?;

    return Ok(HttpResponse::Created()
        .insert_header(("Location", url.trim()))
        .finish());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let conf = Config::from_env()?;
    let conf_data = web::Data::new(conf.clone());
    let client = web::Data::new(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap()
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&conf_data))
            .app_data(web::Data::clone(&client))
            .app_data(web::PayloadConfig::new(conf.upload_limit.unwrap_or(25) * 1024 * 1024))
            .service(index)
            .service(upload)
            .default_service(web::to(|| HttpResponse::NotFound()))
    })
    .bind(conf.bind)?
    .run()
    .await
}
