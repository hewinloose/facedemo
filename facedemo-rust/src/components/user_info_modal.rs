use dioxus::prelude::*;

#[cfg(feature = "tauri-backend")]
use crate::services::tauri_bridge::invoke_no_args;

#[cfg(feature = "tauri-backend")]
#[component]
fn ImageInputControl(
    image_base64: String,
    on_image_input: EventHandler<String>,
    on_error: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "action-row",
            button {
                class: "ghost-button",
                onclick: move |_| {
                    let on_image_input = on_image_input.clone();
                    let on_error = on_error.clone();
                    spawn(async move {
                        match invoke_no_args::<String>("pick_image").await {
                            Ok(base64) => on_image_input.call(base64),
                            Err(error) => on_error.call(error),
                        }
                    });
                },
                "选择图片"
            }
            button {
                class: "ghost-button",
                onclick: move |_| {
                    let on_image_input = on_image_input.clone();
                    let on_error = on_error.clone();
                    spawn(async move {
                        match invoke_no_args::<String>("take_photo").await {
                            Ok(base64) => on_image_input.call(base64),
                            Err(error) => on_error.call(error),
                        }
                    });
                },
                "拍照"
            }
        }
        if !image_base64.is_empty() {
            div { class: "panel",
                img {
                    class: "preview-image",
                    src: "data:image/png;base64,{image_base64}",
                    alt: "用户图片预览",
                }
            }
        }
        p { class: "hint", "Tauri 模式下可直接选择图片；拍照接口当前仅预留。" }
    }
}

#[cfg(not(feature = "tauri-backend"))]
#[component]
fn ImageInputControl(
    image_base64: String,
    on_image_input: EventHandler<String>,
    on_error: EventHandler<String>,
) -> Element {
    let _ = on_error;

    rsx! {
        textarea {
            rows: "5",
            placeholder: "粘贴 Base64 图片数据",
            value: "{image_base64}",
            oninput: move |event| on_image_input.call(event.value()),
        }
        p { class: "hint", "当前为 Web 调试降级模式，请继续用 Base64 文本模拟上传流程。" }
    }
}

#[component]
pub fn UserInfoModal(
    visible: bool,
    user_id: String,
    user_info: String,
    image_base64: String,
    on_close: EventHandler<()>,
    on_user_id_input: EventHandler<String>,
    on_user_info_input: EventHandler<String>,
    on_image_input: EventHandler<String>,
    on_error: EventHandler<String>,
    on_submit: EventHandler<()>,
) -> Element {
    rsx! {
        if visible {
            div { class: "overlay",
                section { class: "panel modal-panel",
                    div { class: "section-header",
                        h2 { "添加用户" }
                        button {
                            class: "ghost-button",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }
                    }
                    div { class: "form-grid",
                        label {
                            span { "用户 ID" }
                            input {
                                r#type: "text",
                                placeholder: "alice",
                                value: "{user_id}",
                                oninput: move |event| on_user_id_input.call(event.value()),
                            }
                        }
                        label {
                            span { "用户信息" }
                            input {
                                r#type: "text",
                                placeholder: "前台",
                                value: "{user_info}",
                                oninput: move |event| on_user_info_input.call(event.value()),
                            }
                        }
                        label {
                            span { "用户图片" }
                            ImageInputControl {
                                image_base64: image_base64.clone(),
                                on_image_input: on_image_input.clone(),
                                on_error: on_error.clone(),
                            }
                        }
                    }
                    div { class: "action-row",
                        button {
                            class: "primary-button",
                            onclick: move |_| on_submit.call(()),
                            "提交"
                        }
                    }
                }
            }
        }
    }
}
