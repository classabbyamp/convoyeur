use actix_web::web::Bytes;
use reqwest::Client;

pub trait Site {
    async fn upload<F: Into<String>, M: Into<String>>(
        &self,
        client: &Client,
        file: Bytes,
        file_name: F,
        mime: M,
    ) -> anyhow::Result<String>;
}
