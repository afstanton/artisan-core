use std::collections::HashMap;

use crate::{
    domain::{CitationRecord, CoreCatalog, Entity, SourceRecord},
    id::{CanonicalId, ExternalId, FormatId},
};

use super::{
    matcher::{MatchCandidate, MatchQuery},
    scoring::score_candidates,
    store::{CanonicalSubject, ReconciliationStore, SubjectKind},
};

#[derive(Debug, Clone)]
pub struct InMemoryReconciliationStore {
    catalog: CoreCatalog,
    external_index: HashMap<(SubjectKind, String), CanonicalId>,
}

impl InMemoryReconciliationStore {
    pub fn new(catalog: CoreCatalog) -> Self {
        let mut store = Self {
            catalog,
            external_index: HashMap::new(),
        };
        store.rebuild_external_index();
        store
    }

    pub fn catalog(&self) -> &CoreCatalog {
        &self.catalog
    }

    pub fn into_catalog(self) -> CoreCatalog {
        self.catalog
    }

    fn rebuild_external_index(&mut self) {
        self.external_index.clear();

        for publisher in &self.catalog.publishers {
            for ext in &publisher.external_ids {
                self.external_index
                    .insert((SubjectKind::Publisher, external_id_key(ext)), publisher.id);
            }
        }

        for source in &self.catalog.sources {
            for ext in &source.external_ids {
                self.external_index
                    .insert((SubjectKind::Source, external_id_key(ext)), source.id);
            }
        }

        for entity_type in &self.catalog.entity_types {
            for ext in &entity_type.external_ids {
                self.external_index.insert(
                    (SubjectKind::EntityType, external_id_key(ext)),
                    entity_type.id,
                );
            }
        }

        for entity in &self.catalog.entities {
            for ext in &entity.external_ids {
                self.external_index
                    .insert((SubjectKind::Entity, external_id_key(ext)), entity.id);
            }
        }

        for citation in &self.catalog.citations {
            for ext in &citation.external_ids {
                self.external_index
                    .insert((SubjectKind::Citation, external_id_key(ext)), citation.id);
            }
        }
    }

    fn search_entity_candidates(&self, query: &MatchQuery) -> Vec<MatchCandidate> {
        let source_by_id: HashMap<CanonicalId, &SourceRecord> =
            self.catalog.sources.iter().map(|s| (s.id, s)).collect();
        let citation_by_id: HashMap<CanonicalId, &CitationRecord> =
            self.catalog.citations.iter().map(|c| (c.id, c)).collect();
        let entity_type_key_by_id: HashMap<CanonicalId, &str> = self
            .catalog
            .entity_types
            .iter()
            .map(|ty| (ty.id, ty.key.as_str()))
            .collect();

        let mut candidates = Vec::new();
        for entity in &self.catalog.entities {
            let mut confidence = 0.0f32;
            let mut reasons = Vec::new();

            if let Some(display_name) = &query.display_name {
                let name_score = score_name_similarity(display_name, &entity.name);
                if name_score == 0.0 {
                    continue;
                }
                confidence += name_score;
                reasons.push("name similarity".to_string());
            } else {
                confidence += 0.40;
            }

            if let Some(kind_hint) = &query.kind_hint {
                if let Some(existing_key) = entity_type_key_by_id.get(&entity.entity_type) {
                    if existing_key.eq_ignore_ascii_case(kind_hint) {
                        confidence += 0.20;
                        reasons.push("entity type key match".to_string());
                    } else {
                        continue;
                    }
                }
            }

            if let Some(source_hint) = &query.source_hint {
                let source_titles = entity_source_titles(entity, &citation_by_id, &source_by_id);
                let known_sources = !source_titles.is_empty();
                let source_match = source_titles.iter().any(|s| fuzzy_match(source_hint, s));
                if known_sources && !source_match {
                    continue;
                }
                if source_match {
                    confidence += 0.10;
                    reasons.push(format!("source hint match ({source_hint})"));
                }
            }

            if let Some(game_system_hint) = &query.game_system_hint {
                let game_systems = entity_game_systems(entity, &citation_by_id, &source_by_id);
                let known_game_systems = !game_systems.is_empty();
                let game_system_match = game_systems
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(game_system_hint));
                if known_game_systems && !game_system_match {
                    continue;
                }
                if game_system_match {
                    confidence += 0.20;
                    reasons.push(format!("game system match ({game_system_hint})"));
                }
            }

            if confidence < 0.60 {
                continue;
            }

            candidates.push(MatchCandidate {
                id: entity.id,
                confidence: confidence.clamp(0.0, 1.0),
                reason: reasons.join(", "),
            });
        }

        score_candidates(query, &candidates)
    }

    fn search_entity_type_candidates(&self, query: &MatchQuery) -> Vec<MatchCandidate> {
        let mut candidates = Vec::new();

        for entity_type in &self.catalog.entity_types {
            let mut confidence = 0.0f32;
            let mut reasons = Vec::new();

            if let Some(display_name) = &query.display_name {
                let name_score = score_name_similarity(display_name, &entity_type.name);
                if name_score == 0.0 {
                    continue;
                }
                confidence += name_score;
                reasons.push("name similarity".to_string());
            } else {
                confidence += 0.40;
            }

            if let Some(kind_hint) = &query.kind_hint {
                if entity_type.key.eq_ignore_ascii_case(kind_hint) {
                    confidence += 0.25;
                    reasons.push("type key match".to_string());
                } else {
                    continue;
                }
            }

            if confidence < 0.60 {
                continue;
            }

            candidates.push(MatchCandidate {
                id: entity_type.id,
                confidence: confidence.clamp(0.0, 1.0),
                reason: reasons.join(", "),
            });
        }

        score_candidates(query, &candidates)
    }

    fn search_source_candidates(&self, query: &MatchQuery) -> Vec<MatchCandidate> {
        let mut candidates = Vec::new();

        for source in &self.catalog.sources {
            let mut confidence = 0.0f32;
            let mut reasons = Vec::new();

            if let Some(display_name) = &query.display_name {
                let name_score = score_name_similarity(display_name, &source.title);
                if name_score == 0.0 {
                    continue;
                }
                confidence += name_score;
                reasons.push("title similarity".to_string());
            } else {
                confidence += 0.40;
            }

            if let Some(source_hint) = &query.source_hint {
                if fuzzy_match(source_hint, &source.title) {
                    confidence += 0.15;
                    reasons.push("source hint match".to_string());
                } else {
                    continue;
                }
            }

            if let Some(game_system_hint) = &query.game_system_hint {
                if source
                    .game_systems
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(game_system_hint))
                {
                    confidence += 0.20;
                    reasons.push("game system match".to_string());
                } else if !source.game_systems.is_empty() {
                    continue;
                }
            }

            if confidence < 0.60 {
                continue;
            }

            candidates.push(MatchCandidate {
                id: source.id,
                confidence: confidence.clamp(0.0, 1.0),
                reason: reasons.join(", "),
            });
        }

        score_candidates(query, &candidates)
    }

    fn search_publisher_candidates(&self, query: &MatchQuery) -> Vec<MatchCandidate> {
        let mut candidates = Vec::new();

        for publisher in &self.catalog.publishers {
            let mut confidence = 0.0f32;
            let mut reasons = Vec::new();

            if let Some(display_name) = &query.display_name {
                let name_score = score_name_similarity(display_name, &publisher.name);
                if name_score == 0.0 {
                    continue;
                }
                confidence += name_score;
                reasons.push("name similarity".to_string());
            } else {
                confidence += 0.40;
            }

            if confidence < 0.60 {
                continue;
            }

            candidates.push(MatchCandidate {
                id: publisher.id,
                confidence: confidence.clamp(0.0, 1.0),
                reason: reasons.join(", "),
            });
        }

        score_candidates(query, &candidates)
    }
}

impl ReconciliationStore for InMemoryReconciliationStore {
    fn find_by_external_id(&self, kind: SubjectKind, id: &ExternalId) -> Option<CanonicalId> {
        self.external_index
            .get(&(kind, external_id_key(id)))
            .copied()
    }

    fn search_candidates(&self, kind: SubjectKind, query: MatchQuery) -> Vec<MatchCandidate> {
        match kind {
            SubjectKind::Entity => self.search_entity_candidates(&query),
            SubjectKind::EntityType => self.search_entity_type_candidates(&query),
            SubjectKind::Publisher => self.search_publisher_candidates(&query),
            SubjectKind::Source => self.search_source_candidates(&query),
            SubjectKind::Citation => Vec::new(),
        }
    }

    fn upsert_subject(&mut self, subject: CanonicalSubject) -> CanonicalId {
        let id = match subject {
            CanonicalSubject::Entity(entity) => {
                let id = entity.id;
                if let Some(existing) = self.catalog.entities.iter_mut().find(|e| e.id == id) {
                    *existing = entity;
                } else {
                    self.catalog.entities.push(entity);
                }
                id
            }
            CanonicalSubject::EntityType(entity_type) => {
                let id = entity_type.id;
                if let Some(existing) = self.catalog.entity_types.iter_mut().find(|e| e.id == id) {
                    *existing = entity_type;
                } else {
                    self.catalog.entity_types.push(entity_type);
                }
                id
            }
            CanonicalSubject::Publisher(publisher) => {
                let id = publisher.id;
                if let Some(existing) = self.catalog.publishers.iter_mut().find(|e| e.id == id) {
                    *existing = publisher;
                } else {
                    self.catalog.publishers.push(publisher);
                }
                id
            }
            CanonicalSubject::Source(source) => {
                let id = source.id;
                if let Some(existing) = self.catalog.sources.iter_mut().find(|e| e.id == id) {
                    *existing = source;
                } else {
                    self.catalog.sources.push(source);
                }
                id
            }
            CanonicalSubject::Citation(citation) => {
                let id = citation.id;
                if let Some(existing) = self.catalog.citations.iter_mut().find(|e| e.id == id) {
                    *existing = citation;
                } else {
                    self.catalog.citations.push(citation);
                }
                id
            }
        };

        self.rebuild_external_index();
        id
    }

    fn link_external_id(&mut self, kind: SubjectKind, canonical: CanonicalId, id: ExternalId) {
        match kind {
            SubjectKind::Entity => {
                if let Some(entity) = self.catalog.entities.iter_mut().find(|e| e.id == canonical)
                    && !entity.external_ids.contains(&id)
                {
                    entity.external_ids.push(id);
                }
            }
            SubjectKind::EntityType => {
                if let Some(entity_type) = self
                    .catalog
                    .entity_types
                    .iter_mut()
                    .find(|e| e.id == canonical)
                    && !entity_type.external_ids.contains(&id)
                {
                    entity_type.external_ids.push(id);
                }
            }
            SubjectKind::Publisher => {
                if let Some(publisher) = self
                    .catalog
                    .publishers
                    .iter_mut()
                    .find(|e| e.id == canonical)
                    && !publisher.external_ids.contains(&id)
                {
                    publisher.external_ids.push(id);
                }
            }
            SubjectKind::Source => {
                if let Some(source) = self.catalog.sources.iter_mut().find(|e| e.id == canonical)
                    && !source.external_ids.contains(&id)
                {
                    source.external_ids.push(id);
                }
            }
            SubjectKind::Citation => {
                if let Some(citation) = self
                    .catalog
                    .citations
                    .iter_mut()
                    .find(|e| e.id == canonical)
                    && !citation.external_ids.contains(&id)
                {
                    citation.external_ids.push(id);
                }
            }
        }

        self.rebuild_external_index();
    }
}

fn external_id_key(id: &ExternalId) -> String {
    let namespace = id.namespace.as_deref().unwrap_or("").to_ascii_lowercase();
    format!(
        "{}|{}|{}",
        format_id_key(&id.format),
        namespace,
        id.value.to_ascii_lowercase()
    )
}

fn format_id_key(id: &FormatId) -> String {
    match id {
        FormatId::ArtisanToml => "artisan_toml".to_string(),
        FormatId::Pcgen => "pcgen".to_string(),
        FormatId::Herolab => "herolab".to_string(),
        FormatId::Hlo => "hlo".to_string(),
        FormatId::Foundry => "foundry".to_string(),
        FormatId::Other(value) => format!("other:{}", value.to_ascii_lowercase()),
    }
}

fn score_name_similarity(left: &str, right: &str) -> f32 {
    if left.eq_ignore_ascii_case(right) {
        return 0.75;
    }

    let left_norm = normalize_for_match(left);
    let right_norm = normalize_for_match(right);
    if left_norm.is_empty() || right_norm.is_empty() {
        return 0.0;
    }

    if left_norm == right_norm {
        return 0.65;
    }

    if left_norm.contains(&right_norm) || right_norm.contains(&left_norm) {
        return 0.40;
    }

    0.0
}

fn fuzzy_match(left: &str, right: &str) -> bool {
    let left_norm = normalize_for_match(left);
    let right_norm = normalize_for_match(right);
    if left_norm.is_empty() || right_norm.is_empty() {
        return false;
    }

    left_norm == right_norm || left_norm.contains(&right_norm) || right_norm.contains(&left_norm)
}

fn normalize_for_match(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn entity_source_titles(
    entity: &Entity,
    citation_by_id: &HashMap<CanonicalId, &CitationRecord>,
    source_by_id: &HashMap<CanonicalId, &SourceRecord>,
) -> Vec<String> {
    let mut titles = Vec::new();

    if let Some(title) = entity
        .attributes
        .get("pcgen_source_long")
        .and_then(|v| v.as_str())
    {
        titles.push(title.to_string());
    }
    if let Some(title) = entity
        .attributes
        .get("pcgen_source_short")
        .and_then(|v| v.as_str())
    {
        titles.push(title.to_string());
    }
    if let Some(title) = entity
        .attributes
        .get("pcgen_source_page")
        .and_then(|v| v.as_str())
    {
        titles.push(title.to_string());
    }

    for citation_id in &entity.citations {
        if let Some(citation) = citation_by_id.get(citation_id)
            && let Some(source) = source_by_id.get(&citation.source)
        {
            titles.push(source.title.clone());
        }
    }

    titles
}

fn entity_game_systems(
    entity: &Entity,
    citation_by_id: &HashMap<CanonicalId, &CitationRecord>,
    source_by_id: &HashMap<CanonicalId, &SourceRecord>,
) -> Vec<String> {
    let mut systems = Vec::new();

    if let Some(mode) = entity
        .attributes
        .get("pcgen_game_mode")
        .and_then(|v| v.as_str())
    {
        systems.push(mode.to_string());
    }

    for citation_id in &entity.citations {
        if let Some(citation) = citation_by_id.get(citation_id)
            && let Some(source) = source_by_id.get(&citation.source)
        {
            for system in &source.game_systems {
                systems.push(system.clone());
            }
        }
    }

    systems
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        domain::{CitationLocator, EntityType, RuleHook},
        reconcile::{
            ImportCandidate, Reconciler, ReconciliationPolicy, ResolutionOutcome, SourceHint,
        },
    };

    use super::*;

    #[test]
    fn reconcile_entities_prefers_game_system_match() {
        let entity_type = make_entity_type("pcgen:type:general", "General");
        let source_5e = make_source("Player's Handbook", &["5e"]);
        let source_pf2e = make_source("Core Rulebook", &["pf2e"]);

        let mut entity_5e = make_entity("Alertness", entity_type.id);
        let mut entity_pf2e = make_entity("Alertness", entity_type.id);

        let citation_5e = make_entity_citation(entity_5e.id, source_5e.id);
        let citation_pf2e = make_entity_citation(entity_pf2e.id, source_pf2e.id);
        entity_5e.citations.push(citation_5e.id);
        entity_pf2e.citations.push(citation_pf2e.id);

        let catalog = CoreCatalog {
            sources: vec![source_5e, source_pf2e],
            citations: vec![citation_5e, citation_pf2e],
            entity_types: vec![entity_type],
            entities: vec![entity_5e.clone(), entity_pf2e],
            ..CoreCatalog::default()
        };

        let mut reconciler = Reconciler {
            store: InMemoryReconciliationStore::new(catalog),
            policy: ReconciliationPolicy::Guided,
        };

        let candidate = ImportCandidate {
            payload: make_candidate_payload("Alertness"),
            external_ids: vec![ExternalId {
                format: FormatId::Pcgen,
                namespace: Some("lst".to_string()),
                value: "Alertness@77".to_string(),
            }],
            display_name: Some("Alertness".to_string()),
            source_hints: vec![SourceHint {
                title: None,
                publisher: None,
                game_system: Some("5e".to_string()),
            }],
            provenance: None,
        };

        let outcomes = reconciler.reconcile_entities(vec![candidate]);
        assert_eq!(outcomes.len(), 1);

        match &outcomes[0] {
            ResolutionOutcome::Matched { id, .. } => assert_eq!(*id, entity_5e.id),
            other => panic!("expected matched outcome, got {other:?}"),
        }
    }

    #[test]
    fn reconcile_entities_prefers_source_hint_match() {
        let entity_type = make_entity_type("pcgen:type:general", "General");
        let source_phb = make_source("Player's Handbook", &["5e"]);
        let source_xge = make_source("Xanathar's Guide to Everything", &["5e"]);

        let mut entity_phb = make_entity("Dodge", entity_type.id);
        let mut entity_xge = make_entity("Dodge", entity_type.id);

        let citation_phb = make_entity_citation(entity_phb.id, source_phb.id);
        let citation_xge = make_entity_citation(entity_xge.id, source_xge.id);
        entity_phb.citations.push(citation_phb.id);
        entity_xge.citations.push(citation_xge.id);

        let catalog = CoreCatalog {
            sources: vec![source_phb, source_xge],
            citations: vec![citation_phb, citation_xge],
            entity_types: vec![entity_type],
            entities: vec![entity_phb.clone(), entity_xge],
            ..CoreCatalog::default()
        };

        let mut reconciler = Reconciler {
            store: InMemoryReconciliationStore::new(catalog),
            policy: ReconciliationPolicy::Guided,
        };

        let candidate = ImportCandidate {
            payload: make_candidate_payload("Dodge"),
            external_ids: vec![ExternalId {
                format: FormatId::Pcgen,
                namespace: Some("lst".to_string()),
                value: "Dodge@5".to_string(),
            }],
            display_name: Some("Dodge".to_string()),
            source_hints: vec![SourceHint {
                title: Some("Player's Handbook".to_string()),
                publisher: None,
                game_system: None,
            }],
            provenance: None,
        };

        let outcomes = reconciler.reconcile_entities(vec![candidate]);
        assert_eq!(outcomes.len(), 1);

        match &outcomes[0] {
            ResolutionOutcome::Matched { id, .. } => assert_eq!(*id, entity_phb.id),
            other => panic!("expected matched outcome, got {other:?}"),
        }
    }

    fn make_entity_type(key: &str, name: &str) -> EntityType {
        EntityType {
            id: CanonicalId::new(),
            key: key.to_string(),
            name: name.to_string(),
            parent: None,
            fields: Vec::new(),
            relationships: Vec::new(),
            descriptive_fields: indexmap::IndexMap::new(),
            mechanical_fields: indexmap::IndexMap::new(),
            external_ids: Vec::new(),
            provenance: None,
        }
    }

    fn make_source(title: &str, game_systems: &[&str]) -> SourceRecord {
        SourceRecord {
            id: CanonicalId::new(),
            title: title.to_string(),
            publisher: None,
            publisher_ids: Vec::new(),
            edition: None,
            license: None,
            game_systems: game_systems.iter().map(|s| s.to_string()).collect(),
            external_ids: Vec::new(),
        }
    }

    fn make_entity(name: &str, entity_type: CanonicalId) -> Entity {
        Entity {
            id: CanonicalId::new(),
            entity_type,
            name: name.to_string(),
            attributes: indexmap::IndexMap::new(),
            effects: Vec::new(),
            prerequisites: Vec::new(),
            rule_hooks: Vec::<RuleHook>::new(),
            citations: Vec::new(),
            external_ids: Vec::new(),
            completeness: crate::domain::CompletenessState::Inferred,
            provenance: None,
        }
    }

    fn make_candidate_payload(name: &str) -> Entity {
        let mut payload = make_entity(name, CanonicalId::new());
        payload.attributes.insert(
            "pcgen_entity_type_key".to_string(),
            json!("pcgen:type:general"),
        );
        payload
    }

    fn make_entity_citation(entity_id: CanonicalId, source_id: CanonicalId) -> CitationRecord {
        CitationRecord {
            id: CanonicalId::new(),
            subject: crate::domain::SubjectRef::Entity(entity_id),
            source: source_id,
            locators: vec![CitationLocator {
                kind: "page".to_string(),
                value: "1".to_string(),
                canonical: false,
            }],
            verification: crate::domain::VerificationState::Unverified,
            external_ids: Vec::new(),
        }
    }
}
