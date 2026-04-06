use serde::{Deserialize, Serialize};

use crate::id::CanonicalId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSet {
    pub reason: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguitySet {
    pub candidates: Vec<CanonicalId>,
}
