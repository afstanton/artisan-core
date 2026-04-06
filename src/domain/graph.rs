use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{id::CanonicalId, provenance::OpaqueExtension};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: CanonicalId,
    pub label: String,
    pub data: IndexMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: CanonicalId,
    pub to: CanonicalId,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterGraph {
    pub nodes: IndexMap<CanonicalId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
    pub opaque_extensions: Vec<OpaqueExtension>,
}
