use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::config::AppConfig;
use crate::models::FaceUserSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiRequest {
    pub method: HttpMethod,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewFaceUser {
    pub user_id: String,
    pub user_info: String,
    pub image_base64: String,
}

#[derive(Debug, Error)]
pub enum FaceApiError {
    #[error("failed to parse baidu payload: {0}")]
    InvalidPayload(String),
    #[error("baidu api error: {0}")]
    Remote(String),
    #[error("missing field in baidu payload: {0}")]
    MissingField(&'static str),
}

#[derive(Debug, Clone)]
pub struct BaiduFaceApi {
    config: AppConfig,
}

impl BaiduFaceApi {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn token_request(&self) -> ApiRequest {
        ApiRequest {
            method: HttpMethod::Post,
            path: "/oauth/2.0/token".to_string(),
            query: vec![
                ("grant_type".to_string(), "client_credentials".to_string()),
                ("client_id".to_string(), self.config.client_id.clone()),
                (
                    "client_secret".to_string(),
                    self.config.client_secret.clone(),
                ),
            ],
            body: None,
        }
    }

    pub fn user_list_request(&self, access_token: &str) -> ApiRequest {
        ApiRequest {
            method: HttpMethod::Post,
            path: "/rest/2.0/face/v3/faceset/group/getusers".to_string(),
            query: vec![("access_token".to_string(), access_token.to_string())],
            body: Some(serde_json::json!({
                "group_id": self.config.group_id,
            })),
        }
    }

    pub fn user_detail_request(&self, access_token: &str, user_id: &str) -> ApiRequest {
        ApiRequest {
            method: HttpMethod::Post,
            path: "/rest/2.0/face/v3/faceset/user/get".to_string(),
            query: vec![("access_token".to_string(), access_token.to_string())],
            body: Some(serde_json::json!({
                "group_id": self.config.group_id,
                "user_id": user_id,
            })),
        }
    }

    pub fn add_user_request(&self, access_token: &str, new_user: NewFaceUser) -> ApiRequest {
        ApiRequest {
            method: HttpMethod::Post,
            path: "/rest/2.0/face/v3/faceset/user/add".to_string(),
            query: vec![("access_token".to_string(), access_token.to_string())],
            body: Some(serde_json::json!({
                "group_id": self.config.group_id,
                "user_id": new_user.user_id,
                "user_info": new_user.user_info,
                "image": new_user.image_base64,
                "image_type": "BASE64",
                "quality_control": "LOW",
                "liveness_control": "LOW",
            })),
        }
    }

    pub fn delete_user_request(&self, access_token: &str, user_id: &str) -> ApiRequest {
        ApiRequest {
            method: HttpMethod::Post,
            path: "/rest/2.0/face/v3/faceset/user/delete".to_string(),
            query: vec![("access_token".to_string(), access_token.to_string())],
            body: Some(serde_json::json!({
                "group_id": self.config.group_id,
                "user_id": user_id,
            })),
        }
    }

    pub fn parse_access_token(&self, payload: &str) -> Result<String, FaceApiError> {
        let value = parse_payload(payload)?;
        ensure_success(&value)?;

        value
            .get("access_token")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(FaceApiError::MissingField("access_token"))
    }

    pub fn parse_user_ids(&self, payload: &str) -> Result<Vec<String>, FaceApiError> {
        let value = parse_payload(payload)?;
        ensure_success(&value)?;

        let ids = value
            .get("result")
            .and_then(|result| result.get("user_id_list"))
            .and_then(Value::as_array)
            .ok_or(FaceApiError::MissingField("result.user_id_list"))?;

        Ok(ids
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect())
    }

    pub fn parse_user_detail(
        &self,
        user_id: &str,
        payload: &str,
    ) -> Result<FaceUserSummary, FaceApiError> {
        let value = parse_payload(payload)?;
        ensure_success(&value)?;

        let user_info = value
            .get("result")
            .and_then(|result| result.get("user_list"))
            .and_then(Value::as_array)
            .and_then(|user_list| user_list.first())
            .and_then(|user| user.get("user_info"))
            .and_then(Value::as_str)
            .ok_or(FaceApiError::MissingField("result.user_list[0].user_info"))?;

        Ok(FaceUserSummary {
            user_id: user_id.to_string(),
            user_info: user_info.to_string(),
        })
    }

    pub fn parse_success(&self, payload: &str) -> Result<(), FaceApiError> {
        let value = parse_payload(payload)?;
        ensure_success(&value)
    }
}

fn parse_payload(payload: &str) -> Result<Value, FaceApiError> {
    serde_json::from_str(payload).map_err(|error| FaceApiError::InvalidPayload(error.to_string()))
}

fn ensure_success(value: &Value) -> Result<(), FaceApiError> {
    if let Some(error_message) = value.get("error").and_then(Value::as_str) {
        return Err(FaceApiError::Remote(error_message.to_string()));
    }

    match value.get("error_code").and_then(Value::as_i64) {
        Some(0) | None => Ok(()),
        Some(code) => {
            let message = value
                .get("error_msg")
                .and_then(Value::as_str)
                .unwrap_or("unknown baidu api error");
            Err(FaceApiError::Remote(format!("{code}: {message}")))
        }
    }
}
