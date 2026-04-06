use serde::{Deserialize, Serialize};

use crate::id::{CanonicalId, ExternalId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: CanonicalId,
    pub title: String,
    pub publisher: Option<String>,
    pub edition: Option<String>,
    pub license: Option<String>,
    pub game_systems: Vec<String>,
    pub external_ids: Vec<ExternalId>,
}
