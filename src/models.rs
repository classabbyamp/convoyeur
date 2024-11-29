use std::{collections::HashMap, str::FromStr};

use actix_web::web::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart::{Form as MpForm, Part},
    Client,
};
use serde::{Deserialize, Serialize};

pub trait Site {
    async fn upload<F: Into<String>, M: Into<String>>(
        &self,
        client: &Client,
        file: Bytes,
        file_name: F,
        mime: M,
    ) -> Result<String, reqwest::Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind: String,
    pub default_host: Option<String>,
    #[serde(rename = "host")]
    pub hosts: HashMap<String, Host>,
    pub users: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: "localhost:8069".into(),
            default_host: None,
            hosts: HashMap::new(),
            users: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Host {
    Form(Form),
}

impl Site for Host {
    async fn upload<F: Into<String>, M: Into<String>>(
        &self,
        client: &Client,
        file: Bytes,
        file_name: F,
        mime: M,
    ) -> Result<String, reqwest::Error> {
        match self {
            Self::Form(f) => f.upload(client, file, file_name, mime).await,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    pub url: String,
    pub file_field: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            url: "".into(),
            file_field: "file".into(),
            fields: HashMap::new(),
            headers: HashMap::new(),
        }
    }
}

impl Site for Form {
    async fn upload<F: Into<String>, M: Into<String>>(
        &self,
        client: &Client,
        file: Bytes,
        file_name: F,
        mime: M,
    ) -> Result<String, reqwest::Error> {
        let mut form = MpForm::new().part(
            self.file_field.clone(),
            Part::stream(file.clone())
                .file_name(file_name.into())
                .mime_str(&mime.into())?,
        );
        let fields = self.fields.clone();
        for (name, part) in fields.into_iter() {
            form = form.text(name, part);
        }
        let headers = self.headers.clone();
        let mut header_map = HeaderMap::new();
        for (key, val) in headers.into_iter() {
            header_map.insert(
                HeaderName::from_str(&key).unwrap(),
                HeaderValue::from_str(&val).unwrap(),
            );
        }

        dbg!(&form);
        let req = client.post(&self.url).headers(header_map).multipart(form);
        dbg!(&req);
        let resp = req.send().await?;
        resp.text().await
    }
}
