use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReconciliationPolicy {
    Strict,
    Guided,
    Permissive,
}
