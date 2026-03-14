use dioxus::prelude::*;
use face_core::models::RecognitionLogEntry;

#[cfg(feature = "tauri-backend")]
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "tauri-backend")]
use crate::services::tauri_bridge::{EventListener, listen_to_event};

#[component]
pub fn RecognitionLogPage(
    logs: Vec<RecognitionLogEntry>,
    on_refresh: EventHandler<MouseEvent>,
    on_select_image: EventHandler<String>,
    on_new_logs: EventHandler<Vec<RecognitionLogEntry>>,
) -> Element {
    #[cfg(feature = "tauri-backend")]
    {
        let listener_slot = use_hook(|| Rc::new(RefCell::new(None::<EventListener>)));
        let listener_slot = listener_slot.clone();
        let on_new_logs = on_new_logs.clone();

        use_effect(move || {
            if listener_slot.borrow().is_some() {
                return;
            }

            let listener_slot = listener_slot.clone();
            let on_new_logs = on_new_logs.clone();
            spawn(async move {
                if let Ok(listener) = listen_to_event::<Vec<RecognitionLogEntry>, _>(
                    "recognition-logs",
                    move |logs| on_new_logs.call(logs),
                )
                .await
                {
                    *listener_slot.borrow_mut() = Some(listener);
                }
            });
        });
    }

    rsx! {
        section { class: "page",
            div { class: "section-header",
                div { class: "page-title",
                    h2 { "识别日志" }
                    p { "替代原 Ionic Tab2，当前保留了刷新和图片查看交互。" }
                }
                div { class: "action-row",
                    button { class: "ghost-button", onclick: move |event| on_refresh.call(event), "重新连接" }
                }
            }
            div { class: "panel",
                if logs.is_empty() {
                    p { class: "empty-state", "当前还没有识别日志。" }
                } else {
                    ul { class: "list",
                        for log in logs {
                            li { key: "{log.date}-{log.user_info}", class: "list-item",
                                div { class: "list-item-top",
                                    strong {
                                        if log.result { "识别成功" } else { "识别失败" }
                                    }
                                    button {
                                        class: "ghost-button",
                                        onclick: {
                                            let image = log.image.clone();
                                            move |_| on_select_image.call(image.clone())
                                        },
                                        "查看图片"
                                    }
                                }
                                span { "{log.user_info}" }
                                span { class: "meta", "{log.date}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
