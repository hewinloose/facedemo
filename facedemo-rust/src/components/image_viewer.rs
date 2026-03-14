use dioxus::prelude::*;

#[component]
pub fn ImageViewer(image_base64: Option<String>, on_close: EventHandler<()>) -> Element {
    let source = image_base64.map(|image_base64| format!("data:image/png;base64,{image_base64}"));

    rsx! {
        if let Some(source) = source {
            div { class: "overlay",
                section { class: "panel viewer-panel",
                    div { class: "section-header",
                        h2 { "图片查看器" }
                        button {
                            class: "ghost-button",
                            onclick: move |_| on_close.call(()),
                            "关闭"
                        }
                    }
                    img { class: "preview-image", src: "{source}" }
                }
            }
        }
    }
}
