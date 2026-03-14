use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecognitionLogEntry {
    pub result: bool,
    pub user_info: String,
    pub date: String,
    pub image: String,
}
