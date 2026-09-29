use serde::{Deserialize, Serialize};

use crate::scope::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId {
    pub scope: Scope,
    pub domain: String,
    pub name: String,
}

impl CapabilityId {
    pub fn request_path(&self) -> String {
        match self.scope {
            Scope::System => format!("gb.requires.system.{}.{}", self.domain, self.name),
            Scope::Home => format!("gb.requires.home.{}.{}", self.domain, self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub module_path: String,
    pub description: String,
}
