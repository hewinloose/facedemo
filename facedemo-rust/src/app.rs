use dioxus::prelude::*;
#[cfg(feature = "tauri-backend")]
use std::{cell::RefCell, rc::Rc};

use crate::components::{image_viewer::ImageViewer, user_info_modal::UserInfoModal};
use crate::pages::{face_library::FaceLibraryPage, recognition_log::RecognitionLogPage};
use crate::services::backend::AppBackend;
use crate::state::app_controller::AppController;
use crate::state::app_state::{AppState, AppTab};
use crate::theme::variables::{APP_STYLES, APP_TITLE};

#[component]
pub fn App() -> Element {
    let backend = AppBackend::from_env_or_demo();
    let controller = AppController::new(backend.clone());
    let load_users_controller = controller.clone();
    let delete_user_controller = controller.clone();
    let refresh_logs_controller = controller.clone();
    let add_user_controller = controller.clone();
    let mut state = use_signal(move || {
        AppState::with_snapshot_and_status(
            backend.initial_users(),
            backend.initial_logs(),
            backend.initial_status(),
        )
    });
    let snapshot = state.read().clone();
    let active_tab = snapshot.active_tab.clone();
    let users = snapshot.users.clone();
    let logs = snapshot.logs.clone();
    let status_message = snapshot.status_message.clone();
    let error_message = snapshot.error_message.clone();
    let user_draft = snapshot.user_draft.clone();
    let selected_log_image = snapshot.selected_log_image.clone();
    let show_add_user_modal = snapshot.show_add_user_modal;

    #[cfg(feature = "tauri-backend")]
    {
        let previous_tab = use_hook(|| Rc::new(RefCell::new(AppTab::FaceLibrary)));
        let previous_tab = previous_tab.clone();
        let auto_start_controller = controller.clone();
        let auto_start_state = state;

        use_effect(move || {
            let active_tab = auto_start_state.read().active_tab.clone();
            let should_start = {
                let mut previous_tab = previous_tab.borrow_mut();
                let should_start = !matches!(*previous_tab, AppTab::RecognitionLog)
                    && matches!(active_tab, AppTab::RecognitionLog);
                *previous_tab = active_tab.clone();
                should_start
            };

            if !should_start {
                return;
            }

            let auto_start_controller = auto_start_controller.clone();
            let mut state = auto_start_state;
            spawn(async move {
                let mut next_state = state.read().clone();
                let _ = auto_start_controller.start_log_listener(&mut next_state).await;
                state.set(next_state);
            });
        });
    }

    rsx! {
        document::Title { "{APP_TITLE}" }
        style { "{APP_STYLES}" }
        div { class: "app-shell",
            header { class: "app-header",
                div {
                    p { class: "eyebrow", "Rust + Tauri v2 + Dioxus" }
                    h1 { "{APP_TITLE}" }
                    p { class: "subtitle", "迁移中的最小可运行壳，保留了人脸库和识别日志两条主流程。" }
                    if let Some(status_message) = status_message {
                        p { class: "banner success", "{status_message}" }
                    }
                    if let Some(error_message) = error_message {
                        p { class: "banner error", "{error_message}" }
                    }
                }
            }
            main { class: "app-content",
                match active_tab {
                    AppTab::FaceLibrary => rsx! {
                        FaceLibraryPage {
                            users: users.clone(),
                            on_refresh: move |_| {
                                let controller = load_users_controller.clone();
                                let mut state = state;
                                async move {
                                    let mut next_state = state.read().clone();
                                    let _ = controller.load_users(&mut next_state).await;
                                    state.set(next_state);
                                }
                            },
                            on_open_add: move |_| state.write().open_add_user_modal(),
                            on_delete: move |user_id: String| {
                                let controller = delete_user_controller.clone();
                                let mut state = state;
                                async move {
                                    let mut next_state = state.read().clone();
                                    let _ = controller.delete_user(&mut next_state, &user_id).await;
                                    state.set(next_state);
                                }
                            }
                        }
                    },
                    AppTab::RecognitionLog => rsx! {
                        RecognitionLogPage {
                            logs: logs.clone(),
                            on_refresh: move |_| {
                                let controller = refresh_logs_controller.clone();
                                let mut state = state;
                                async move {
                                    let mut next_state = state.read().clone();
                                    let _ = controller.start_log_listener(&mut next_state).await;
                                    state.set(next_state);
                                }
                            },
                            on_select_image: move |image: String| state.write().select_log_image(image),
                            on_new_logs: move |logs: Vec<face_core::models::RecognitionLogEntry>| {
                                let mut next_state = state.read().clone();
                                next_state.prepend_logs(logs);
                                state.set(next_state);
                            },
                        }
                    },
                }
            }
            footer { class: "tab-bar",
                TabButton {
                    label: "人脸库",
                    active: matches!(active_tab, AppTab::FaceLibrary),
                    onclick: move |_| state.write().set_active_tab(AppTab::FaceLibrary),
                }
                TabButton {
                    label: "识别日志",
                    active: matches!(active_tab, AppTab::RecognitionLog),
                    onclick: move |_| state.write().set_active_tab(AppTab::RecognitionLog),
                }
            }
            UserInfoModal {
                visible: show_add_user_modal,
                user_id: user_draft.user_id.clone(),
                user_info: user_draft.user_info.clone(),
                image_base64: user_draft.image_base64.clone(),
                on_close: move |_| state.write().close_add_user_modal(),
                on_user_id_input: move |value: String| state.write().update_user_draft_id(value),
                on_user_info_input: move |value: String| state.write().update_user_draft_info(value),
                on_image_input: move |value: String| state.write().update_user_draft_image(value),
                on_error: move |message: String| state.write().set_error(message),
                on_submit: move |_| {
                    let controller = add_user_controller.clone();
                    let mut state = state;
                    async move {
                        let Some(new_user) = ({
                            let current_state = state.read().clone();
                            current_state.user_draft.as_new_user()
                        }) else {
                            state.write().set_error("请完整填写用户 ID、用户信息和 Base64 图片");
                            return;
                        };

                        let mut next_state = state.read().clone();
                        if controller.add_user(&mut next_state, new_user).await.is_ok() {
                            next_state.close_add_user_modal();
                        }
                        state.set(next_state);
                    }
                },
            }
            ImageViewer {
                image_base64: selected_log_image,
                on_close: move |_| state.write().clear_selected_log_image(),
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let class_name = if active { "tab-button active" } else { "tab-button" };

    rsx! {
        button {
            class: "{class_name}",
            onclick: move |event| onclick.call(event),
            "{label}"
        }
    }
}
