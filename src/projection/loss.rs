use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossNote {
    pub code: String,
    pub message: String,
}
