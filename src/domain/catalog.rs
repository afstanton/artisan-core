use serde::{Deserialize, Serialize};

use crate::id::{CanonicalId, ExternalId};
use crate::projection::{LossNote, ProjectionMap};

use super::{CharacterGraph, CitationRecord, Entity, EntityType, PublisherRecord, SourceRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityLink {
    pub kind: String,
    pub canonical_id: CanonicalId,
    pub external_id: ExternalId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingRecord {
    pub id: String,
    pub description: Option<String>,
    pub source_entity_type: Option<CanonicalId>,
    pub target_entity_type: Option<CanonicalId>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreCatalog {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub publishers: Vec<PublisherRecord>,
    pub sources: Vec<SourceRecord>,
    pub citations: Vec<CitationRecord>,
    pub entity_types: Vec<EntityType>,
    pub entities: Vec<Entity>,
    pub character_graphs: Vec<CharacterGraph>,
    pub identity_links: Vec<IdentityLink>,
    pub mapping_records: Vec<MappingRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projection_maps: Vec<ProjectionMap>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loss_notes: Vec<LossNote>,
}
