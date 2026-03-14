use face_core::client::FaceClient;
use face_core::face_api::{BaiduFaceApi, NewFaceUser};
use face_core::models::FaceUserSummary;
use tauri::State;

use crate::runtime::BAIDU_BASE_URL;
use crate::state::AppServices;

#[tauri::command]
pub async fn get_baidu_token(services: State<'_, AppServices>) -> Result<String, String> {
    client(&services)
        .fetch_token()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_user_list(
    access_token: Option<String>,
    services: State<'_, AppServices>,
) -> Result<Vec<FaceUserSummary>, String> {
    client(&services)
        .fetch_users(access_token.as_deref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_user_info(
    user_id: String,
    access_token: Option<String>,
    services: State<'_, AppServices>,
) -> Result<FaceUserSummary, String> {
    client(&services)
        .fetch_user_info(&user_id, access_token.as_deref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn add_user(
    new_user: NewFaceUser,
    services: State<'_, AppServices>,
) -> Result<FaceUserSummary, String> {
    client(&services)
        .add_user(new_user)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_user(
    user_id: String,
    access_token: Option<String>,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    client(&services)
        .delete_user(&user_id, access_token.as_deref())
        .await
        .map_err(|error| error.to_string())
}

fn client(services: &AppServices) -> FaceClient<crate::runtime::ReqwestTransport> {
    FaceClient::new(
        BaiduFaceApi::new(services.config.clone()),
        BAIDU_BASE_URL,
        services.transport.clone(),
    )
}
