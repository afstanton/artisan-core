pub mod conflicts;
pub mod memory_store;
pub mod matcher;
pub mod policy;
pub mod reconciler;
pub mod scoring;
pub mod store;

pub use conflicts::{AmbiguitySet, ConflictSet};
pub use memory_store::InMemoryReconciliationStore;
pub use matcher::{MatchCandidate, MatchQuery};
pub use policy::ReconciliationPolicy;
pub use reconciler::{ImportCandidate, Reconciler, ResolutionOutcome, SourceHint};
pub use store::{CanonicalSubject, ReconciliationStore, SubjectKind};
