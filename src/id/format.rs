use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatId {
    ArtisanToml,
    Pcgen,
    Herolab,
    Hlo,
    Foundry,
    Other(String),
}
