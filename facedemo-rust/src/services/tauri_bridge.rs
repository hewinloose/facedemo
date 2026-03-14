use js_sys::Function;
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::from_value;
use tauri_wasm::{args, invoke};
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};

#[derive(serde::Deserialize)]
struct EventEnvelope<T> {
    payload: T,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn Fn(JsValue)>) -> Result<JsValue, JsValue>;
}

pub struct EventListener {
    _callback: Closure<dyn Fn(JsValue)>,
    detach: Function,
}

impl Drop for EventListener {
    fn drop(&mut self) {
        let _ = self.detach.call0(&JsValue::NULL);
    }
}

pub async fn invoke_unit(command: &str) -> Result<(), String> {
    invoke(command).await.map(|_| ()).map_err(|error| error.to_string())
}

pub async fn invoke_unit_with_args<A>(command: &str, payload: &A) -> Result<(), String>
where
    A: Serialize + ?Sized,
{
    let args = args(payload).map_err(|error| error.to_string())?;
    invoke(command)
        .with_args(args)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub async fn invoke_with_args<T, A>(command: &str, payload: &A) -> Result<T, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let args = args(payload).map_err(|error| error.to_string())?;
    let value = invoke(command)
        .with_args(args)
        .await
        .map_err(|error| error.to_string())?;

    from_value(value).map_err(|error| error.to_string())
}

pub async fn invoke_no_args<T>(command: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let value = invoke(command).await.map_err(|error| error.to_string())?;
    from_value(value).map_err(|error| error.to_string())
}

pub async fn listen_to_event<T, F>(event: &'static str, handler: F) -> Result<EventListener, String>
where
    T: DeserializeOwned + 'static,
    F: Fn(T) + 'static,
{
    let callback = Closure::new(move |value: JsValue| {
        if let Ok(envelope) = from_value::<EventEnvelope<T>>(value) {
            handler(envelope.payload);
        }
    });

    let detach = listen(event, &callback)
        .await
        .map_err(js_error_to_string)?
        .dyn_into::<Function>()
        .map_err(|_| "tauri event listener detach handle is not a function".to_string())?;

    Ok(EventListener {
        _callback: callback,
        detach,
    })
}

fn js_error_to_string(error: JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "unknown tauri frontend error".to_string())
}
