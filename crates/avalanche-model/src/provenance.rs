use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::json::JsonValue;
use crate::layer::{Layer, Priority};
use crate::option::{DefinitionType, OptionDefinition};
use crate::source::SourceLocation;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub option_path: String,
    pub definitions: Vec<OptionDefinition>,
    pub effective_value: Option<JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub value: JsonValue,
    pub source: Option<SourceLocation>,
    pub layer: Option<Layer>,
    pub priority: Option<Priority>,
    pub definition_type: Option<DefinitionType>,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub condition: Option<Condition>,
}

impl Provenance {
    pub fn new(option_path: impl Into<String>) -> Self {
        Self {
            option_path: option_path.into(),
            definitions: Vec::new(),
            effective_value: None,
        }
    }

    pub fn add(&mut self, definition: OptionDefinition) {
        self.definitions.push(definition);
    }

    pub fn set_effective_value(&mut self, value: Option<JsonValue>) {
        self.effective_value = value;
    }

    pub fn sort_definitions(&mut self) {
        self.definitions.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| layer_rank(&a.layer).cmp(&layer_rank(&b.layer)))
                .then_with(|| a.source.file.cmp(&b.source.file))
                .then_with(|| a.source.span.start.line.cmp(&b.source.span.start.line))
        });
    }

    pub fn winning_definition(&self) -> Option<&OptionDefinition> {
        self.definitions.iter().min_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| layer_rank(&b.layer).cmp(&layer_rank(&a.layer)))
        })
    }

    pub fn overridden_definitions(&self) -> Vec<&OptionDefinition> {
        let winner = self
            .winning_definition()
            .map(|w| (w.priority, w.layer.clone(), w.source.clone()));
        self.definitions
            .iter()
            .filter(|d| {
                winner
                    .as_ref()
                    .map(|(p, l, s)| d.priority != *p || d.layer != *l || d.source != *s)
                    .unwrap_or(true)
            })
            .collect()
    }

    pub fn contributions(&self) -> Vec<Contribution> {
        if let Some(effective) = &self.effective_value {
            match &effective.0 {
                serde_json::Value::Array(items) => items
                    .iter()
                    .map(|item| {
                        let definition = self
                            .definitions
                            .iter()
                            .find(|d| contributes_value(d.value.as_ref(), item))
                            .or_else(|| self.winning_definition());
                        Contribution::new(JsonValue(item.clone()), definition)
                    })
                    .collect(),
                serde_json::Value::Object(map) => map
                    .iter()
                    .map(|(key, value)| {
                        let definition = self
                            .definitions
                            .iter()
                            .find(|d| contributes_key(d.value.as_ref(), key))
                            .or_else(|| self.winning_definition());
                        Contribution::new(JsonValue(value.clone()), definition)
                    })
                    .collect(),
                _ => {
                    let definition = self.winning_definition();
                    let value = definition
                        .and_then(|d| d.value.clone())
                        .map(JsonValue)
                        .unwrap_or_else(|| effective.clone());
                    vec![Contribution::new(value, definition)]
                }
            }
        } else {
            self.definitions
                .iter()
                .filter_map(|d| {
                    d.value
                        .clone()
                        .map(|v| Contribution::new(JsonValue(v), Some(d)))
                })
                .collect()
        }
    }
}

impl Contribution {
    fn new(value: JsonValue, definition: Option<&OptionDefinition>) -> Self {
        Self {
            value,
            source: definition.map(|d| d.source.clone()),
            layer: definition.map(|d| d.layer.clone()),
            priority: definition.map(|d| d.priority),
            definition_type: definition.map(|d| d.definition_type.clone()),
            host: definition.and_then(|d| d.host.clone()),
            user: definition.and_then(|d| d.user.clone()),
            profile: definition.and_then(|d| d.profile.clone()),
            condition: definition.and_then(|d| d.condition.clone()),
        }
    }
}

fn layer_rank(layer: &Layer) -> u8 {
    match layer {
        Layer::External => 0,
        Layer::Profile => 1,
        Layer::Capability => 2,
        Layer::Program => 3,
        Layer::Desktop => 4,
        Layer::User => 5,
        Layer::Host => 6,
    }
}

fn contributes_value(value: Option<&serde_json::Value>, item: &serde_json::Value) -> bool {
    match value {
        Some(serde_json::Value::Array(items)) => items.contains(item),
        Some(other) => other == item,
        None => false,
    }
}

fn contributes_key(value: Option<&serde_json::Value>, key: &str) -> bool {
    match value {
        Some(serde_json::Value::Object(map)) => map.contains_key(key),
        _ => false,
    }
}
