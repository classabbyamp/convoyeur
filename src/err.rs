// err.rs: part of convoyeur
//
// Copyright (c) 2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use std::{error::Error, fmt::Display};

use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};

#[derive(Debug)]
pub struct AppError {
    inner: Box<dyn Error>,
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self {
            inner: Box::<dyn Error>::from(value),
        }
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self {
            inner: Box::<dyn Error>::from(value),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
