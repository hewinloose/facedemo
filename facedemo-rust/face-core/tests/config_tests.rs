use std::collections::HashMap;

use face_core::config::AppConfig;

fn build_env() -> HashMap<String, String> {
    HashMap::from([
        ("BAIDU_CLIENT_ID".to_string(), "client-id".to_string()),
        (
            "BAIDU_CLIENT_SECRET".to_string(),
            "client-secret".to_string(),
        ),
        ("BAIDU_GROUP_ID".to_string(), "group1".to_string()),
        (
            "WS_SERVER_URL".to_string(),
            "ws://47.113.92.62:8081".to_string(),
        ),
    ])
}

#[test]
fn loads_all_required_values_from_a_map() {
    let config = AppConfig::from_map(&build_env()).expect("config should load");

    assert_eq!(config.client_id, "client-id");
    assert_eq!(config.client_secret, "client-secret");
    assert_eq!(config.group_id, "group1");
    assert_eq!(config.ws_url, "ws://47.113.92.62:8081");
}

#[test]
fn returns_the_missing_key_when_required_value_is_absent() {
    let mut env = build_env();
    env.remove("BAIDU_CLIENT_SECRET");

    let error = AppConfig::from_map(&env).expect_err("missing env var should fail");

    assert_eq!(error.missing_key(), "BAIDU_CLIENT_SECRET");
}
