use serde::{Deserialize, Serialize};

use crate::option::OptionDefinition;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub option_path: String,
    pub definitions: Vec<OptionDefinition>,
}

impl Provenance {
    pub fn new(option_path: impl Into<String>) -> Self {
        Self {
            option_path: option_path.into(),
            definitions: Vec::new(),
        }
    }

    pub fn add(&mut self, definition: OptionDefinition) {
        self.definitions.push(definition);
    }
}
