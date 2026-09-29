use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipStatus {
    Owned,
    Unowned,
    WrongOwner,
    Ambiguous,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnershipRecord {
    pub option_path: String,
    pub owner_module: String,
    pub status: OwnershipStatus,
}

#[derive(Debug, Clone, Default)]
pub struct OwnershipRegistry {
    records: HashMap<String, Vec<OwnershipRecord>>,
}

impl OwnershipRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, option_path: &str, owner_module: &str) {
        let entry = self.records.entry(option_path.to_string()).or_default();
        entry.push(OwnershipRecord {
            option_path: option_path.to_string(),
            owner_module: owner_module.to_string(),
            status: OwnershipStatus::Owned,
        });
    }

    pub fn resolve(&self, option_path: &str) -> OwnershipStatus {
        let records = match self.records.get(option_path) {
            Some(r) => r,
            None => return OwnershipStatus::Unowned,
        };

        if records.is_empty() {
            return OwnershipStatus::Unowned;
        }

        let owners: Vec<&str> = records.iter().map(|r| r.owner_module.as_str()).collect();
        if owners.len() == 1 {
            return OwnershipStatus::Owned;
        }

        let unique: std::collections::HashSet<&str> = owners.iter().copied().collect();
        if unique.len() > 1 {
            return OwnershipStatus::Ambiguous;
        }

        OwnershipStatus::Owned
    }

    pub fn owner_of(&self, option_path: &str) -> Option<&str> {
        let records = self.records.get(option_path)?;
        if records.len() == 1 {
            return Some(records[0].owner_module.as_str());
        }
        None
    }

    pub fn can_auto_edit(&self, option_path: &str) -> bool {
        matches!(self.resolve(option_path), OwnershipStatus::Owned)
    }
}

pub struct OwnershipService {
    registry: OwnershipRegistry,
}

impl Default for OwnershipService {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnershipService {
    pub fn new() -> Self {
        Self {
            registry: OwnershipRegistry::new(),
        }
    }

    pub fn registry(&self) -> &OwnershipRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut OwnershipRegistry {
        &mut self.registry
    }

    pub fn resolve(&self, option_path: &str) -> OwnershipStatus {
        self.registry.resolve(option_path)
    }

    pub fn owner_of(&self, option_path: &str) -> Option<&str> {
        self.registry.owner_of(option_path)
    }

    pub fn can_auto_edit(&self, option_path: &str) -> bool {
        self.registry.can_auto_edit(option_path)
    }
}
