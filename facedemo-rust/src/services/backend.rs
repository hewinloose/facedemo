use async_trait::async_trait;
use face_core::client::{FaceApiTransport, FaceClient, TransportError};
use face_core::config::{AppConfig, ConfigError};
use face_core::face_api::{ApiRequest, BaiduFaceApi, NewFaceUser};
use face_core::models::{FaceUserSummary, RecognitionLogEntry};
use thiserror::Error;

#[cfg(feature = "tauri-backend")]
use crate::services::tauri_bridge::{
    invoke_no_args, invoke_unit, invoke_unit_with_args, invoke_with_args,
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct BackendError {
    message: String,
}

impl BackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<face_core::client::FaceClientError> for BackendError {
    fn from(value: face_core::client::FaceClientError) -> Self {
        Self::new(value.to_string())
    }
}

impl From<ConfigError> for BackendError {
    fn from(value: ConfigError) -> Self {
        Self::new(value.to_string())
    }
}

#[async_trait(?Send)]
pub trait FaceBackend: Clone + Send + Sync + 'static {
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError>;
    async fn add_user(&self, new_user: NewFaceUser) -> Result<FaceUserSummary, BackendError>;
    async fn delete_user(&self, user_id: &str) -> Result<(), BackendError>;
    async fn fetch_logs(&self) -> Result<(), BackendError>;
}

pub const BAIDU_BASE_URL: &str = "https://aip.baidubce.com";

#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl FaceApiTransport for ReqwestTransport {
    async fn send(&self, base_url: &str, request: ApiRequest) -> Result<String, TransportError> {
        let base = base_url.trim_end_matches('/');
        let path = request.path.trim_start_matches('/');
        let joined = format!("{base}/{path}");
        let mut url =
            reqwest::Url::parse(&joined).map_err(|error| TransportError::new(error.to_string()))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in &request.query {
                query_pairs.append_pair(key, value);
            }
        }

        let builder = match request.method {
            face_core::face_api::HttpMethod::Get => self.client.get(url),
            face_core::face_api::HttpMethod::Post => self.client.post(url),
        };
        let builder = if let Some(body) = &request.body {
            builder.json(body)
        } else {
            builder
        };

        let response = builder
            .send()
            .await
            .map_err(|error| TransportError::new(error.to_string()))?;

        response
            .text()
            .await
            .map_err(|error| TransportError::new(error.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct ClientBackedBackend<T> {
    client: FaceClient<T>,
    logs_seed: Vec<RecognitionLogEntry>,
}

impl<T> ClientBackedBackend<T>
where
    T: FaceApiTransport,
{
    pub fn new(client: FaceClient<T>, logs_seed: Vec<RecognitionLogEntry>) -> Self {
        Self { client, logs_seed }
    }
}

pub type HttpFaceBackend = ClientBackedBackend<ReqwestTransport>;

impl HttpFaceBackend {
    pub fn from_config(config: AppConfig) -> Self {
        let client = FaceClient::new(
            BaiduFaceApi::new(config),
            BAIDU_BASE_URL,
            ReqwestTransport::new(reqwest::Client::new()),
        );

        Self::new(client, DemoBackend::sample_logs())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn try_from_env() -> Result<Self, BackendError> {
        dotenvy::dotenv().ok();

        let mut values = std::collections::HashMap::new();
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

        Ok(Self::from_config(AppConfig::from_map(&values)?))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn try_from_env() -> Result<Self, BackendError> {
        Err(BackendError::new(
            "wasm 运行时不支持从 .env 读取 HTTP 后端配置",
        ))
    }
}

#[async_trait(?Send)]
impl<T> FaceBackend for ClientBackedBackend<T>
where
    T: FaceApiTransport,
{
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError> {
        self.client.fetch_users(None).await.map_err(Into::into)
    }

    async fn add_user(&self, new_user: NewFaceUser) -> Result<FaceUserSummary, BackendError> {
        self.client.add_user(new_user).await.map_err(Into::into)
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), BackendError> {
        self.client.delete_user(user_id, None).await.map_err(Into::into)
    }

    async fn fetch_logs(&self) -> Result<(), BackendError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DemoBackend;

impl DemoBackend {
    pub fn sample_users() -> Vec<FaceUserSummary> {
        vec![
            FaceUserSummary {
                user_id: "zhangsan".to_string(),
                user_info: "前台".to_string(),
            },
            FaceUserSummary {
                user_id: "lisi".to_string(),
                user_info: "访客".to_string(),
            },
        ]
    }

    pub fn sample_logs() -> Vec<RecognitionLogEntry> {
        vec![
            RecognitionLogEntry {
                result: true,
                user_info: "张三".to_string(),
                date: "2026-03-15 10:12:00".to_string(),
                image: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8Xw8AAoMBgA4WXTcAAAAASUVORK5CYII=".to_string(),
            },
            RecognitionLogEntry {
                result: false,
                user_info: String::new(),
                date: "2026-03-15 10:10:00".to_string(),
                image: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8Xw8AAoMBgA4WXTcAAAAASUVORK5CYII=".to_string(),
            },
        ]
    }
}

#[async_trait(?Send)]
impl FaceBackend for DemoBackend {
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError> {
        Ok(Self::sample_users())
    }

    async fn add_user(&self, new_user: NewFaceUser) -> Result<FaceUserSummary, BackendError> {
        Ok(FaceUserSummary {
            user_id: new_user.user_id,
            user_info: new_user.user_info,
        })
    }

    async fn delete_user(&self, _user_id: &str) -> Result<(), BackendError> {
        Ok(())
    }

    async fn fetch_logs(&self) -> Result<(), BackendError> {
        Ok(())
    }
}

#[cfg(feature = "tauri-backend")]
#[derive(Debug, Clone, Default)]
pub struct TauriInvokeBackend;

#[cfg(feature = "tauri-backend")]
#[async_trait(?Send)]
impl FaceBackend for TauriInvokeBackend {
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError> {
        invoke_no_args("get_user_list")
            .await
            .map_err(BackendError::new)
    }

    async fn add_user(&self, new_user: NewFaceUser) -> Result<FaceUserSummary, BackendError> {
        invoke_with_args("add_user", &InvokeAddUserArgs { new_user })
            .await
            .map_err(BackendError::new)
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), BackendError> {
        invoke_unit_with_args(
            "delete_user",
            &InvokeDeleteUserArgs {
                user_id: user_id.to_string(),
                access_token: None::<String>,
            },
        )
        .await
        .map_err(BackendError::new)
    }

    async fn fetch_logs(&self) -> Result<(), BackendError> {
        invoke_unit("start_websocket_listener")
            .await
            .map_err(BackendError::new)
    }
}

#[cfg(feature = "tauri-backend")]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InvokeAddUserArgs {
    new_user: NewFaceUser,
}

#[cfg(feature = "tauri-backend")]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InvokeDeleteUserArgs<T> {
    user_id: String,
    access_token: Option<T>,
}

#[derive(Debug, Clone)]
pub enum AppBackend {
    Demo(DemoBackend),
    Http(HttpFaceBackend),
    #[cfg(feature = "tauri-backend")]
    Tauri(TauriInvokeBackend),
}

impl AppBackend {
    pub fn from_env_or_demo() -> Self {
        #[cfg(feature = "tauri-backend")]
        {
            return Self::Tauri(TauriInvokeBackend);
        }

        #[cfg(all(not(feature = "tauri-backend"), target_arch = "wasm32"))]
        {
            return Self::Demo(DemoBackend);
        }

        #[cfg(all(not(feature = "tauri-backend"), not(target_arch = "wasm32")))]
        {
            HttpFaceBackend::try_from_env()
                .map(Self::Http)
                .unwrap_or_else(|_| Self::Demo(DemoBackend))
        }
    }

    pub fn initial_users(&self) -> Vec<FaceUserSummary> {
        match self {
            Self::Demo(_) => DemoBackend::sample_users(),
            Self::Http(_) => Vec::new(),
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(_) => Vec::new(),
        }
    }

    pub fn initial_logs(&self) -> Vec<RecognitionLogEntry> {
        match self {
            Self::Demo(_) => DemoBackend::sample_logs(),
            Self::Http(backend) => backend.logs_seed.clone(),
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(_) => Vec::new(),
        }
    }

    pub fn initial_status(&self) -> &'static str {
        match self {
            Self::Demo(_) => "已装载示例数据",
            Self::Http(_) => "已启用真实后端，点击刷新开始加载",
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(_) => "已启用 Tauri 后端，进入日志页后自动启动监听",
        }
    }
}

#[async_trait(?Send)]
impl FaceBackend for AppBackend {
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError> {
        match self {
            Self::Demo(backend) => backend.fetch_users().await,
            Self::Http(backend) => backend.fetch_users().await,
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(backend) => backend.fetch_users().await,
        }
    }

    async fn add_user(&self, new_user: NewFaceUser) -> Result<FaceUserSummary, BackendError> {
        match self {
            Self::Demo(backend) => backend.add_user(new_user).await,
            Self::Http(backend) => backend.add_user(new_user).await,
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(backend) => backend.add_user(new_user).await,
        }
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), BackendError> {
        match self {
            Self::Demo(backend) => backend.delete_user(user_id).await,
            Self::Http(backend) => backend.delete_user(user_id).await,
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(backend) => backend.delete_user(user_id).await,
        }
    }

    async fn fetch_logs(&self) -> Result<(), BackendError> {
        match self {
            Self::Demo(backend) => backend.fetch_logs().await,
            Self::Http(backend) => backend.fetch_logs().await,
            #[cfg(feature = "tauri-backend")]
            Self::Tauri(backend) => backend.fetch_logs().await,
        }
    }
}
