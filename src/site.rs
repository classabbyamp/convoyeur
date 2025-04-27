// site.rs: part of convoyeur
//
// Copyright (c) 2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use std::error::Error;

use actix_web::web::Bytes;
use reqwest::Client;

pub trait Site {
    async fn upload<F: Into<String>, M: Into<String>>(
        &self,
        client: &Client,
        file: Bytes,
        file_name: F,
        mime: M,
    ) -> Result<String, Box<dyn Error>>;
}
