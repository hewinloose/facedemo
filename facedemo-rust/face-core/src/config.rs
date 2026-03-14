use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub group_id: String,
    pub ws_url: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),
}

impl ConfigError {
    pub fn missing_key(&self) -> &'static str {
        match self {
            Self::MissingVar(key) => key,
        }
    }
}

impl AppConfig {
    pub fn from_map(values: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let get = |key| {
            values
                .get(key)
                .cloned()
                .ok_or(ConfigError::MissingVar(key))
        };

        Ok(Self {
            client_id: get("BAIDU_CLIENT_ID")?,
            client_secret: get("BAIDU_CLIENT_SECRET")?,
            group_id: get("BAIDU_GROUP_ID")?,
            ws_url: get("WS_SERVER_URL")?,
        })
    }
}
