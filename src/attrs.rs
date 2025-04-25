// attrs.rs: part of convoyeur
//
// Copyright (c) 2025 classabbyamp
// SPDX-License-Identifier: LiLiQ-P-1.1

use std::fmt;

static DEFAULT_MIME: &str = "application/octet-stream";

pub struct FileAttrs {
    pub name: String,
    pub mime: String,
}

impl FileAttrs {
    pub fn from(name: Option<impl AsRef<str>>, mime: Option<impl AsRef<str>>) -> Self {
        Self {
            name: name.map_or("file".to_string(), |x| x.as_ref().to_string()),
            mime: mime.map_or(DEFAULT_MIME.to_string(), |x| x.as_ref().to_string()),
        }
    }
}

impl Default for FileAttrs {
    fn default() -> Self {
        Self {
            name: "file".to_string(),
            mime: DEFAULT_MIME.to_string(),
        }
    }
}

impl fmt::Display for FileAttrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File(name: {}, mime: {})", self.name, self.mime)
    }
}
