use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use face_core::client::{FaceApiTransport, FaceClient, TransportError};
use face_core::config::AppConfig;
use face_core::face_api::{ApiRequest, NewFaceUser};
use face_core::models::FaceUserSummary;
use futures::executor::block_on;

#[derive(Clone)]
struct FakeTransport {
    calls: Arc<Mutex<VecDeque<(ApiRequest, String)>>>,
}

impl FakeTransport {
    fn new(calls: Vec<(ApiRequest, &str)>) -> Self {
        Self {
            calls: Arc::new(Mutex::new(
                calls
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
        let Some((expected_request, response)) = self.calls.lock().expect("lock").pop_front() else {
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
fn fetch_token_uses_transport_and_returns_access_token() {
    let api = face_core::face_api::BaiduFaceApi::new(config());
    let client = FaceClient::new(
        api.clone(),
        "https://aip.baidubce.com",
        FakeTransport::new(vec![(
            api.token_request(),
            r#"{"access_token":"token-123"}"#,
        )]),
    );

    let token = block_on(client.fetch_token()).expect("token should load");

    assert_eq!(token, "token-123");
}

#[test]
fn fetch_users_chains_list_and_detail_requests() {
    let api = face_core::face_api::BaiduFaceApi::new(config());
    let client = FaceClient::new(
        api.clone(),
        "https://aip.baidubce.com",
        FakeTransport::new(vec![
            (
                api.user_list_request("token-123"),
                r#"{"result":{"user_id_list":["alice","bob"]}}"#,
            ),
            (
                api.user_detail_request("token-123", "alice"),
                r#"{"result":{"user_list":[{"user_info":"前台"}]}}"#,
            ),
            (
                api.user_detail_request("token-123", "bob"),
                r#"{"result":{"user_list":[{"user_info":"访客"}]}}"#,
            ),
        ]),
    );

    let users = block_on(client.fetch_users(Some("token-123"))).expect("users should load");

    assert_eq!(
        users,
        vec![
            FaceUserSummary {
                user_id: "alice".to_string(),
                user_info: "前台".to_string(),
            },
            FaceUserSummary {
                user_id: "bob".to_string(),
                user_info: "访客".to_string(),
            }
        ]
    );
}

#[test]
fn add_and_delete_user_reuse_token_resolution_when_missing() {
    let api = face_core::face_api::BaiduFaceApi::new(config());
    let transport = FakeTransport::new(vec![
        (api.token_request(), r#"{"access_token":"token-123"}"#),
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
        (api.token_request(), r#"{"access_token":"token-456"}"#),
        (
            api.delete_user_request("token-456", "alice"),
            r#"{"error_code":0,"error_msg":"SUCCESS"}"#,
        ),
    ]);
    let client = FaceClient::new(api, "https://aip.baidubce.com", transport);

    block_on(client.add_user(NewFaceUser {
        user_id: "alice".to_string(),
        user_info: "前台".to_string(),
        image_base64: "base64".to_string(),
    }))
    .expect("add should succeed");

    block_on(client.delete_user("alice", None)).expect("delete should succeed");
}
