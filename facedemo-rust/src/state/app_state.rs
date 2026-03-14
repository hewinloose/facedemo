use face_core::models::{FaceUserSummary, RecognitionLogEntry};
use face_core::face_api::NewFaceUser;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppTab {
    #[default]
    FaceLibrary,
    RecognitionLog,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub active_tab: AppTab,
    pub users: Vec<FaceUserSummary>,
    pub logs: Vec<RecognitionLogEntry>,
    pub user_draft: UserDraft,
    pub show_add_user_modal: bool,
    pub selected_log_image: Option<String>,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserDraft {
    pub user_id: String,
    pub user_info: String,
    pub image_base64: String,
}

impl UserDraft {
    pub fn can_submit(&self) -> bool {
        !self.user_id.trim().is_empty()
            && !self.user_info.trim().is_empty()
            && !self.image_base64.trim().is_empty()
    }

    pub fn as_new_user(&self) -> Option<NewFaceUser> {
        self.can_submit().then(|| NewFaceUser {
            user_id: self.user_id.trim().to_string(),
            user_info: self.user_info.trim().to_string(),
            image_base64: self.image_base64.trim().to_string(),
        })
    }

    pub fn clear(&mut self) {
        self.user_id.clear();
        self.user_info.clear();
        self.image_base64.clear();
    }
}

impl AppState {
    pub fn with_snapshot(users: Vec<FaceUserSummary>, logs: Vec<RecognitionLogEntry>) -> Self {
        Self::with_snapshot_and_status(users, logs, "已装载示例数据")
    }

    pub fn with_snapshot_and_status(
        users: Vec<FaceUserSummary>,
        logs: Vec<RecognitionLogEntry>,
        status_message: impl Into<String>,
    ) -> Self {
        Self {
            users,
            logs,
            status_message: Some(status_message.into()),
            ..Self::default()
        }
    }

    pub fn replace_users(&mut self, users: Vec<FaceUserSummary>) {
        self.users = users;
    }

    pub fn open_add_user_modal(&mut self) {
        self.show_add_user_modal = true;
        self.error_message = None;
    }

    pub fn close_add_user_modal(&mut self) {
        self.show_add_user_modal = false;
        self.user_draft.clear();
    }

    pub fn update_user_draft_id(&mut self, user_id: String) {
        self.user_draft.user_id = user_id;
    }

    pub fn update_user_draft_info(&mut self, user_info: String) {
        self.user_draft.user_info = user_info;
    }

    pub fn update_user_draft_image(&mut self, image_base64: String) {
        self.user_draft.image_base64 = image_base64;
    }

    pub fn upsert_user(&mut self, user: FaceUserSummary) {
        if let Some(existing_user) = self
            .users
            .iter_mut()
            .find(|existing_user| existing_user.user_id == user.user_id)
        {
            *existing_user = user;
            return;
        }

        self.users.push(user);
    }

    pub fn prepend_logs(&mut self, logs: Vec<RecognitionLogEntry>) {
        self.logs.splice(0..0, logs);
    }

    pub fn remove_user(&mut self, user_id: &str) {
        self.users.retain(|user| user.user_id != user_id);
    }

    pub fn set_active_tab(&mut self, tab: AppTab) {
        self.active_tab = tab;
    }

    pub fn select_log_image(&mut self, image_base64: String) {
        self.selected_log_image = Some(image_base64);
    }

    pub fn clear_selected_log_image(&mut self) {
        self.selected_log_image = None;
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.error_message = None;
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }
}
