use serde::{Deserialize, Serialize};

use crate::id::{CanonicalId, ExternalId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherRecord {
    pub id: CanonicalId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_ids: Vec<ExternalId>,
}
