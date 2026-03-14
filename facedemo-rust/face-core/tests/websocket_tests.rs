use face_core::models::RecognitionLogEntry;
use face_core::websocket::parse_log_entries;

#[test]
fn parses_recognition_log_entries_from_websocket_payload() {
    let logs = parse_log_entries(
        r#"[
          {
            "result": true,
            "user_info": "张三",
            "date": "2026-03-15 09:12:00",
            "image": "base64-image"
          },
          {
            "result": false,
            "user_info": "",
            "date": "2026-03-15 09:13:00",
            "image": "base64-image-2"
          }
        ]"#,
    )
    .expect("payload should parse");

    assert_eq!(
        logs,
        vec![
            RecognitionLogEntry {
                result: true,
                user_info: "张三".to_string(),
                date: "2026-03-15 09:12:00".to_string(),
                image: "base64-image".to_string(),
            },
            RecognitionLogEntry {
                result: false,
                user_info: "".to_string(),
                date: "2026-03-15 09:13:00".to_string(),
                image: "base64-image-2".to_string(),
            }
        ]
    );
}

#[test]
fn returns_a_parse_error_for_invalid_payloads() {
    let error = parse_log_entries("{ invalid json ]").expect_err("invalid json should fail");

    assert!(error.to_string().contains("websocket"));
}
