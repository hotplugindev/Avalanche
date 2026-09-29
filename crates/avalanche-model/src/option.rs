use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::json::JsonValue;
use crate::layer::{Layer, Priority};
use crate::scope::Scope;
use crate::source::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NixType {
    Bool,
    Int,
    Float,
    Str,
    Path,
    List,
    Attrs,
    AttrsOf,
    Submodule,
    Package,
    Enum,
    Function,
    Raw,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionSchema {
    pub path: String,
    pub scope: Scope,
    pub nix_type: NixType,
    pub default: Option<JsonValue>,
    pub description: Option<String>,
    pub example: Option<JsonValue>,
    pub internal: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionType {
    Literal,
    MkDefault,
    MkForce,
    MkIf,
    MkMerge,
    MkOverride,
    Expression,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionDefinition {
    pub option_path: String,
    pub value: Option<serde_json::Value>,
    pub source: SourceLocation,
    pub priority: Priority,
    pub layer: Layer,
    pub scope: Scope,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub condition: Option<Condition>,
    pub definition_type: DefinitionType,
}

impl OptionDefinition {
    pub fn is_mk_default(&self) -> bool {
        self.definition_type == DefinitionType::MkDefault || self.priority.is_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveValue {
    pub option_path: String,
    pub value: Option<JsonValue>,
    pub definitions: Vec<OptionDefinition>,
}

impl EffectiveValue {
    pub fn winning_definition(&self) -> Option<&OptionDefinition> {
        self.definitions
            .iter()
            .min_by(|a, b| a.priority.cmp(&b.priority))
    }
}
