use crate::{
    domain::{CitationRecord, Entity, EntityType, PublisherRecord, SourceRecord},
    id::{CanonicalId, ExternalId},
};

use super::matcher::{MatchCandidate, MatchQuery};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubjectKind {
    Entity,
    EntityType,
    Publisher,
    Source,
    Citation,
}

#[derive(Debug, Clone)]
pub enum CanonicalSubject {
    Entity(Entity),
    EntityType(EntityType),
    Publisher(PublisherRecord),
    Source(SourceRecord),
    Citation(CitationRecord),
}

pub trait ReconciliationStore {
    fn find_by_external_id(&self, kind: SubjectKind, id: &ExternalId) -> Option<CanonicalId>;
    fn search_candidates(&self, kind: SubjectKind, query: MatchQuery) -> Vec<MatchCandidate>;
    fn upsert_subject(&mut self, subject: CanonicalSubject) -> CanonicalId;
    fn link_external_id(&mut self, kind: SubjectKind, canonical: CanonicalId, id: ExternalId);
}
