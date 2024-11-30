use std::env;

use actix_web::{
    get, http::header::ContentDisposition, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder
};
use anyhow::{anyhow, Context};
use log::debug;
use models::{AppError, Config, Site};

mod models;

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

    if let Some(username) = req.headers().get("Soju-Username") {
        let username = match username.to_str() {
            Ok(u) => u,
            Err(e) => return Err(anyhow!("failed to get username: {}", e).into()),
        };

        let maybe_host_id = if conf.users.contains_key(username) {
            conf.users.get(username)
        } else {
            conf.default_host.as_ref()
        };

        if let Some(host_id) = maybe_host_id {
            if let Some(host) = conf.hosts.get(host_id) {
                debug!("uploading file with file_name={:?}, mime_type={:?}, size={:?} to host {:?} for user {:?}",
                       file_name, mime_type, file_size, host_id, username);
                let url = host
                    .upload(client.get_ref(), body, file_name, mime_type)
                    .await.context("failed to upload to host")?;

                return Ok(HttpResponse::Created()
                    .insert_header(("Location", url.trim()))
                    .finish());
            }
            return Err(anyhow!("host {:?} does not exist in configuration", maybe_host_id).into());
        }
        return Err(anyhow!("host not found for user {:?}", username).into());
    }
    Err(anyhow!("missing Soju-Username header").into())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let conf = models::Config::from_env()?;
    let conf_data = web::Data::new(conf.clone());
    let client = web::Data::new(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(anyhow::Error::from),
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&conf_data))
            .app_data(web::Data::clone(&client))
            .service(index)
            .service(upload)
    })
    .bind(conf.bind)?
    .run()
    .await
}
