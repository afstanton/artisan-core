use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    id::{CanonicalId, ExternalId},
    provenance::Provenance,
};

use super::rules::{Effect, Prerequisite, RuleHook};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletenessState {
    Complete,
    Descriptive,
    Inferred,
    Stub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: CanonicalId,
    pub entity_type: CanonicalId,
    pub name: String,
    pub attributes: IndexMap<String, Value>,
    pub effects: Vec<Effect>,
    pub prerequisites: Vec<Prerequisite>,
    pub rule_hooks: Vec<RuleHook>,
    pub citations: Vec<CanonicalId>,
    pub external_ids: Vec<ExternalId>,
    pub completeness: CompletenessState,
    pub provenance: Option<Provenance>,
}
