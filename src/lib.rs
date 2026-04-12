pub mod diagnostics;
pub mod domain;
pub mod format;
pub mod id;
pub mod projection;
pub mod provenance;
pub mod reconcile;

pub use domain::{
    CharacterGraph, CitationRecord, CoreCatalog, Entity, EntityType, FieldCardinality, FieldDef,
    FieldType, IdentityLink, MappingRecord, ParsedCatalog, PublisherRecord,
    RelationshipCardinality, RelationshipDef, SourceRecord,
};
pub use format::{CatalogParser, CatalogUnparser};
pub use id::{CanonicalId, ExternalId, FormatId};
pub use projection::{LossNote, ProjectionMap};
pub use reconcile::{
    ImportCandidate, InMemoryReconciliationStore, Reconciler, ReconciliationPolicy,
    ResolutionOutcome,
};
