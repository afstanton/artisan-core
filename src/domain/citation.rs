use serde::{Deserialize, Serialize};

use crate::id::{CanonicalId, ExternalId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectRef {
    Entity(CanonicalId),
    EntityType(CanonicalId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationState {
    Unverified,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationLocator {
    pub kind: String,
    pub value: String,
    pub canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRecord {
    pub id: CanonicalId,
    pub subject: SubjectRef,
    pub source: CanonicalId,
    pub locators: Vec<CitationLocator>,
    pub verification: VerificationState,
    pub external_ids: Vec<ExternalId>,
}
