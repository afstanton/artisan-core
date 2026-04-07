use crate::domain::CoreCatalog;

/// Parses a serialized representation of a `CoreCatalog` from a string.
///
/// Formats that only produce import candidates (e.g. PCGen before it supports
/// full round-trip) should not implement this trait — use `Reconciler` instead.
pub trait CatalogParser {
    type Error;
    fn parse_catalog(input: &str) -> Result<CoreCatalog, Self::Error>;
}

/// Serializes a `CoreCatalog` to a string representation.
pub trait CatalogUnparser {
    type Error;
    fn unparse_catalog(catalog: &CoreCatalog) -> Result<String, Self::Error>;
}
