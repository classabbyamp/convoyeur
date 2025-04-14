use std::{collections::HashMap, env, fs::File};

use log::info;
use serde::{Deserialize, Serialize};

use crate::host::Host;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind: String,
    pub default_host: Option<String>,
    pub upload_limit: Option<usize>,
    #[serde(rename = "host")]
    pub hosts: HashMap<String, Host>,
    pub users: HashMap<String, String>,
}

impl Config {
    pub fn from_env() -> std::io::Result<Self> {
        if let Some(conf_path) = env::var_os("CONVOYEUR_CONF") {
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
            upload_limit: None,
            hosts: HashMap::new(),
            users: HashMap::new(),
        }
    }
}
