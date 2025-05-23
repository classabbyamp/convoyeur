// host.rs: part of convoyeur
//
// Copyright (c) 2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use std::{collections::HashMap, error::Error, fmt, str::FromStr};

use actix_web::web::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart::{Form as MpForm, Part},
    Client,
};
use serde::{Deserialize, Serialize};

use crate::site::Site;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Host {
    Form(Form),
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Form(v) => write!(f, "Host(Form({}))", v),
        }
    }
}

impl Site for Host {
    async fn upload<F: Into<String>, M: Into<String>>(
        &self,
        client: &Client,
        file: Bytes,
        file_name: F,
        mime: M,
    ) -> Result<String, Box<dyn Error>> {
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

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
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
    ) -> Result<String, Box<dyn Error>> {
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
            header_map.insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&val)?);
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
