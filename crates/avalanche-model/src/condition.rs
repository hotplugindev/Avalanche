use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub expression: String,
    pub evaluated_value: Option<NixValue>,
    pub satisfied: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum NixValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Path(String),
    List(Vec<NixValue>),
    Attrs(std::collections::BTreeMap<String, NixValue>),
    Null,
}

impl PartialEq for NixValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NixValue::Bool(a), NixValue::Bool(b)) => a == b,
            (NixValue::Int(a), NixValue::Int(b)) => a == b,
            (NixValue::Float(a), NixValue::Float(b)) => a.to_bits() == b.to_bits(),
            (NixValue::Str(a), NixValue::Str(b)) => a == b,
            (NixValue::Path(a), NixValue::Path(b)) => a == b,
            (NixValue::List(a), NixValue::List(b)) => a == b,
            (NixValue::Attrs(a), NixValue::Attrs(b)) => a == b,
            (NixValue::Null, NixValue::Null) => true,
            _ => false,
        }
    }
}
