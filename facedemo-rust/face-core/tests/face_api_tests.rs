use face_core::config::AppConfig;
use face_core::face_api::{BaiduFaceApi, HttpMethod, NewFaceUser};

fn config() -> AppConfig {
    AppConfig {
        client_id: "client-id".to_string(),
        client_secret: "client-secret".to_string(),
        group_id: "group1".to_string(),
        ws_url: "ws://47.113.92.62:8081".to_string(),
    }
}

#[test]
fn builds_token_request_using_config_values() {
    let api = BaiduFaceApi::new(config());

    let request = api.token_request();

    assert_eq!(request.method, HttpMethod::Post);
    assert_eq!(request.path, "/oauth/2.0/token");
    assert_eq!(
        request.query,
        vec![
            ("grant_type".to_string(), "client_credentials".to_string()),
            ("client_id".to_string(), "client-id".to_string()),
            ("client_secret".to_string(), "client-secret".to_string()),
        ]
    );
    assert!(request.body.is_none());
}

#[test]
fn builds_add_user_request_with_base64_payload() {
    let api = BaiduFaceApi::new(config());

    let request = api.add_user_request(
        "token-123",
        NewFaceUser {
            user_id: "alice".to_string(),
            user_info: "前台".to_string(),
            image_base64: "YmFzZTY0LWltYWdl".to_string(),
        },
    );

    assert_eq!(request.method, HttpMethod::Post);
    assert_eq!(request.path, "/rest/2.0/face/v3/faceset/user/add");
    assert_eq!(
        request.query,
        vec![("access_token".to_string(), "token-123".to_string())]
    );

    let body = request.body.expect("body should exist");
    assert_eq!(body["group_id"], "group1");
    assert_eq!(body["user_id"], "alice");
    assert_eq!(body["user_info"], "前台");
    assert_eq!(body["image"], "YmFzZTY0LWltYWdl");
    assert_eq!(body["image_type"], "BASE64");
}

#[test]
fn builds_user_query_and_delete_requests_with_group_scope() {
    let api = BaiduFaceApi::new(config());

    let list_request = api.user_list_request("token-123");
    let detail_request = api.user_detail_request("token-123", "alice");
    let delete_request = api.delete_user_request("token-123", "alice");

    assert_eq!(list_request.path, "/rest/2.0/face/v3/faceset/group/getusers");
    assert_eq!(list_request.body.expect("list body")["group_id"], "group1");

    let detail_body = detail_request.body.expect("detail body");
    assert_eq!(detail_request.path, "/rest/2.0/face/v3/faceset/user/get");
    assert_eq!(detail_body["group_id"], "group1");
    assert_eq!(detail_body["user_id"], "alice");

    let delete_body = delete_request.body.expect("delete body");
    assert_eq!(delete_request.path, "/rest/2.0/face/v3/faceset/user/delete");
    assert_eq!(delete_body["group_id"], "group1");
    assert_eq!(delete_body["user_id"], "alice");
}

#[test]
fn parses_user_ids_and_user_details_from_baidu_responses() {
    let api = BaiduFaceApi::new(config());

    let ids = api
        .parse_user_ids(
            r#"{
              "result": {
                "user_id_list": ["alice", "bob"]
              }
            }"#,
        )
        .expect("user ids should parse");
    let user = api
        .parse_user_detail(
            "alice",
            r#"{
              "result": {
                "user_list": [
                  { "user_info": "前台" }
                ]
              }
            }"#,
        )
        .expect("user detail should parse");

    assert_eq!(ids, vec!["alice".to_string(), "bob".to_string()]);
    assert_eq!(user.user_id, "alice");
    assert_eq!(user.user_info, "前台");
}

#[test]
fn rejects_error_payloads_from_baidu() {
    let api = BaiduFaceApi::new(config());

    let error = api
        .parse_access_token(
            r#"{
              "error": "invalid_client",
              "error_description": "unknown client id"
            }"#,
        )
        .expect_err("error payload should fail");

    assert!(error.to_string().contains("invalid_client"));
}

#[test]
fn accepts_success_payloads_with_zero_error_code() {
    let api = BaiduFaceApi::new(config());

    let result = api.parse_success(
        r#"{
          "error_code": 0,
          "error_msg": "SUCCESS"
        }"#,
    );

    assert!(result.is_ok());
}
