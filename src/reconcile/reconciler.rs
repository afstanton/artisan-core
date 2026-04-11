use crate::{
    domain::{CitationRecord, Entity, EntityType, PublisherRecord, SourceRecord},
    id::{CanonicalId, ExternalId},
    provenance::Provenance,
};

use super::{
    conflicts::ConflictSet,
    matcher::{MatchCandidate, MatchQuery},
    policy::ReconciliationPolicy,
    store::{CanonicalSubject, ReconciliationStore, SubjectKind},
};

#[derive(Debug, Clone)]
pub struct SourceHint {
    pub title: Option<String>,
    pub publisher: Option<String>,
    pub game_system: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportCandidate<T> {
    pub payload: T,
    pub external_ids: Vec<ExternalId>,
    pub display_name: Option<String>,
    pub source_hints: Vec<SourceHint>,
    pub provenance: Option<Provenance>,
}

#[derive(Debug, Clone)]
pub enum ResolutionOutcome {
    Matched {
        id: CanonicalId,
        confidence: f32,
    },
    Created {
        id: CanonicalId,
    },
    Ambiguous {
        candidates: Vec<MatchCandidate>,
    },
    Conflict {
        existing: CanonicalId,
        details: ConflictSet,
    },
}

pub struct Reconciler<S: ReconciliationStore> {
    pub store: S,
    pub policy: ReconciliationPolicy,
}

impl<S: ReconciliationStore> Reconciler<S> {
    pub fn reconcile_publishers(
        &mut self,
        candidates: Vec<ImportCandidate<PublisherRecord>>,
    ) -> Vec<ResolutionOutcome> {
        candidates
            .into_iter()
            .map(|candidate| {
                self.resolve_candidate(
                    SubjectKind::Publisher,
                    candidate.external_ids,
                    CanonicalSubject::Publisher(candidate.payload),
                    MatchQuery {
                        display_name: candidate.display_name,
                        kind_hint: None,
                        source_hint: None,
                        game_system_hint: None,
                    },
                )
            })
            .collect()
    }

    pub fn reconcile_sources(
        &mut self,
        candidates: Vec<ImportCandidate<SourceRecord>>,
    ) -> Vec<ResolutionOutcome> {
        candidates
            .into_iter()
            .map(|candidate| {
                let source_hint = candidate
                    .source_hints
                    .iter()
                    .find_map(|hint| hint.title.clone());
                let game_system_hint = candidate
                    .source_hints
                    .iter()
                    .find_map(|hint| hint.game_system.clone())
                    .or_else(|| candidate.payload.game_systems.first().cloned());

                self.resolve_candidate(
                    SubjectKind::Source,
                    candidate.external_ids,
                    CanonicalSubject::Source(candidate.payload),
                    MatchQuery {
                        display_name: candidate.display_name,
                        kind_hint: None,
                        source_hint,
                        game_system_hint,
                    },
                )
            })
            .collect()
    }

    pub fn reconcile_entity_types(
        &mut self,
        candidates: Vec<ImportCandidate<EntityType>>,
    ) -> Vec<ResolutionOutcome> {
        candidates
            .into_iter()
            .map(|candidate| {
                self.resolve_candidate(
                    SubjectKind::EntityType,
                    candidate.external_ids,
                    CanonicalSubject::EntityType(candidate.payload.clone()),
                    MatchQuery {
                        display_name: candidate.display_name,
                        kind_hint: Some(candidate.payload.key),
                        source_hint: None,
                        game_system_hint: None,
                    },
                )
            })
            .collect()
    }

    pub fn reconcile_entities(
        &mut self,
        candidates: Vec<ImportCandidate<Entity>>,
    ) -> Vec<ResolutionOutcome> {
        candidates
            .into_iter()
            .map(|candidate| {
                let kind_hint = candidate
                    .payload
                    .attributes
                    .get("pcgen_entity_type_key")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                let source_hint = candidate
                    .source_hints
                    .iter()
                    .find_map(|hint| hint.title.clone());
                let game_system_hint = candidate
                    .source_hints
                    .iter()
                    .find_map(|hint| hint.game_system.clone());

                self.resolve_candidate(
                    SubjectKind::Entity,
                    candidate.external_ids,
                    CanonicalSubject::Entity(candidate.payload),
                    MatchQuery {
                        display_name: candidate.display_name,
                        kind_hint,
                        source_hint,
                        game_system_hint,
                    },
                )
            })
            .collect()
    }

    pub fn reconcile_citations(
        &mut self,
        candidates: Vec<ImportCandidate<CitationRecord>>,
    ) -> Vec<ResolutionOutcome> {
        candidates
            .into_iter()
            .map(|candidate| {
                self.resolve_candidate(
                    SubjectKind::Citation,
                    candidate.external_ids,
                    CanonicalSubject::Citation(candidate.payload),
                    MatchQuery {
                        display_name: candidate.display_name,
                        kind_hint: None,
                        source_hint: None,
                        game_system_hint: None,
                    },
                )
            })
            .collect()
    }

    fn resolve_candidate(
        &mut self,
        kind: SubjectKind,
        external_ids: Vec<ExternalId>,
        subject: CanonicalSubject,
        match_query: MatchQuery,
    ) -> ResolutionOutcome {
        for external_id in &external_ids {
            if let Some(existing) = self.store.find_by_external_id(kind, external_id) {
                return ResolutionOutcome::Matched {
                    id: existing,
                    confidence: 1.0,
                };
            }
        }

        let match_candidates = self.store.search_candidates(kind, match_query);

        if match_candidates.len() > 1 {
            return ResolutionOutcome::Ambiguous {
                candidates: match_candidates,
            };
        }

        if let Some(best) = match_candidates.first() {
            if best.confidence >= 0.95 {
                for external_id in external_ids {
                    self.store.link_external_id(kind, best.id, external_id);
                }
                return ResolutionOutcome::Matched {
                    id: best.id,
                    confidence: best.confidence,
                };
            }
        }

        let created = self.store.upsert_subject(subject);
        ResolutionOutcome::Created { id: created }
    }
}
