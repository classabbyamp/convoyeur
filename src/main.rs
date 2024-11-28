use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, http::header::ContentDisposition};

use crate::site::{FormUploadSite, Site};

mod site;

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);
static DEFAULT_MIME: &str = "application/octet-stream";

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("meow")
}

#[post("/")]
async fn upload(req: HttpRequest, body: web::Bytes) -> impl Responder {
    let disposition = ContentDisposition::from_raw(req.headers().get("Content-Disposition").unwrap()).unwrap();
    let file_name = disposition.get_filename().unwrap_or("file");
    let mime_type = req.headers().get("Content-Type").map_or(DEFAULT_MIME, |x| x.to_str().unwrap_or(DEFAULT_MIME));
    let username = req.headers().get("Soju-Username");
    dbg!(&file_name);
    dbg!(&mime_type);
    dbg!(&username);

    let site = FormUploadSite {
        url: "https://0x0.st".into(),
        .. Default::default()
    };

    let url = site.upload(body, file_name, mime_type).await.unwrap();
    dbg!(&url);

    HttpResponse::Created()
        .insert_header(("Location", url.trim()))
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| App::new().service(index).service(upload))
        .bind(("127.0.0.1", 8069))?
        .run()
        .await
}
