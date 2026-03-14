use futures::executor::block_on;

use face_core::face_api::NewFaceUser;
use face_core::models::FaceUserSummary;
use facedemo_rust::services::backend::{BackendError, FaceBackend};
use facedemo_rust::state::app_controller::AppController;
use facedemo_rust::state::app_state::AppState;

#[derive(Clone, Default)]
struct FakeBackend {
    users: Vec<FaceUserSummary>,
    fail_with: Option<&'static str>,
}

#[async_trait::async_trait(?Send)]
impl FaceBackend for FakeBackend {
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError> {
        if let Some(message) = self.fail_with {
            return Err(BackendError::new(message));
        }

        Ok(self.users.clone())
    }

    async fn add_user(&self, new_user: NewFaceUser) -> Result<FaceUserSummary, BackendError> {
        if let Some(message) = self.fail_with {
            return Err(BackendError::new(message));
        }

        Ok(FaceUserSummary {
            user_id: new_user.user_id,
            user_info: new_user.user_info,
        })
    }

    async fn delete_user(&self, _user_id: &str) -> Result<(), BackendError> {
        if let Some(message) = self.fail_with {
            return Err(BackendError::new(message));
        }

        Ok(())
    }

    async fn fetch_logs(&self) -> Result<(), BackendError> {
        if let Some(message) = self.fail_with {
            return Err(BackendError::new(message));
        }

        Ok(())
    }
}

#[test]
fn load_users_replaces_state_and_sets_status() {
    let controller = AppController::new(FakeBackend {
        users: vec![
            FaceUserSummary {
                user_id: "alice".to_string(),
                user_info: "前台".to_string(),
            },
            FaceUserSummary {
                user_id: "bob".to_string(),
                user_info: "访客".to_string(),
            },
        ],
        ..Default::default()
    });
    let mut state = AppState::default();

    block_on(controller.load_users(&mut state)).expect("users should load");

    assert_eq!(state.users.len(), 2);
    assert_eq!(state.users[0].user_id, "alice");
    assert_eq!(state.status_message.as_deref(), Some("已加载 2 个用户"));
    assert_eq!(state.error_message, None);
}

#[test]
fn add_user_updates_state_and_keeps_latest_value() {
    let controller = AppController::new(FakeBackend::default());
    let mut state = AppState::default();

    block_on(controller.add_user(
        &mut state,
        NewFaceUser {
            user_id: "alice".to_string(),
            user_info: "前台".to_string(),
            image_base64: "base64".to_string(),
        },
    ))
    .expect("user should be added");

    assert_eq!(state.users.len(), 1);
    assert_eq!(state.users[0].user_id, "alice");
    assert_eq!(state.status_message.as_deref(), Some("已添加用户 alice"));
}

#[test]
fn delete_user_removes_existing_user_from_state() {
    let controller = AppController::new(FakeBackend::default());
    let mut state = AppState::default();
    state.users = vec![
        FaceUserSummary {
            user_id: "alice".to_string(),
            user_info: "前台".to_string(),
        },
        FaceUserSummary {
            user_id: "bob".to_string(),
            user_info: "访客".to_string(),
        },
    ];

    block_on(controller.delete_user(&mut state, "alice")).expect("delete should succeed");

    assert_eq!(state.users.len(), 1);
    assert_eq!(state.users[0].user_id, "bob");
    assert_eq!(state.status_message.as_deref(), Some("已删除用户 alice"));
}

#[test]
fn start_log_listener_sets_status_message() {
    let controller = AppController::new(FakeBackend::default());
    let mut state = AppState::default();

    block_on(controller.start_log_listener(&mut state)).expect("listener should start");

    assert_eq!(state.status_message.as_deref(), Some("日志监听已启动"));
    assert_eq!(state.error_message, None);
}

#[test]
fn load_users_records_backend_failures_in_state() {
    let controller = AppController::new(FakeBackend {
        fail_with: Some("network down"),
        ..Default::default()
    });
    let mut state = AppState::default();

    let error = block_on(controller.load_users(&mut state)).expect_err("load should fail");

    assert_eq!(error.to_string(), "network down");
    assert_eq!(state.error_message.as_deref(), Some("network down"));
}
