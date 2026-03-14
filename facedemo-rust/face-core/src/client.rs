use async_trait::async_trait;
use thiserror::Error;

use crate::face_api::{ApiRequest, BaiduFaceApi, FaceApiError, NewFaceUser};
use crate::models::FaceUserSummary;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct TransportError {
    message: String,
}

impl TransportError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum FaceClientError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Api(#[from] FaceApiError),
}

#[async_trait]
pub trait FaceApiTransport: Clone + Send + Sync + 'static {
    async fn send(&self, base_url: &str, request: ApiRequest) -> Result<String, TransportError>;
}

#[derive(Debug, Clone)]
pub struct FaceClient<T> {
    api: BaiduFaceApi,
    base_url: String,
    transport: T,
}

impl<T> FaceClient<T>
where
    T: FaceApiTransport,
{
    pub fn new(api: BaiduFaceApi, base_url: impl Into<String>, transport: T) -> Self {
        Self {
            api,
            base_url: base_url.into(),
            transport,
        }
    }

    pub async fn fetch_token(&self) -> Result<String, FaceClientError> {
        let payload = self
            .transport
            .send(&self.base_url, self.api.token_request())
            .await?;

        Ok(self.api.parse_access_token(&payload)?)
    }

    pub async fn fetch_users(
        &self,
        access_token: Option<&str>,
    ) -> Result<Vec<FaceUserSummary>, FaceClientError> {
        let token = self.resolve_token(access_token).await?;
        let payload = self
            .transport
            .send(&self.base_url, self.api.user_list_request(&token))
            .await?;
        let user_ids = self.api.parse_user_ids(&payload)?;
        let mut users = Vec::with_capacity(user_ids.len());

        for user_id in user_ids {
            let detail_payload = self
                .transport
                .send(&self.base_url, self.api.user_detail_request(&token, &user_id))
                .await?;
            users.push(self.api.parse_user_detail(&user_id, &detail_payload)?);
        }

        Ok(users)
    }

    pub async fn fetch_user_info(
        &self,
        user_id: &str,
        access_token: Option<&str>,
    ) -> Result<FaceUserSummary, FaceClientError> {
        let token = self.resolve_token(access_token).await?;
        let payload = self
            .transport
            .send(&self.base_url, self.api.user_detail_request(&token, user_id))
            .await?;

        Ok(self.api.parse_user_detail(user_id, &payload)?)
    }

    pub async fn add_user(
        &self,
        new_user: NewFaceUser,
    ) -> Result<FaceUserSummary, FaceClientError> {
        let token = self.resolve_token(None).await?;
        let user_id = new_user.user_id.clone();
        let payload = self
            .transport
            .send(&self.base_url, self.api.add_user_request(&token, new_user))
            .await?;
        self.api.parse_success(&payload)?;

        self.fetch_user_info(&user_id, Some(&token)).await
    }

    pub async fn delete_user(
        &self,
        user_id: &str,
        access_token: Option<&str>,
    ) -> Result<(), FaceClientError> {
        let token = self.resolve_token(access_token).await?;
        let payload = self
            .transport
            .send(&self.base_url, self.api.delete_user_request(&token, user_id))
            .await?;

        Ok(self.api.parse_success(&payload)?)
    }

    async fn resolve_token(&self, access_token: Option<&str>) -> Result<String, FaceClientError> {
        if let Some(access_token) = access_token {
            return Ok(access_token.to_string());
        }

        self.fetch_token().await
    }
}
