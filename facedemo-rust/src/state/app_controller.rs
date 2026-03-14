use face_core::face_api::NewFaceUser;

use crate::services::backend::{BackendError, FaceBackend};
use crate::state::app_state::AppState;

#[derive(Clone)]
pub struct AppController<B> {
    backend: B,
}

impl<B> AppController<B>
where
    B: FaceBackend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub async fn load_users(&self, state: &mut AppState) -> Result<(), BackendError> {
        match self.backend.fetch_users().await {
            Ok(users) => {
                let user_count = users.len();
                state.replace_users(users);
                state.set_status(format!("已加载 {user_count} 个用户"));
                Ok(())
            }
            Err(error) => {
                state.set_error(error.to_string());
                Err(error)
            }
        }
    }

    pub async fn add_user(
        &self,
        state: &mut AppState,
        new_user: NewFaceUser,
    ) -> Result<(), BackendError> {
        match self.backend.add_user(new_user).await {
            Ok(user) => {
                let user_id = user.user_id.clone();
                state.upsert_user(user);
                state.set_status(format!("已添加用户 {user_id}"));
                Ok(())
            }
            Err(error) => {
                state.set_error(error.to_string());
                Err(error)
            }
        }
    }

    pub async fn delete_user(
        &self,
        state: &mut AppState,
        user_id: &str,
    ) -> Result<(), BackendError> {
        match self.backend.delete_user(user_id).await {
            Ok(()) => {
                state.remove_user(user_id);
                state.set_status(format!("已删除用户 {user_id}"));
                Ok(())
            }
            Err(error) => {
                state.set_error(error.to_string());
                Err(error)
            }
        }
    }

    pub async fn start_log_listener(&self, state: &mut AppState) -> Result<(), BackendError> {
        match self.backend.fetch_logs().await {
            Ok(()) => {
                state.set_status("日志监听已启动");
                Ok(())
            }
            Err(error) => {
                state.set_error(error.to_string());
                Err(error)
            }
        }
    }
}
