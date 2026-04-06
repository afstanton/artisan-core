use serde::{Deserialize, Serialize};

use crate::id::FormatId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source_format: FormatId,
    pub location: Option<SourceLocation>,
    pub source_fragment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpaqueExtension {
    pub path: String,
    pub raw_xml: Option<String>,
    pub raw_script: Option<String>,
    pub attributes: Vec<(String, String)>,
}
