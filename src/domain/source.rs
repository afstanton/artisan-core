use serde::{Deserialize, Serialize};

use crate::id::{CanonicalId, ExternalId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: CanonicalId,
    pub title: String,
    // Legacy compatibility field; prefer `publisher_ids` for normalized relations.
    pub publisher: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub publisher_ids: Vec<CanonicalId>,
    pub edition: Option<String>,
    pub license: Option<String>,
    pub game_systems: Vec<String>,
    pub external_ids: Vec<ExternalId>,
}
