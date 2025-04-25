// Convoyeur
//
// Copyright (c) 2024-2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use std::env;

use actix_web::middleware::from_fn;
use actix_web::HttpMessage;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::{anyhow, Context};
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

async fn index() -> impl Responder {
    HttpResponse::Ok().body(USER_AGENT)
}

async fn upload(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<reqwest::Client>,
) -> Result<impl Responder, AppError> {
    if let Some(host) = req.extensions().get::<Host>() {
        let exts = req.extensions();
        let attrs = exts.get::<FileAttrs>().unwrap();

        info!("uploading file {} to host {}", attrs, host);
        let url = host
            .upload(client.get_ref(), body, &attrs.name, &attrs.mime)
            .await
            .context("failed to upload to host")?;

        Ok(HttpResponse::Created()
            .insert_header(("Location", url.trim()))
            .finish())
    } else {
        Err(anyhow!("no upload host specified").into())
    }
}

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let conf = Config::from_env()?;
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
                .service(web::resource("/").route(web::get().to(index)))
                .service(
                    web::resource("/")
                        .route(web::post().to(upload))
                        .wrap(from_fn(strip_exif))
                        .wrap(from_fn(get_file_attrs))
                        .wrap(from_fn(check_headers)),
                )
                .default_service(web::to(|| HttpResponse::NotFound()))
        })
        .bind(conf.bind)?
        .run(),
    )
}
