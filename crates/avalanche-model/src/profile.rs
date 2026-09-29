use serde::{Deserialize, Serialize};

use crate::scope::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId {
    pub scope: Scope,
    pub name: String,
}

impl ProfileId {
    pub fn new(scope: Scope, name: impl Into<String>) -> Self {
        Self {
            scope,
            name: name.into(),
        }
    }

    pub fn requester_name(&self) -> String {
        format!("profiles.{}.{}", self.scope, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub source_file: String,
}
