use serde::{Deserialize, Serialize};

use crate::id::CanonicalId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchQuery {
    pub display_name: Option<String>,
    pub kind_hint: Option<String>,
    pub source_hint: Option<String>,
    pub game_system_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCandidate {
    pub id: CanonicalId,
    pub confidence: f32,
    pub reason: String,
}
