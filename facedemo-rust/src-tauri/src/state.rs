use std::sync::Arc;

use face_core::config::AppConfig;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::runtime::ReqwestTransport;

#[derive(Debug)]
pub struct AppServices {
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub transport: ReqwestTransport,
    pub websocket_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl AppServices {
    pub fn new(config: AppConfig) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            config,
            transport: ReqwestTransport::new(http_client.clone()),
            http_client,
            websocket_task: Arc::new(Mutex::new(None)),
        }
    }
}
