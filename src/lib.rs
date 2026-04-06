pub mod diagnostics;
pub mod domain;
pub mod id;
pub mod projection;
pub mod provenance;
pub mod reconcile;

pub use domain::{
	CharacterGraph, CitationRecord, CoreCatalog, Entity, EntityType, IdentityLink, MappingRecord,
	SourceRecord,
};
pub use id::{CanonicalId, ExternalId, FormatId};
pub use projection::{LossNote, ProjectionMap};
pub use reconcile::{
	ImportCandidate, InMemoryReconciliationStore, Reconciler, ReconciliationPolicy,
	ResolutionOutcome,
};
