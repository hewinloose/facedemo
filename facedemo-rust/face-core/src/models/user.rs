use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaceUserSummary {
    pub user_id: String,
    pub user_info: String,
}
