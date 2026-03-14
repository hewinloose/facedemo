use face_core::face_api::NewFaceUser;
use face_core::models::{FaceUserSummary, RecognitionLogEntry};
use facedemo_rust::state::app_state::{AppState, AppTab};

#[test]
fn upsert_user_adds_new_users_and_updates_existing_ones() {
    let mut state = AppState::default();

    state.upsert_user(FaceUserSummary {
        user_id: "alice".to_string(),
        user_info: "前台".to_string(),
    });
    state.upsert_user(FaceUserSummary {
        user_id: "alice".to_string(),
        user_info: "前台值班".to_string(),
    });
    state.upsert_user(FaceUserSummary {
        user_id: "bob".to_string(),
        user_info: "访客".to_string(),
    });

    assert_eq!(state.users.len(), 2);
    assert_eq!(state.users[0].user_info, "前台值班");
    assert_eq!(state.users[1].user_id, "bob");
}

#[test]
fn prepend_logs_keeps_latest_messages_at_the_front() {
    let mut state = AppState::default();
    state.logs.push(RecognitionLogEntry {
        result: true,
        user_info: "旧记录".to_string(),
        date: "2026-03-15 09:00:00".to_string(),
        image: "old-image".to_string(),
    });

    state.prepend_logs(vec![
        RecognitionLogEntry {
            result: true,
            user_info: "新记录1".to_string(),
            date: "2026-03-15 09:01:00".to_string(),
            image: "new-image-1".to_string(),
        },
        RecognitionLogEntry {
            result: false,
            user_info: "新记录2".to_string(),
            date: "2026-03-15 09:02:00".to_string(),
            image: "new-image-2".to_string(),
        },
    ]);

    assert_eq!(state.logs.len(), 3);
    assert_eq!(state.logs[0].user_info, "新记录1");
    assert_eq!(state.logs[1].user_info, "新记录2");
    assert_eq!(state.logs[2].user_info, "旧记录");
}

#[test]
fn switches_the_active_tab_explicitly() {
    let mut state = AppState::default();

    state.set_active_tab(AppTab::RecognitionLog);

    assert_eq!(state.active_tab, AppTab::RecognitionLog);
}

#[test]
fn manages_user_draft_for_add_user_modal() {
    let mut state = AppState::default();

    state.open_add_user_modal();
    state.update_user_draft_id("alice".to_string());
    state.update_user_draft_info("前台".to_string());
    state.update_user_draft_image("YmFzZTY0".to_string());

    assert!(state.show_add_user_modal);
    assert!(state.user_draft.can_submit());
    assert_eq!(
        state.user_draft.as_new_user(),
        Some(NewFaceUser {
            user_id: "alice".to_string(),
            user_info: "前台".to_string(),
            image_base64: "YmFzZTY0".to_string(),
        })
    );

    state.close_add_user_modal();

    assert!(!state.show_add_user_modal);
    assert!(state.user_draft.user_id.is_empty());
    assert!(state.user_draft.user_info.is_empty());
    assert!(state.user_draft.image_base64.is_empty());
}

#[test]
fn selects_and_clears_log_image() {
    let mut state = AppState::default();

    state.select_log_image("base64-image".to_string());
    assert_eq!(state.selected_log_image.as_deref(), Some("base64-image"));

    state.clear_selected_log_image();
    assert_eq!(state.selected_log_image, None);
}
