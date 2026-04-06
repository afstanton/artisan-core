use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{id::{CanonicalId, ExternalId}, provenance::Provenance};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub id: CanonicalId,
    pub key: String,
    pub name: String,
    pub parent: Option<CanonicalId>,
    pub descriptive_fields: IndexMap<String, String>,
    pub mechanical_fields: IndexMap<String, String>,
    pub external_ids: Vec<ExternalId>,
    pub provenance: Option<Provenance>,
}
