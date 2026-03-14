use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use face_core::client::{FaceApiTransport, FaceClient, TransportError};
use face_core::config::AppConfig;
use face_core::face_api::{ApiRequest, BaiduFaceApi, NewFaceUser};
use face_core::models::{FaceUserSummary, RecognitionLogEntry};
use facedemo_rust::services::backend::{ClientBackedBackend, FaceBackend};
use futures::executor::block_on;

#[derive(Clone)]
struct FakeTransport {
    queue: Arc<Mutex<VecDeque<(ApiRequest, String)>>>,
}

impl FakeTransport {
    fn new(queue: Vec<(ApiRequest, &str)>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(
                queue
                    .into_iter()
                    .map(|(request, response)| (request, response.to_string()))
                    .collect(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl FaceApiTransport for FakeTransport {
    async fn send(&self, _base_url: &str, request: ApiRequest) -> Result<String, TransportError> {
        let Some((expected_request, response)) = self.queue.lock().expect("lock").pop_front() else {
            panic!("unexpected extra request: {request:?}");
        };

        assert_eq!(request, expected_request);
        Ok(response)
    }
}

fn config() -> AppConfig {
    AppConfig {
        client_id: "client-id".to_string(),
        client_secret: "client-secret".to_string(),
        group_id: "group1".to_string(),
        ws_url: "ws://47.113.92.62:8081".to_string(),
    }
}

#[test]
fn client_backed_backend_fetches_users_via_face_client() {
    let api = BaiduFaceApi::new(config());
    let backend = ClientBackedBackend::new(
        FaceClient::new(
            api.clone(),
            "https://aip.baidubce.com",
            FakeTransport::new(vec![
                (
                    api.token_request(),
                    r#"{"access_token":"token-123"}"#,
                ),
                (
                    api.user_list_request("token-123"),
                    r#"{"result":{"user_id_list":["alice"]}}"#,
                ),
                (
                    api.user_detail_request("token-123", "alice"),
                    r#"{"result":{"user_list":[{"user_info":"前台"}]}}"#,
                ),
            ]),
        ),
        vec![RecognitionLogEntry {
            result: true,
            user_info: "张三".to_string(),
            date: "2026-03-15 10:12:00".to_string(),
            image: "img".to_string(),
        }],
    );

    let users = block_on(backend.fetch_users()).expect("users should load");

    assert_eq!(
        users,
        vec![FaceUserSummary {
            user_id: "alice".to_string(),
            user_info: "前台".to_string(),
        }]
    );
}

#[test]
fn client_backed_backend_adds_and_deletes_users_and_keeps_log_fallback() {
    let api = BaiduFaceApi::new(config());
    let backend = ClientBackedBackend::new(
        FaceClient::new(
            api.clone(),
            "https://aip.baidubce.com",
            FakeTransport::new(vec![
                (
                    api.token_request(),
                    r#"{"access_token":"token-123"}"#,
                ),
                (
                    api.add_user_request(
                        "token-123",
                        NewFaceUser {
                            user_id: "alice".to_string(),
                            user_info: "前台".to_string(),
                            image_base64: "base64".to_string(),
                        },
                    ),
                    r#"{"error_code":0,"error_msg":"SUCCESS"}"#,
                ),
                (
                    api.user_detail_request("token-123", "alice"),
                    r#"{"result":{"user_list":[{"user_info":"前台"}]}}"#,
                ),
                (
                    api.token_request(),
                    r#"{"access_token":"token-456"}"#,
                ),
                (
                    api.delete_user_request("token-456", "alice"),
                    r#"{"error_code":0,"error_msg":"SUCCESS"}"#,
                ),
            ]),
        ),
        vec![RecognitionLogEntry {
            result: false,
            user_info: String::new(),
            date: "2026-03-15 10:10:00".to_string(),
            image: "img".to_string(),
        }],
    );

    let user = block_on(backend.add_user(NewFaceUser {
        user_id: "alice".to_string(),
        user_info: "前台".to_string(),
        image_base64: "base64".to_string(),
    }))
    .expect("user should add");
    block_on(backend.delete_user("alice")).expect("user should delete");
    let result = block_on(backend.fetch_logs());

    assert_eq!(user.user_id, "alice");
    assert!(result.is_ok());
}
