use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptProgram {
    pub source: Option<String>,
    pub statements: Vec<ScriptStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptStatement {
    Opaque(String),
}
