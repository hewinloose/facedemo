use dioxus::prelude::*;
use face_core::models::FaceUserSummary;

#[component]
pub fn FaceLibraryPage(
    users: Vec<FaceUserSummary>,
    on_refresh: EventHandler<MouseEvent>,
    on_open_add: EventHandler<MouseEvent>,
    on_delete: EventHandler<String>,
) -> Element {
    rsx! {
        section { class: "page",
            div { class: "section-header",
                div { class: "page-title",
                    h2 { "人脸库" }
                    p { "替代原 Ionic Tab1，当前已具备刷新、添加和删除的最小交互壳。" }
                }
                div { class: "action-row",
                    button { class: "ghost-button", onclick: move |event| on_refresh.call(event), "刷新" }
                    button { class: "primary-button", onclick: move |event| on_open_add.call(event), "添加用户" }
                }
            }
            div { class: "panel",
                if users.is_empty() {
                    p { class: "empty-state", "当前还没有已加载用户。" }
                } else {
                    ul { class: "list",
                        for user in users {
                            li { key: "{user.user_id}", class: "list-item",
                                div { class: "list-item-top",
                                    strong { "({user.user_id})" }
                                    button {
                                        class: "ghost-button danger",
                                        onclick: {
                                            let user_id = user.user_id.clone();
                                            move |_| on_delete.call(user_id.clone())
                                        },
                                        "删除"
                                    }
                                }
                                span { "{user.user_info}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
