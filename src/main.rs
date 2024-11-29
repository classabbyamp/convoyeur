use std::{env, fs::File};

use actix_web::{
    get, http::header::ContentDisposition, post, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use log::{debug, info};
use models::{Config, Site};

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
) -> impl Responder {
    let disposition =
        ContentDisposition::from_raw(req.headers().get("Content-Disposition").unwrap()).unwrap();
    let file_name = disposition.get_filename().unwrap_or("file");
    let mime_type = req
        .headers()
        .get("Content-Type")
        .map_or(DEFAULT_MIME, |x| x.to_str().unwrap_or(DEFAULT_MIME));
    debug!("file_name={:?}, mime_type={:?}", file_name, mime_type);

    if let Some(username) = req.headers().get("Soju-Username") {
        let username = username.to_str().unwrap();
        let maybe_host_id = if conf.users.contains_key(username) {
            conf.users.get(username)
        } else {
            conf.default_host.as_ref()
        };
        debug!("username={:?}, maybe_host_id={:?}", username, maybe_host_id);

        if let Some(host_id) = maybe_host_id {
            debug!("host_id={:?}", host_id);
            if let Some(host) = conf.hosts.get(host_id) {
                debug!("host={:?}", host);
                let url = host
                    .upload(client.get_ref(), body, file_name, mime_type)
                    .await
                    .unwrap();
                debug!("url={:?}", url);

                return HttpResponse::Created()
                    .insert_header(("Location", url.trim()))
                    .finish();
            }
            debug!("host not found");
        }
        debug!("host_id not found");
    }
    debug!("username not found");
    todo!()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let conf: models::Config = if let Some(conf_path) = env::var_os("ADAPTER_CONFIG") {
        info!("loading configuration from {:?}", conf_path);
        let input = File::open(conf_path)?;
        hcl::from_reader(input).unwrap()
    } else {
        models::Config::default()
    };
    dbg!(&conf);

    let conf_data = web::Data::new(conf.clone());
    let client = web::Data::new(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap(),
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
