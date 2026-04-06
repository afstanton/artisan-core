use serde::{Deserialize, Serialize};

use super::script::ScriptProgram;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleHook {
    pub phase: Option<String>,
    pub priority: Option<i32>,
    pub index: Option<i32>,
    pub script: Option<ScriptProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub kind: String,
    pub target: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    pub kind: String,
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub event: String,
    pub condition: Option<String>,
}
