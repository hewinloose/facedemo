use face_core::face_api::{ApiRequest, HttpMethod};
use src_tauri::runtime::{build_request_url, RECOGNITION_LOG_EVENT};

#[test]
fn builds_urls_from_base_paths_and_query_parameters() {
    let url = build_request_url(
        "https://aip.baidubce.com/",
        &ApiRequest {
            method: HttpMethod::Post,
            path: "/rest/2.0/face/v3/faceset/user/get".to_string(),
            query: vec![
                ("access_token".to_string(), "token-123".to_string()),
                ("group".to_string(), "前台 1".to_string()),
            ],
            body: None,
        },
    );

    assert_eq!(
        url,
        "https://aip.baidubce.com/rest/2.0/face/v3/faceset/user/get?access_token=token-123&group=%E5%89%8D%E5%8F%B0+1"
    );
}

#[test]
fn keeps_the_websocket_event_name_stable() {
    assert_eq!(RECOGNITION_LOG_EVENT, "recognition-logs");
}
