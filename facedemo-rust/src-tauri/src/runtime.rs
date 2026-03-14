use async_trait::async_trait;
use face_core::client::{FaceApiTransport, TransportError};
use face_core::face_api::{ApiRequest, HttpMethod};
use thiserror::Error;

pub const BAIDU_BASE_URL: &str = "https://aip.baidubce.com";
pub const RECOGNITION_LOG_EVENT: &str = "recognition-logs";

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("http request failed: {0}")]
    Http(String),
}

#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

pub fn build_request_url(base_url: &str, request: &ApiRequest) -> String {
    let base = base_url.trim_end_matches('/');
    let path = request.path.trim_start_matches('/');
    let joined = format!("{base}/{path}");

    let mut url = reqwest::Url::parse(&joined).expect("base url should be valid");
    {
        let mut query_pairs = url.query_pairs_mut();
        for (key, value) in &request.query {
            query_pairs.append_pair(key, value);
        }
    }

    url.into()
}

async fn execute_request(
    client: &reqwest::Client,
    base_url: &str,
    request: &ApiRequest,
) -> Result<String, RuntimeError> {
    let url = build_request_url(base_url, request);
    let builder = match request.method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
    };
    let builder = if let Some(body) = &request.body {
        builder.json(body)
    } else {
        builder
    };

    let response = builder
        .send()
        .await
        .map_err(|error| RuntimeError::Http(error.to_string()))?;

    response
        .text()
        .await
        .map_err(|error| RuntimeError::Http(error.to_string()))
}

#[async_trait]
impl FaceApiTransport for ReqwestTransport {
    async fn send(&self, base_url: &str, request: ApiRequest) -> Result<String, TransportError> {
        execute_request(&self.client, base_url, &request)
            .await
            .map_err(|error| TransportError::new(error.to_string()))
    }
}
