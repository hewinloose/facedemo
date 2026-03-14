use crate::models::RecognitionLogEntry;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebsocketMessageError {
    #[error("websocket payload parsing failed: {0}")]
    InvalidPayload(String),
}

pub fn parse_log_entries(payload: &str) -> Result<Vec<RecognitionLogEntry>, WebsocketMessageError> {
    serde_json::from_str(payload).map_err(|error| WebsocketMessageError::InvalidPayload(error.to_string()))
}
