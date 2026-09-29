use serde::{Deserialize, Serialize};

use crate::scope::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    Core,
    Profile,
    Capability,
    Program,
    Desktop,
    Aggregate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId {
    pub kind: ModuleKind,
    pub scope: Option<Scope>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    pub id: ModuleId,
    pub path: String,
    pub imported: bool,
}
