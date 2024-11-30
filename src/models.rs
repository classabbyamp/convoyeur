use std::{collections::HashMap, env, fs::File, str::FromStr};

use actix_web::{error::ResponseError, http::{header::ContentType, StatusCode}, web::Bytes, HttpResponse};
use derive_more::derive::{Display, Error};
use log::info;
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
    ) -> anyhow::Result<String>;
}

#[derive(Debug, Display, Error)]
#[display("{inner}")]
pub struct AppError {
    inner: anyhow::Error,
}

impl ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.inner {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind: String,
    pub default_host: Option<String>,
    #[serde(rename = "host")]
    pub hosts: HashMap<String, Host>,
    pub users: HashMap<String, String>,
}

impl Config {
    pub fn from_env() -> std::io::Result<Self> {
        if let Some(conf_path) = env::var_os("ADAPTER_CONF") {
            info!("loading configuration from {:?}", conf_path);
            let input = File::open(conf_path)?;
            // TODO: remove unwrap
            Ok(hcl::from_reader(input).unwrap())
        } else {
            Ok(Self::default())
        }
    }
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
    ) -> anyhow::Result<String> {
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
    ) -> anyhow::Result<String> {
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
                HeaderName::from_str(&key)?,
                HeaderValue::from_str(&val)?,
            );
        }

        Ok(client
            .post(&self.url)
            .headers(header_map)
            .multipart(form)
            .send()
            .await?
            .text()
            .await?)
    }
}
