use serde::{Deserialize, Serialize};

use super::FormatId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalId {
    pub format: FormatId,
    pub namespace: Option<String>,
    pub value: String,
}
