use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::id::CanonicalId;

use super::loss::LossNote;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalProjection {
    pub target_format: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionMap {
    pub canonical_to_external: IndexMap<CanonicalId, ExternalProjection>,
    pub unresolved: Vec<CanonicalId>,
    pub lossy_notes: Vec<LossNote>,
}
