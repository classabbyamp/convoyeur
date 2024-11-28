use std::collections::HashMap;

use actix_web::web::Bytes;
use reqwest::{header::HeaderMap, multipart::{Form, Part}, Client};

use crate::USER_AGENT;

pub trait Site {
    async fn upload<S: Into<String>>(&self, file: Bytes, file_name: S, mime: S) -> Result<String, reqwest::Error>;
}

pub struct FormUploadSite {
    pub client: Client,
    pub url: String,
    pub file_field: String,
    pub extra_fields: HashMap<String, String>,
    pub extra_headers: HeaderMap,
}

impl FormUploadSite {
    fn new_rustypaste(url: &str, authz: Option<&str>, expire: Option<&str>) -> Self {
        todo!()
    }

    fn new_0x0_st(url: &str, secret: bool, expire: Option<&str>) -> Self {
        todo!()
    }

    fn new_x0_at(url: &str, secret: bool, expire: Option<&str>) -> Self {
        todo!()
    }

    fn new_logpaste(url: &str) -> Self {
        todo!()
    }
}

impl Default for FormUploadSite {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder().user_agent(USER_AGENT).build().unwrap(),
            url: "".into(),
            file_field: "file".into(),
            extra_fields: HashMap::new(),
            extra_headers: HeaderMap::default(),
        }
    }
}

impl Site for FormUploadSite {
    async fn upload<S: Into<String>>(&self, file: Bytes, file_name: S, mime: S) -> Result<String, reqwest::Error> {
        let mut form = Form::new().part(
            self.file_field.clone(),
            Part::stream(file.clone())
                .file_name(file_name.into())
                .mime_str(&mime.into())?
        );
        let fields = self.extra_fields.clone();
        for (name, part) in fields.into_iter() {
            form = form.text(name, part);
        }
        dbg!(&form);
        let req = self.client.post(&self.url).headers(self.extra_headers.clone()).multipart(form);
        dbg!(&req);
        let resp = req.send().await?;
        resp.text().await
    }
}
