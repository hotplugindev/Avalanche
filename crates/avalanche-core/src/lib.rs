use avalanche_doctor::{Diagnosis, DoctorInputs, DoctorService};
use avalanche_git::GitService;
use avalanche_graph::{build_request_graph, GraphService, NodeId, NodeKind, RequestGraph};
use std::path::Path;

use avalanche_index::{compute_fingerprint, is_stale, IndexResult, IndexService};
use avalanche_model::{
    EditKind, Layer, MutationIntent, MutationPlan, OwnershipStatus, RenderHint, Repository,
    ResolvedEdit, Scope, Transaction, ValidationReport, ValidationSeverity,
};
use avalanche_nix::NixService;
use avalanche_ownership::{build_ownership_registry, OwnershipRegistry, OwnershipService};
use avalanche_source::SourceService;
use avalanche_validate::ValidationService;
use avalanche_write::{DiskFileSource, WriteError, WriteService};
use serde::Serialize;

pub use avalanche_index::RepositoryIndex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WhyChain {
    pub via: String,
    pub steps: Vec<String>,
}

pub struct ConfigurationEngine {
    pub index_service: IndexService,
    pub write_service: WriteService,
    pub git_service: GitService,
    pub doctor_service: DoctorService,
    pub validation_service: ValidationService,
    pub ownership_service: OwnershipService,
    pub graph_service: GraphService,
    pub nix_service: NixService,
    pub source_service: SourceService,
}

impl ConfigurationEngine {
    pub fn new(repo_path: impl Into<String>) -> Self {
        let repo_path = repo_path.into();
        Self {
            index_service: IndexService::new(),
            write_service: WriteService::new(),
            git_service: GitService::new(repo_path.clone()),
            doctor_service: DoctorService::new(),
            validation_service: ValidationService::new(),
            ownership_service: OwnershipService::new(),
            graph_service: GraphService::new(),
            nix_service: NixService::new(repo_path.clone()),
            source_service: SourceService::new(),
        }
    }

    pub fn index(&self, repository: &Repository) -> IndexResult<RepositoryIndex> {
        self.index_service.build(repository, &self.nix_service)
    }

    pub fn index_static(&self, repository: &Repository) -> IndexResult<RepositoryIndex> {
        self.index_service.build_static_only(repository)
    }

    pub fn ownership(&self, index: &RepositoryIndex) -> OwnershipRegistry {
        build_ownership_registry(&index.provenance, &index.modules, &index.requests)
    }

    pub fn load_ownership(&mut self, index: &RepositoryIndex) {
        let registry = build_ownership_registry(&index.provenance, &index.modules, &index.requests);
        self.ownership_service = OwnershipService::with_registry(registry);
    }

    pub fn graph(&self, index: &RepositoryIndex) -> RequestGraph {
        build_request_graph(
            &index.provenance,
            &index.requests,
            &index.modules,
            &index.imports,
        )
    }

    pub fn load_graph(&mut self, index: &RepositoryIndex) {
        let graph = build_request_graph(
            &index.provenance,
            &index.requests,
            &index.modules,
            &index.imports,
        );
        self.graph_service = GraphService::with_graph(graph);
    }

    pub fn why(&self, index: &RepositoryIndex, input: &str) -> Result<Vec<WhyChain>, String> {
        let graph = self.graph(index);
        let node = resolve_node(&graph, input)
            .ok_or_else(|| format!("unknown node '{}' in request graph", input))?;
        let chains = match node.kind {
            NodeKind::Option => {
                let mut v = Vec::new();
                for owner in graph.owners(&node) {
                    for path in graph.why(owner) {
                        v.push(WhyChain {
                            via: owner.display(),
                            steps: path.iter().map(NodeId::display).collect(),
                        });
                    }
                }
                v
            }
            _ => graph
                .why(&node)
                .into_iter()
                .map(|path| WhyChain {
                    via: node.display(),
                    steps: path.iter().map(NodeId::display).collect(),
                })
                .collect(),
        };
        Ok(chains)
    }

    pub fn dependents(&self, index: &RepositoryIndex, input: &str) -> Result<Vec<String>, String> {
        let graph = self.graph(index);
        let node = resolve_node(&graph, input)
            .ok_or_else(|| format!("unknown node '{}' in request graph", input))?;
        Ok(graph
            .dependents(&node)
            .iter()
            .map(|n| n.display())
            .collect())
    }

    pub fn diagnose(&self, index: &RepositoryIndex) -> Diagnosis {
        let inputs = DoctorInputs {
            repository: index.repository.clone(),
            provenance: index.provenance.clone(),
            modules: index.modules.clone(),
            imports: index.imports.clone(),
            schemas: index.options.clone(),
        };
        self.doctor_service.diagnose(&inputs)
    }

    pub fn plan_mutation(
        &self,
        index: &RepositoryIndex,
        intents: Vec<MutationIntent>,
    ) -> Result<MutationPlan, WriteError> {
        let current =
            compute_fingerprint(Path::new(&index.repository.root)).map_err(|e| WriteError::Io {
                path: index.repository.root.clone(),
                reason: e.to_string(),
            })?;
        if is_stale(&index.fingerprint, &current) {
            return Err(WriteError::StaleIndex);
        }

        let registry = build_ownership_registry(&index.provenance, &index.modules, &index.requests);
        let files = DiskFileSource::new(&index.repository.root);

        let mut edits: Vec<ResolvedEdit> = Vec::new();
        for intent in &intents {
            edits.push(resolve_intent(index, &registry, intent)?);
        }

        self.write_service
            .plan_edits(&edits, &files, index.fingerprint.clone())
    }

    pub fn execute_mutation(
        &self,
        index: &RepositoryIndex,
        intents: Vec<MutationIntent>,
    ) -> Result<Transaction, WriteError> {
        let plan = self.plan_mutation(index, intents.clone())?;

        let pre_flight: Vec<(&str, &str)> = plan
            .file_changes
            .iter()
            .map(|c| (c.path.as_str(), c.after.as_str()))
            .collect();
        let structural = self.validation_service.validate_structural(&pre_flight);
        if !structural.passed {
            let msgs: Vec<String> = structural
                .findings
                .iter()
                .filter(|f| f.severity == ValidationSeverity::Error)
                .map(|f| f.message.clone())
                .collect();
            return Err(WriteError::ValidationFailed(format!(
                "{}; repository was not modified",
                msgs.join("; ")
            )));
        }

        let mut files = DiskFileSource::new(&index.repository.root);
        let mut txn = self.write_service.apply_plan(plan, intents, &mut files)?;
        txn.validation = Some(structural);
        Ok(txn)
    }

    pub fn validate_transaction(&self, txn: &Transaction) -> ValidationReport {
        let hosts: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            txn.file_changes
                .iter()
                .filter_map(|c| {
                    let path = c.path.replace('\\', "/");
                    path.strip_prefix("hosts/")
                        .and_then(|rest| rest.split('/').next())
                        .map(str::to_string)
                })
                .filter(|h| seen.insert(h.clone()))
                .collect()
        };

        let mut report = self
            .validation_service
            .validate_nix_eval(&self.nix_service, &hosts);
        for host in &hosts {
            let assertions = self
                .validation_service
                .validate_assertions(&self.nix_service, host);
            for finding in assertions.findings {
                report.add(finding);
            }
        }
        report
    }

    pub fn plan(&self, intents: Vec<MutationIntent>) -> Result<Transaction, WriteError> {
        self.write_service.plan(intents)
    }

    pub fn apply(&mut self, transaction: &mut Transaction) -> Result<(), WriteError> {
        self.write_service.apply(transaction)
    }
}

fn resolve_node(graph: &RequestGraph, input: &str) -> Option<NodeId> {
    for kind in [
        NodeKind::Capability,
        NodeKind::Option,
        NodeKind::Profile,
        NodeKind::Program,
        NodeKind::Host,
        NodeKind::User,
        NodeKind::Desktop,
        NodeKind::Aggregate,
        NodeKind::Core,
        NodeKind::Module,
    ] {
        let node = NodeId::new(kind, input);
        if graph.nodes.contains_key(&node) {
            return Some(node);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnerEntry {
    pub option_path: String,
    pub owner_module: String,
    pub owner_kind: String,
    pub status: OwnershipStatus,
    pub contributing_files: Vec<String>,
}

impl ConfigurationEngine {
    pub fn owners(&self, index: &RepositoryIndex, option_path: &str) -> Result<OwnerEntry, String> {
        let registry = build_ownership_registry(&index.provenance, &index.modules, &index.requests);
        let record = registry
            .record_for(option_path)
            .ok_or_else(|| format!("no ownership record for '{}'", option_path))?;
        Ok(OwnerEntry {
            option_path: record.option_path.clone(),
            owner_module: record.owner_module.clone(),
            owner_kind: format!("{:?}", record.owner_kind),
            status: record.status,
            contributing_files: record.contributing_files.clone(),
        })
    }

    pub fn diff(&self) -> Result<String, String> {
        self.git_service.diff().map_err(|e| e.to_string())
    }
}

fn resolve_intent(
    index: &RepositoryIndex,
    registry: &OwnershipRegistry,
    intent: &MutationIntent,
) -> Result<ResolvedEdit, WriteError> {
    match intent {
        MutationIntent::SetOption { path, value, scope } => {
            resolve_set_option(index, registry, path, value, scope)
        }
        MutationIntent::SetProfileDefault {
            path,
            value,
            profile,
        } => resolve_set_profile_default(index, path, value, profile),
        MutationIntent::SetHostOverride { path, value, host } => {
            resolve_set_host_override(index, path, value, host)
        }
        MutationIntent::ResetHostOverride { path, host } => {
            resolve_reset_host_override(index, path, host)
        }
        other => Err(WriteError::Unsupported(format!(
            "{} is not yet supported by the write planner; repository was not modified",
            intent_name(other)
        ))),
    }
}

fn resolve_set_option(
    index: &RepositoryIndex,
    registry: &OwnershipRegistry,
    path: &str,
    value: &serde_json::Value,
    scope: &str,
) -> Result<ResolvedEdit, WriteError> {
    let record = registry.record_for(path).ok_or_else(|| {
        WriteError::OwnershipDenied(format!(
            "option '{path}' is not registered in the repository model; refusing to guess a target file"
        ))
    })?;
    if record.status != OwnershipStatus::Owned {
        return Err(WriteError::OwnershipDenied(format!(
            "option '{path}' has ownership {:?} (owner '{}'); only Owned options are auto-editable",
            record.status, record.owner_module
        )));
    }

    let owner = record.owner_module.clone();
    let existing = index
        .provenance
        .get(path)
        .and_then(|p| p.definitions.iter().find(|d| d.source.file == owner));
    let layer = existing
        .map(|d| d.layer.clone())
        .unwrap_or(Layer::Capability);
    let eff_scope = existing
        .map(|d| d.scope)
        .or_else(|| parse_scope(scope))
        .unwrap_or(Scope::System);
    let kind = edit_kind_for(index, path, &owner, existing.is_some())?;

    Ok(ResolvedEdit {
        file: owner,
        option_path: path.to_string(),
        kind,
        value: Some(value.clone()),
        render: RenderHint::Literal,
        layer,
        scope: eff_scope,
        host: existing.and_then(|d| d.host.clone()),
        user: existing.and_then(|d| d.user.clone()),
        profile: existing.and_then(|d| d.profile.clone()),
        reason: format!("set {path}"),
    })
}

fn resolve_set_profile_default(
    index: &RepositoryIndex,
    path: &str,
    value: &serde_json::Value,
    profile: &str,
) -> Result<ResolvedEdit, WriteError> {
    let prof = index
        .profiles
        .iter()
        .find(|p| p.id.name == profile)
        .ok_or_else(|| WriteError::EditFailed(format!("unknown profile '{profile}'")))?;
    let file = prof.source_file.clone();
    let existing = index.provenance.get(path).and_then(|p| {
        p.definitions
            .iter()
            .find(|d| d.source.file == file && d.layer == Layer::Profile)
    });
    let kind = edit_kind_for(index, path, &file, existing.is_some())?;

    Ok(ResolvedEdit {
        file,
        option_path: path.to_string(),
        kind,
        value: Some(value.clone()),
        render: RenderHint::MkDefault,
        layer: Layer::Profile,
        scope: prof.id.scope,
        host: None,
        user: None,
        profile: Some(profile.to_string()),
        reason: format!("set profile default for {path} in {profile}"),
    })
}

fn resolve_set_host_override(
    index: &RepositoryIndex,
    path: &str,
    value: &serde_json::Value,
    host: &str,
) -> Result<ResolvedEdit, WriteError> {
    let (file, existing) = locate_host_def(index, path, host);
    let kind = edit_kind_for(index, path, &file, existing)?;
    let scope = host_scope(index, path, &file);

    Ok(ResolvedEdit {
        file,
        option_path: path.to_string(),
        kind,
        value: Some(value.clone()),
        render: RenderHint::Literal,
        layer: Layer::Host,
        scope,
        host: Some(host.to_string()),
        user: None,
        profile: None,
        reason: format!("set host override for {path} on {host}"),
    })
}

fn resolve_reset_host_override(
    index: &RepositoryIndex,
    path: &str,
    host: &str,
) -> Result<ResolvedEdit, WriteError> {
    let (file, existing) = locate_host_def(index, path, host);
    if !existing {
        return Err(WriteError::EditFailed(format!(
            "no host override for '{path}' on host '{host}'; nothing to reset, repository was not modified"
        )));
    }
    let scope = host_scope(index, path, &file);

    Ok(ResolvedEdit {
        file,
        option_path: path.to_string(),
        kind: EditKind::RemoveBinding,
        value: None,
        render: RenderHint::Literal,
        layer: Layer::Host,
        scope,
        host: Some(host.to_string()),
        user: None,
        profile: None,
        reason: format!("reset host override for {path} on {host} (delete, not write false)"),
    })
}

fn host_scope(index: &RepositoryIndex, path: &str, file: &str) -> Scope {
    index
        .provenance
        .get(path)
        .and_then(|p| p.definitions.iter().find(|d| d.source.file == file))
        .map(|d| d.scope)
        .unwrap_or_else(|| infer_scope_from_path(path))
}

fn edit_kind_for(
    index: &RepositoryIndex,
    path: &str,
    file: &str,
    has_existing_def: bool,
) -> Result<EditKind, WriteError> {
    if !has_existing_def {
        return Ok(EditKind::InsertBinding);
    }
    match index
        .static_index
        .assignment_for(path)
        .iter()
        .find(|a| a.location.file == file)
    {
        Some(a) if a.is_auto_editable() => Ok(EditKind::ReplaceValue),
        Some(a) => Err(WriteError::Unsupported(format!(
            "current definition of '{path}' in {file} is {:?}; refusing to silently transform a computed/unknown expression",
            a.kind
        ))),
        None => Ok(EditKind::InsertBinding),
    }
}

fn locate_host_def(index: &RepositoryIndex, path: &str, host: &str) -> (String, bool) {
    if let Some(def) = index.provenance.get(path).and_then(|p| {
        p.definitions
            .iter()
            .find(|d| d.layer == Layer::Host && d.host.as_deref() == Some(host))
    }) {
        return (def.source.file.clone(), true);
    }
    (format!("hosts/{host}/default.nix"), false)
}

fn parse_scope(s: &str) -> Option<Scope> {
    match s {
        "system" => Some(Scope::System),
        "home" => Some(Scope::Home),
        _ => None,
    }
}

fn infer_scope_from_path(path: &str) -> Scope {
    if path.starts_with("gb.home.") || path.contains(".home.") {
        Scope::Home
    } else {
        Scope::System
    }
}

fn intent_name(intent: &MutationIntent) -> &'static str {
    match intent {
        MutationIntent::SetOption { .. } => "SetOption",
        MutationIntent::SetProfileDefault { .. } => "SetProfileDefault",
        MutationIntent::SetHostOverride { .. } => "SetHostOverride",
        MutationIntent::ResetHostOverride { .. } => "ResetHostOverride",
        MutationIntent::EnableCapability { .. } => "EnableCapability",
        MutationIntent::DisableCapability { .. } => "DisableCapability",
        MutationIntent::AddRequest { .. } => "AddRequest",
        MutationIntent::RemoveRequest { .. } => "RemoveRequest",
        MutationIntent::CreateModule { .. } => "CreateModule",
        MutationIntent::CreateHost { .. } => "CreateHost",
        MutationIntent::CreateUser { .. } => "CreateUser",
    }
}
