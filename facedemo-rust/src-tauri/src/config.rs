use std::collections::HashMap;

use face_core::config::{AppConfig, ConfigError};

pub fn load_config_from_env() -> Result<AppConfig, ConfigError> {
    dotenvy::dotenv().ok();

    let mut values = HashMap::new();
    for key in [
        "BAIDU_CLIENT_ID",
        "BAIDU_CLIENT_SECRET",
        "BAIDU_GROUP_ID",
        "WS_SERVER_URL",
    ] {
        if let Ok(value) = std::env::var(key) {
            values.insert(key.to_string(), value);
        }
    }

    AppConfig::from_map(&values)
}
