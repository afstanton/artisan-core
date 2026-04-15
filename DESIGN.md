# artisan-core: Canonical Model and Reconciliation Architecture

## Overview

`artisan-core` is the canonical rules/content model used by all format adapters.

It is designed around one primary requirement:

- Parse from one ecosystem (PCGen, HeroLab, etc.), reconcile identities against stored canonical records, and emit a semantically equivalent representation for another ecosystem.

In practice, this means `artisan-core` is not just shared structs. It is the identity, provenance, and reconciliation engine that allows:

- PCGen -> core -> HeroLab
- HeroLab -> core -> PCGen
- Future N-format interoperability without pairwise translators

## Canonical Grain Rule

The canonical `EntityType` and `Entity` graph in `artisan-core` must exist at
the finest semantic grain present among all supported formats.

If the canonical layer is coarser than one of the imported ecosystems, we have
already lost information before reconciliation or projection starts. The
canonical model therefore needs to be maximally expressive and format-neutral,
not an average of the formats we currently support.

Implications:

- if one format distinguishes concepts that another format collapses,
  canonical records must preserve the finer distinction
- coarser formats project into collapsed views of canonical data
- finer formats project more directly from canonical data
- reverse projection from a coarse format into a finer one is a mapping and
  selection problem, not a reason to erase fine-grained canonicals

This also means reconciliation persistence is required for both `EntityType`
and `Entity`, not just one of them. Stable cross-format conversion depends on
knowing both:

- which canonical type a format-specific type corresponds to
- which canonical entity a format-specific record corresponds to

Where identity is not exact, `mapping_records`, `projection_maps`, and
`loss_notes` should carry directional conversion knowledge instead of implying
that all mappings are symmetric.

## Core Responsibilities

1. **Canonical data model**
   - Entity, EntityType, Source, Citation, and script/runtime-neutral rule structures.
2. **Cross-format identity mapping**
   - External IDs from each format mapped to one canonical record.
3. **Reconciliation workflows**
   - Match imported records to existing canonical records with confidence/scoring.
4. **Loss-aware normalization**
   - Preserve unknown or format-specific payloads while still producing canonical nodes.
5. **Deterministic projection support**
   - Provide sufficient data and provenance for downstream format-specific unparsers.

## Privileged vs Data-Defined Types

`artisan-core` should keep the privileged set intentionally small.

- **Data-defined (default):** most domain concepts, including `Character`, are regular `EntityType` records.
- **Privileged (platform-level):** only records/actions that must exist for cross-system operation and cannot be safely represented as user-defined game content.

Current privileged records are `Publisher`, `Source`, and `Citation` because cross-format reconciliation depends on them as global provenance anchors.

`Character` is an important `EntityType`, but not inherently privileged. It should remain data-defined unless engine constraints later prove a platform-owned schema is required.

Privilege decisions should follow engine primitives and interoperability requirements (identity, provenance, event/runtime contracts), not source-format conventions.

## Non-Goals

`artisan-core` does not parse PCGen tokens or HeroLab XML directly. Those responsibilities stay in adapter crates (`artisan-pcgen`, `artisan-herolab`).

`artisan-core` also does not own UI concerns.

## Architecture Layers

```
Format Adapters (artisan-pcgen, artisan-herolab, ...)
    ↓ produce import candidates + provenance
[artisan-core] Ingest + Reconciliation Layer
    ├─ identity resolution
    ├─ source/citation normalization
    ├─ entity/entity-type matching
    └─ conflict tracking
    ↓ canonical graph
[artisan-core] Canonical Domain Layer
    ├─ Entity
    ├─ EntityType
    ├─ Source
    ├─ Publisher
    ├─ Citation
    ├─ Rule/Effect/Prerequisite/Script IR
    └─ CharacterGraph
    ↓ projection request
Format Adapters (target unparser)
```

## Canonical Domain Model

### Identity primitives

```rust
pub struct CanonicalId(pub uuid::Uuid);

pub struct ExternalId {
    pub format: FormatId,            // pcgen, herolab, hlo, ...
    pub namespace: Option<String>,   // optional source/tool namespace
    pub value: String,               // raw external key/id
}

pub enum SubjectKind {
    Entity,
    EntityType,
    Source,
    Citation,
}
```

Every canonical record can carry multiple `ExternalId` values. This is the foundation of cross-format matching.

### Source and citation (first-class)

```rust
pub struct SourceRecord {
    pub id: CanonicalId,
    pub title: String,
    pub publisher: Option<String>,
    pub publisher_ids: Vec<CanonicalId>,
    pub edition: Option<String>,
    pub license: Option<String>,
    pub game_systems: Vec<String>,
    pub external_ids: Vec<ExternalId>,
}

pub struct PublisherRecord {
    pub id: CanonicalId,
    pub name: String,
    pub external_ids: Vec<ExternalId>,
}

pub struct CitationRecord {
    pub id: CanonicalId,
    pub subject: SubjectRef,         // Entity or EntityType
    pub source: CanonicalId,
    pub locators: Vec<CitationLocator>,
    pub verification: VerificationState,
    pub external_ids: Vec<ExternalId>,
}
```

Publishers, sources, and citations are privileged platform records and are required for robust reconciliation.

Note: this privileged status does not extend to gameplay types like `Character`, `Monster`, or `Spell`; those stay in the game-system-defined `EntityType` layer.

### Entity and EntityType

`Entity` and `EntityType` mirror the docs model:

- descriptive fields and mechanical fields
- typed references
- validation metadata
- provenance and fidelity notes

The model must support completeness states (`Complete`, `Descriptive`, `Inferred`, `Stub`) to handle partial imports.

### Rules and scripting

Rules are runtime-neutral IR:

- `Effect`
- `Prerequisite`
- `Trigger`
- `ScriptProgram` / expression variants

Format adapters map their native scripting/token systems into this IR. `artisan-core` remains language/runtime agnostic.

### CharacterGraph

Because HeroLab lead/portfolio structures are graph-like and PCGen has relationship-rich content, core includes a generic graph shape for persisted state:

```rust
pub struct CharacterGraph {
    pub nodes: indexmap::IndexMap<CanonicalId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
    pub opaque_extensions: Vec<OpaqueExtension>,
}
```

This allows preservation of richly structured imports while enabling semantic normalization.

## Reconciliation Engine

Reconciliation is the heart of `artisan-core`.

### Input contract from adapters

Adapters submit normalized candidates:

```rust
pub struct ImportCandidate<T> {
    pub payload: T,
    pub external_ids: Vec<ExternalId>,
    pub display_name: Option<String>,
    pub source_hints: Vec<SourceHint>,
    pub provenance: Provenance,
}
```

### Resolution stages

1. **Exact external ID match**
   - Fast path: if `format + namespace + value` already mapped, reuse canonical record.
2. **Strong metadata match**
   - Name + game system + type + source alignment.
3. **Heuristic similarity scoring**
   - Field-level similarity, structural fingerprints, known aliases.
4. **Conflict/ambiguity handling**
   - If confidence below threshold or tie exists, emit review-required result.

### Resolution result

```rust
pub enum ResolutionOutcome {
    Matched { id: CanonicalId, confidence: f32 },
    Created { id: CanonicalId },
    Ambiguous { candidates: Vec<MatchCandidate> },
    Conflict { existing: CanonicalId, details: ConflictSet },
}
```

No silent destructive merge is allowed.

## Source/Citation Normalization Rules

To support your source matching requirement, core enforces:

1. Resolve Source first (external IDs, then metadata similarity).
2. Resolve EntityType and Entity in Source-aware context when possible.
3. Merge Citation by `(subject, source)` with additive locator sets.
4. Preserve all locator forms; do not downgrade detailed locator sets.

This aligns imported PCGen/HeroLab source representations into a single canonical source graph.

## Conversion Workflow (PCGen <-> HeroLab)

### Example: PCGen -> HeroLab

```
PCGen parser emits candidates
    ↓
core reconciles source/entity/entity-type identities
    ↓
core returns canonical records + mapping table
    ↓
HeroLab unparser projects canonical records to HeroLab XML/script model
```

### Example: HeroLab -> PCGen

```
HeroLab parser emits candidates
    ↓
core reconciles against same canonical store
    ↓
core returns canonical records + mapping table
    ↓
PCGen unparser projects canonical records to LST/PCC model
```

The mapping table is explicit and reusable per conversion job.

```rust
pub struct ProjectionMap {
    pub canonical_to_external: indexmap::IndexMap<CanonicalId, ExternalProjection>,
    pub unresolved: Vec<CanonicalId>,
    pub lossy_notes: Vec<LossNote>,
}
```

## Fidelity and Opaque Preservation

Cross-format conversion cannot assume full equivalence.

`artisan-core` therefore stores:

- `OpaqueExtension` for unknown source payloads
- `LossNote` for known semantic gaps
- provenance with file/line/token/script references

Adapters can choose strict mode (fail on lossy projection) or permissive mode (emit with diagnostics).

## Public API Surface (core)

```rust
pub trait ReconciliationStore {
    fn find_by_external_id(&self, kind: SubjectKind, id: &ExternalId) -> Option<CanonicalId>;
    fn search_candidates(&self, kind: SubjectKind, query: MatchQuery) -> Vec<MatchCandidate>;
    fn upsert_subject(&mut self, subject: CanonicalSubject) -> CanonicalId;
    fn link_external_id(&mut self, kind: SubjectKind, canonical: CanonicalId, id: ExternalId);
}

pub struct Reconciler<S: ReconciliationStore> {
    pub store: S,
    pub policy: ReconciliationPolicy,
}

impl<S: ReconciliationStore> Reconciler<S> {
    pub fn reconcile_sources(&mut self, candidates: Vec<ImportCandidate<SourceRecord>>) -> Vec<ResolutionOutcome>;
    pub fn reconcile_entity_types(&mut self, candidates: Vec<ImportCandidate<EntityType>>) -> Vec<ResolutionOutcome>;
    pub fn reconcile_entities(&mut self, candidates: Vec<ImportCandidate<Entity>>) -> Vec<ResolutionOutcome>;
}
```

Core remains storage-agnostic; concrete persistence can be SQLite-backed in higher crates.

## Crate Structure

```
artisan-core/
├── src/
│   ├── lib.rs
│   ├── id/
│   │   ├── canonical.rs
│   │   ├── external.rs
│   │   └── format.rs
│   ├── domain/
│   │   ├── entity.rs
│   │   ├── entity_type.rs
│   │   ├── source.rs
│   │   ├── citation.rs
│   │   ├── rules.rs
│   │   ├── script.rs
│   │   └── graph.rs
│   ├── reconcile/
│   │   ├── mod.rs
│   │   ├── matcher.rs
│   │   ├── scoring.rs
│   │   ├── conflicts.rs
│   │   └── policy.rs
│   ├── projection/
│   │   ├── map.rs
│   │   └── loss.rs
│   ├── provenance/
│   │   ├── mod.rs
│   │   └── opaque.rs
│   └── diagnostics/
│       ├── codes.rs
│       └── report.rs
├── tests/
│   ├── source_resolution.rs
│   ├── entity_matching.rs
│   ├── citation_merge.rs
│   ├── pcgen_to_herolab_flow.rs
│   └── herolab_to_pcgen_flow.rs
└── DESIGN.md
```

## Reconciliation Policies

Core policy is configurable:

- `Strict`: reject ambiguous or lossy outcomes.
- `Guided`: return ambiguity/conflict sets for user confirmation.
- `Permissive`: create inferred/stub records with diagnostics.

Default should be `Guided` for import UX safety.

## Testing Strategy

1. **Unit tests for identity and matching**
   - external ID exact match, fuzzy match thresholds, tie behavior.
2. **Source/citation merge tests**
   - additive locator behavior and canonical locator preservation.
3. **Round-trip reconciliation tests**
   - PCGen fixture import -> canonical -> HeroLab projection map.
   - HeroLab fixture import -> canonical -> PCGen projection map.
4. **Conflict/ambiguity tests**
   - same name with divergent mechanics/source context.
5. **Fidelity diagnostics tests**
   - ensure lossy conversions are surfaced deterministically.

## Implementation Roadmap

**Phase 1: Canonical identity and domain skeleton**
- [ ] CanonicalId/ExternalId/FormatId primitives
- [ ] Source/Citation/Entity/EntityType base records
- [ ] Provenance and OpaqueExtension types

**Phase 2: Reconciliation core**
- [ ] Store trait and in-memory reference implementation
- [ ] Match/scoring engine
- [ ] Conflict and ambiguity result model
- [ ] Source-first reconciliation pipeline

**Phase 3: Projection and conversion contract**
- [ ] ProjectionMap and LossNote
- [ ] Adapter-facing reconciliation APIs
- [ ] Contract tests with fixture candidates from pcgen/herolab crates

**Phase 4: Hardening**
- [ ] Performance profiling on large datasets
- [ ] Deterministic diagnostics and stable ordering
- [ ] Schema evolution/migration hooks

## Immediate Next Steps

1. Replace placeholder `src/lib.rs` with module exports for domain + reconcile + provenance.
2. Implement `id` and `domain::source/citation` first (foundation for source matching).
3. Add `ReconciliationStore` in-memory implementation for adapter integration tests.
4. Wire first end-to-end test with mocked PCGen/HeroLab candidates.
