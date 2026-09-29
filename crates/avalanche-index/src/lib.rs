use avalanche_model::repository::Fingerprint;
use avalanche_model::{
    Application, Capability, EffectiveValue, Host, Module, OptionDefinition, OptionSchema, Profile,
    Repository,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub repository: Repository,
    pub options: HashMap<String, OptionSchema>,
    pub definitions: HashMap<String, Vec<OptionDefinition>>,
    pub effective: HashMap<String, EffectiveValue>,
    pub capabilities: Vec<Capability>,
    pub applications: Vec<Application>,
    pub profiles: Vec<Profile>,
    pub hosts: Vec<Host>,
    pub modules: Vec<Module>,
    pub fingerprint: Fingerprint,
}

impl RepositoryIndex {
    pub fn new(repository: Repository) -> Self {
        let fingerprint = repository.fingerprint.clone();
        Self {
            options: HashMap::new(),
            definitions: HashMap::new(),
            effective: HashMap::new(),
            capabilities: Vec::new(),
            applications: Vec::new(),
            profiles: Vec::new(),
            hosts: Vec::new(),
            modules: Vec::new(),
            repository,
            fingerprint,
        }
    }

    pub fn option(&self, path: &str) -> Option<&OptionSchema> {
        self.options.get(path)
    }

    pub fn definitions_for(&self, path: &str) -> Option<&Vec<OptionDefinition>> {
        self.definitions.get(path)
    }

    pub fn effective_value(&self, path: &str) -> Option<&EffectiveValue> {
        self.effective.get(path)
    }

    pub fn is_stale(&self, current_fingerprint: &Fingerprint) -> bool {
        self.fingerprint != *current_fingerprint
    }
}

pub struct IndexService;

impl Default for IndexService {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexService {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self, repository: Repository) -> RepositoryIndex {
        RepositoryIndex::new(repository)
    }
}
