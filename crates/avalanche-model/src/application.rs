use serde::{Deserialize, Serialize};

use crate::scope::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationId {
    pub scope: Scope,
    pub category: String,
    pub name: String,
}

impl ApplicationId {
    pub fn option_path(&self) -> String {
        match self.scope {
            Scope::System => format!("gb.programs.system.{}.{}", self.category, self.name),
            Scope::Home => format!("gb.home.programs.{}.{}", self.category, self.name),
        }
    }

    pub fn requester_name(&self) -> String {
        match self.scope {
            Scope::System => format!("programs.system.{}.{}", self.category, self.name),
            Scope::Home => format!("programs.home.{}.{}", self.category, self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Application {
    pub id: ApplicationId,
    pub module_path: String,
}
