use futures_util::{SinkExt, StreamExt};
use tauri::{AppHandle, Emitter, State};
use tokio_tungstenite::tungstenite::Message;

use crate::runtime::RECOGNITION_LOG_EVENT;
use crate::state::AppServices;

fn should_keep_existing_listener(task_finished: Option<bool>) -> bool {
    matches!(task_finished, Some(false))
}

#[cfg(test)]
mod tests {
    use super::should_keep_existing_listener;

    #[test]
    fn keeps_running_listener_and_restarts_finished_or_missing_listener() {
        assert!(should_keep_existing_listener(Some(false)));
        assert!(!should_keep_existing_listener(Some(true)));
        assert!(!should_keep_existing_listener(None));
    }
}

#[tauri::command]
pub async fn start_websocket_listener(
    app: AppHandle,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    let ws_url = services.config.ws_url.clone();
    let task_slot = services.websocket_task.clone();
    let mut guard = task_slot.lock().await;
    if should_keep_existing_listener(guard.as_ref().map(tokio::task::JoinHandle::is_finished)) {
        return Ok(());
    }
    *guard = None;

    let app_handle = app.clone();
    let task = tokio::spawn(async move {
        let connection = tokio_tungstenite::connect_async(&ws_url).await;
        let Ok((stream, _)) = connection else {
            tracing::error!("failed to connect websocket: {}", ws_url);
            return;
        };

        let (mut write, mut read) = stream.split();
        if write.send(Message::Text("s".into())).await.is_err() {
            tracing::warn!("failed to send initial websocket subscription frame");
        }

        while let Some(message_result) = read.next().await {
            let Ok(message) = message_result else {
                tracing::warn!("failed to read websocket message");
                continue;
            };

            match message {
                Message::Text(payload) => match face_core::websocket::parse_log_entries(&payload) {
                    Ok(logs) => {
                        if let Err(error) = app_handle.emit(RECOGNITION_LOG_EVENT, logs) {
                            tracing::warn!("failed to emit websocket event: {error}");
                        }
                    }
                    Err(error) => tracing::warn!("failed to parse websocket payload: {error}"),
                },
                Message::Binary(_) => tracing::debug!("binary websocket frame ignored"),
                _ => {}
            }
        }
    });

    *guard = Some(task);
    Ok(())
}
