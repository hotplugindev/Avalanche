pub mod error;
pub mod fingerprint;
pub mod semantic_index;
pub mod static_index;

pub use error::{IndexError, IndexResult};
pub use fingerprint::{compute_fingerprint, is_stale, INDEX_SCHEMA_VERSION};
pub use semantic_index::{build_semantic_index, SemanticIndex};
pub use static_index::{build_static_index, ParsedFile, StaticIndex};

use std::collections::HashMap;
use std::path::Path;

use avalanche_model::repository::{Fingerprint, Repository};
use avalanche_model::{
    Application, Capability, EffectiveValue, Host, Module, OptionDefinition, OptionSchema, Profile,
    User,
};
use avalanche_nix::NixService;
use avalanche_source::SourceService;

#[derive(Debug, Clone)]
pub struct RepositoryIndex {
    pub repository: Repository,
    pub options: HashMap<String, OptionSchema>,
    pub definitions: HashMap<String, Vec<OptionDefinition>>,
    pub effective: HashMap<String, EffectiveValue>,
    pub capabilities: Vec<Capability>,
    pub applications: Vec<Application>,
    pub profiles: Vec<Profile>,
    pub hosts: Vec<Host>,
    pub users: Vec<User>,
    pub modules: Vec<Module>,
    pub assignments: HashMap<String, Vec<avalanche_source::Assignment>>,
    pub source_locations: HashMap<String, Vec<avalanche_model::source::SourceLocation>>,
    pub imports: HashMap<String, Vec<String>>,
    pub assertions: HashMap<String, Vec<avalanche_nix::Assertion>>,
    pub requests: Vec<avalanche_model::ResolvedRequest>,
    pub flake_inputs: Vec<avalanche_nix::FlakeInput>,
    pub fingerprint: Fingerprint,
    pub static_index: StaticIndex,
    pub semantic_index: SemanticIndex,
}

impl RepositoryIndex {
    pub fn option(&self, path: &str) -> Option<&OptionSchema> {
        self.options.get(path).or_else(|| self.semantic_index.schema_for(path))
    }

    pub fn definitions_for(&self, path: &str) -> Option<&Vec<OptionDefinition>> {
        self.definitions.get(path)
    }

    pub fn effective_value(&self, path: &str) -> Option<&EffectiveValue> {
        self.effective
            .get(path)
            .or_else(|| self.semantic_index.effective_for(path))
    }

    pub fn assignment_for(&self, path: &str) -> &[avalanche_source::Assignment] {
        self.static_index.assignment_for(path)
    }

    pub fn source_location_for(&self, path: &str) -> &[avalanche_model::source::SourceLocation] {
        self.static_index.source_location_for(path)
    }

    pub fn imports_for(&self, file: &str) -> &[String] {
        self.imports
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn assertions_for_host(&self, host: &str) -> &[avalanche_nix::Assertion] {
        self.semantic_index.assertions_for_host(host)
    }

    pub fn failed_assertions(&self) -> Vec<(&str, &avalanche_nix::Assertion)> {
        self.semantic_index.failed_assertions()
    }

    pub fn active_requests(&self) -> Vec<&avalanche_model::ResolvedRequest> {
        self.semantic_index.active_requests()
    }

    pub fn modules_of_kind(&self, kind: &avalanche_model::ModuleKind) -> Vec<&Module> {
        self.static_index.modules_of_kind(kind)
    }

    pub fn host_by_name(&self, name: &str) -> Option<&Host> {
        self.hosts.iter().find(|h| h.name == name)
    }

    pub fn user_by_username(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }

    pub fn profile_by_name(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id.name == name)
    }

    pub fn is_stale(&self, current_fingerprint: &Fingerprint) -> bool {
        fingerprint::is_stale(&self.fingerprint, current_fingerprint)
    }

    pub fn where_is_value_written(&self, option_path: &str) -> Vec<&avalanche_source::Assignment> {
        self.assignment_for(option_path).iter().collect()
    }

    pub fn auto_editable(&self, option_path: &str) -> bool {
        self.static_index
            .auto_editable_assignments(option_path)
            .iter()
            .any(|a| a.is_auto_editable())
    }
}

pub struct IndexService {
    source: SourceService,
}

impl Default for IndexService {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexService {
    pub fn new() -> Self {
        Self {
            source: SourceService::new(),
        }
    }

    pub fn build(
        &self,
        repository: &Repository,
        nix: &NixService,
    ) -> IndexResult<RepositoryIndex> {
        let repo_path = Path::new(&repository.root);

        let fp = compute_fingerprint(repo_path)?;

        let static_idx = build_static_index(repo_path, &self.source)?;

        let semantic_idx = build_semantic_index(nix)?;

        let hosts = repository.hosts.clone();
        let profiles = repository.profiles.clone();
        let capabilities = repository
            .modules
            .iter()
            .filter(|m| m.id.kind == avalanche_model::ModuleKind::Capability)
            .map(|m| Capability {
                id: avalanche_model::CapabilityId {
                    scope: m.id.scope.unwrap_or(avalanche_model::Scope::System),
                    domain: m.id.name.clone(),
                    name: String::new(),
                },
                module_path: m.path.clone(),
                description: String::new(),
            })
            .collect();

        let mut effective = HashMap::new();
        for (path, ev) in &semantic_idx.effective {
            effective.insert(path.clone(), ev.clone());
        }

        let mut definitions = HashMap::new();
        for (path, assignments) in &static_idx.assignments {
            let defs: Vec<OptionDefinition> = assignments
                .iter()
                .map(|a| OptionDefinition {
                    option_path: path.clone(),
                    value: None,
                    source: a.location.clone(),
                    priority: avalanche_model::Priority::normal(),
                    layer: avalanche_model::Layer::Host,
                    scope: avalanche_model::Scope::System,
                    host: None,
                    user: None,
                    profile: None,
                    condition: None,
                    definition_type: avalanche_model::DefinitionType::Literal,
                })
                .collect();
            definitions.insert(path.clone(), defs);
        }

        Ok(RepositoryIndex {
            repository: repository.clone(),
            options: semantic_idx.schemas.clone(),
            definitions,
            effective,
            capabilities,
            applications: Vec::new(),
            profiles,
            hosts,
            users: repository.users.clone(),
            modules: static_idx.modules.clone(),
            assignments: static_idx.assignments.clone(),
            source_locations: static_idx.source_locations.clone(),
            imports: static_idx.imports.clone(),
            assertions: semantic_idx.assertions.clone(),
            requests: semantic_idx.requests.clone(),
            flake_inputs: semantic_idx.flake_inputs.clone(),
            fingerprint: fp,
            static_index: static_idx,
            semantic_index: semantic_idx,
        })
    }

    pub fn build_static_only(&self, repository: &Repository) -> IndexResult<RepositoryIndex> {
        let repo_path = Path::new(&repository.root);
        let fp = compute_fingerprint(repo_path)?;
        let static_idx = build_static_index(repo_path, &self.source)?;
        let semantic_idx = SemanticIndex::empty();

        let mut definitions = HashMap::new();
        for (path, assignments) in &static_idx.assignments {
            let defs: Vec<OptionDefinition> = assignments
                .iter()
                .map(|a| OptionDefinition {
                    option_path: path.clone(),
                    value: None,
                    source: a.location.clone(),
                    priority: avalanche_model::Priority::normal(),
                    layer: avalanche_model::Layer::Host,
                    scope: avalanche_model::Scope::System,
                    host: None,
                    user: None,
                    profile: None,
                    condition: None,
                    definition_type: avalanche_model::DefinitionType::Literal,
                })
                .collect();
            definitions.insert(path.clone(), defs);
        }

        Ok(RepositoryIndex {
            repository: repository.clone(),
            options: HashMap::new(),
            definitions,
            effective: HashMap::new(),
            capabilities: Vec::new(),
            applications: Vec::new(),
            profiles: repository.profiles.clone(),
            hosts: repository.hosts.clone(),
            users: repository.users.clone(),
            modules: static_idx.modules.clone(),
            assignments: static_idx.assignments.clone(),
            source_locations: static_idx.source_locations.clone(),
            imports: static_idx.imports.clone(),
            assertions: HashMap::new(),
            requests: Vec::new(),
            flake_inputs: Vec::new(),
            fingerprint: fp,
            static_index: static_idx,
            semantic_index: semantic_idx,
        })
    }

    pub fn check_stale(&self, repository: &Repository, stored: &Fingerprint) -> IndexResult<bool> {
        let repo_path = Path::new(&repository.root);
        let current = compute_fingerprint(repo_path)?;
        Ok(fingerprint::is_stale(stored, &current))
    }
}
