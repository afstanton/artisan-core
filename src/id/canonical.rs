use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanonicalId(pub Uuid);

impl CanonicalId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CanonicalId {
    fn default() -> Self {
        Self::new()
    }
}
