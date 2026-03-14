pub mod commands;
pub mod config;
pub mod runtime;
pub mod state;

use crate::state::AppServices;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_target(false).try_init().ok();

    let config = config::load_config_from_env().expect("failed to load app config");
    let services = AppServices::new(config);

    tauri::Builder::default()
        .manage(services)
        .invoke_handler(tauri::generate_handler![
            commands::camera::pick_image,
            commands::camera::take_photo,
            commands::face_api::get_baidu_token,
            commands::face_api::get_user_list,
            commands::face_api::get_user_info,
            commands::face_api::add_user,
            commands::face_api::delete_user,
            commands::websocket::start_websocket_listener,
        ])
        .setup(|app| {
            let _ = app.handle();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
