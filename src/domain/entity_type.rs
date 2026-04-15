use serde::{Deserialize, Serialize};

use crate::{
    id::{CanonicalId, ExternalId},
    provenance::Provenance,
};

// ---------------------------------------------------------------------------
// Logical field model
// ---------------------------------------------------------------------------

/// Value type for a logical field on an entity type.
///
/// This describes the *semantic* type of the field — independent of any
/// particular serialization format. Format adapters (artisan-pcgen, etc.)
/// map their own wire types onto these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Free-form text.
    Text,
    /// Integer numeric value.
    Integer,
    /// Decimal (floating-point) numeric value.
    Decimal,
    /// Boolean flag.
    Boolean,
    /// A value chosen from a closed enumeration of named variants.
    Enum(Vec<String>),
    /// A reference to another entity, identified by its type key.
    Reference(String),
    /// An ordered list of values of the given inner type.
    List(Box<FieldType>),
    /// An inline structured sub-record with its own named fields.
    ///
    /// Used for bracket-group values in PCGen, XML sub-elements in HeroLab, etc.
    Record(Vec<FieldDef>),
}

/// How many values a field may hold.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldCardinality {
    /// Exactly one value (or zero if not required).
    One,
    /// Zero or more values (always represented as a list).
    Many,
}

/// A single logical field on an entity type.
///
/// Field definitions are format-independent. They describe what data an entity
/// type carries semantically. Format adapters (artisan-pcgen, artisan-herolab,
/// etc.) bind their own token/attribute grammars to these field keys.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldDef {
    /// Canonical field key. Used as the bridge between format adapters.
    ///
    /// Format adapters reference this key in their mapping layers (e.g.,
    /// `ArtisanMapping::Field("hit_die")` in artisan-pcgen).
    pub key: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic value type.
    pub field_type: FieldType,
    /// How many values this field may hold.
    pub cardinality: FieldCardinality,
    /// Whether this field must be present for the entity to be valid.
    pub required: bool,
    /// Optional prose description of the field's meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Logical relationship model
// ---------------------------------------------------------------------------

/// Cardinality of a relationship between entity types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipCardinality {
    /// Each entity of this type relates to at most one entity of the target type.
    One,
    /// Each entity of this type may relate to many entities of the target type.
    Many,
}

/// A named relationship from this entity type to another.
///
/// Relationships are directional: they express "this entity type references
/// entities of another type." The inverse relationship (if any) is a separate
/// `RelationshipDef` on the target type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDef {
    /// Canonical relationship key, e.g. `"class_levels"`, `"granted_feats"`.
    pub key: String,
    /// Human-readable display name.
    pub name: String,
    /// The `EntityType.key` of the entity type this relationship points to.
    pub target_type_key: String,
    /// How many target entities this relationship may hold.
    pub cardinality: RelationshipCardinality,
    /// Whether at least one related entity must be present.
    pub required: bool,
    /// Optional prose description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// EntityType
// ---------------------------------------------------------------------------

/// The logical type definition for a class of artisan entities.
///
/// `EntityType` captures the *semantic* structure of an entity — what fields
/// and relationships it has — independently of any particular data format.
/// Format adapters (artisan-pcgen, artisan-herolab, etc.) bind their own
/// line/record grammars to the canonical field and relationship keys defined
/// here, using the `entity_type_key` string as the bridge.
///
/// # Relationship to format adapters
///
/// In artisan-pcgen, a `LineGrammar` declares which PCGen tokens map to which
/// `FieldDef.key` values via `ArtisanMapping::Field`. In artisan-herolab a
/// future `XmlElementGrammar` will map XML attribute names to the same field
/// keys. The entity type is the stable contract; format grammars are adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub id: CanonicalId,
    /// Canonical type key, e.g. `"pcgen:entity:class"`, `"dnd5e:spell"`.
    pub key: String,
    /// Human-readable name.
    pub name: String,
    /// The game system this type belongs to, e.g. `"35e"`, `"Pathfinder"`,
    /// `"dnd5e"`. `None` means the type is format-generic or the game system
    /// is not yet known.
    ///
    /// Used together with `key` and `external_ids` to find-or-create the
    /// canonical EntityType when loading data from any format adapter:
    /// two formats contributing data for the same game system and the same
    /// conceptual type should resolve to the same `EntityType`, not create
    /// duplicates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_system: Option<String>,
    /// Parent type, if this is a specialization of a more general type.
    pub parent: Option<CanonicalId>,
    /// Logical field definitions for this entity type.
    ///
    /// Populated by the domain layer. Format adapters bind their own
    /// token/attribute grammars to these field keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldDef>,
    /// Logical relationship definitions for this entity type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipDef>,
    pub external_ids: Vec<ExternalId>,
    pub provenance: Option<Provenance>,
}
