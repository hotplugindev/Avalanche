use avalanche_doctor::{Diagnosis, DoctorInputs, DoctorService};
use avalanche_git::GitService;
use avalanche_graph::{build_request_graph, GraphService, NodeId, NodeKind, RequestGraph};
use std::path::Path;

use avalanche_index::{compute_fingerprint, is_stale, IndexResult, IndexService};
use avalanche_model::{
    Application, EditKind, Layer, ModuleKind, MutationIntent, MutationPlan, NewFile,
    OwnershipStatus, RenderHint, Repository, ResolvedEdit, Scope, Transaction, ValidationReport,
    ValidationSeverity, ValidationTier,
};
use avalanche_nix::NixService;
use avalanche_ownership::{build_ownership_registry, OwnershipRegistry, OwnershipService};
use avalanche_source::SourceService;
use avalanche_validate::ValidationService;
use avalanche_write::{
    capability_module, category_default, init_skeleton, profile_module, program_module,
    CapabilitySpec, DiskFileSource, FileSource, InitSpec, ProfileSpec, ProgramSpec, WriteError,
    WriteService,
};
use serde::Serialize;

pub use avalanche_index::RepositoryIndex;

pub mod view;
pub use view::*;

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
            requests: index.requests.clone(),
        };
        self.doctor_service.diagnose(&inputs)
    }

    pub fn init_repository(target: &std::path::Path, spec: &InitSpec) -> Result<(), WriteError> {
        spec.validate().map_err(WriteError::EditFailed)?;

        let schema_path = target.join("modules/core/schema.nix");
        let flake_path = target.join("flake.nix");
        let target_str = target.display().to_string();

        if schema_path.exists() && flake_path.exists() {
            let engine = ConfigurationEngine::new(&target_str);
            let repo = Repository::new(&target_str);
            let index = engine.index_static(&repo).map_err(|e| {
                WriteError::EditFailed(format!("failed to index existing repository: {e}"))
            })?;

            let mut intents = Vec::new();
            let host_file = format!("hosts/{}/default.nix", spec.host_name);
            if !index_has_file(&index, &host_file) {
                intents.push(MutationIntent::CreateHost {
                    name: spec.host_name.clone(),
                });
            }
            let user_file = format!("users/{}.nix", spec.user_name);
            if !index_has_file(&index, &user_file) {
                intents.push(MutationIntent::CreateUser {
                    name: spec.user_name.clone(),
                });
            }
            if !intents.is_empty() {
                engine.execute_mutation(&index, intents)?;
            }
            return Ok(());
        }

        let skeleton = init_skeleton(spec);
        for (rel, content) in &skeleton {
            let full = target.join(rel);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).map_err(|e| WriteError::Io {
                    path: parent.display().to_string(),
                    reason: e.to_string(),
                })?;
            }
            std::fs::write(&full, content).map_err(|e| WriteError::Io {
                path: full.display().to_string(),
                reason: e.to_string(),
            })?;
        }

        avalanche_git::GitService::init(&target_str).map_err(|e| {
            let _ = std::fs::remove_dir_all(target);
            WriteError::EditFailed(format!("git init failed: {e}"))
        })?;

        let engine = ConfigurationEngine::new(&target_str);
        let repo = Repository::new(&target_str);
        let index = engine.index_static(&repo).map_err(|e| {
            let _ = std::fs::remove_dir_all(target);
            WriteError::EditFailed(format!("failed to index new repository: {e}"))
        })?;

        let diag = engine.diagnose(&index);
        if diag.error_count() > 0 {
            let msgs: Vec<String> = diag
                .findings
                .iter()
                .filter(|f| f.severity == ValidationSeverity::Error)
                .map(|f| f.message.clone())
                .collect();
            let _ = std::fs::remove_dir_all(target);
            return Err(WriteError::ValidationFailed(format!(
                "repository failed doctor validation: {}; rolled back",
                msgs.join("; ")
            )));
        }

        Ok(())
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
        let mut creates: Vec<NewFile> = Vec::new();
        for intent in &intents {
            let resolved = resolve_intent(index, &registry, intent)?;
            edits.extend(resolved.edits);
            creates.extend(resolved.creates);
        }

        self.write_service
            .plan_edits(&edits, &creates, &files, index.fingerprint.clone())
    }

    pub fn execute_mutation(
        &self,
        index: &RepositoryIndex,
        intents: Vec<MutationIntent>,
    ) -> Result<Transaction, WriteError> {
        self.execute_mutation_with_tier(index, intents, ValidationTier::Structural)
    }

    pub fn execute_mutation_with_tier(
        &self,
        index: &RepositoryIndex,
        intents: Vec<MutationIntent>,
        tier: ValidationTier,
    ) -> Result<Transaction, WriteError> {
        let plan = self.plan_mutation(index, intents.clone())?;

        let mut pre_flight: Vec<(&str, &str)> = plan
            .file_changes
            .iter()
            .map(|c| (c.path.as_str(), c.after.as_str()))
            .collect();
        pre_flight.extend(
            plan.create
                .iter()
                .map(|nf| (nf.path.as_str(), nf.content.as_str())),
        );
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

        let post_flight: Vec<(&str, &str)> = txn
            .file_changes
            .iter()
            .map(|c| (c.path.as_str(), c.after.as_str()))
            .collect();
        let post_structural = self.validation_service.validate_structural(&post_flight);

        let mut report = ValidationReport::new(tier);
        for finding in structural.findings {
            report.add(finding);
        }
        for finding in post_structural.findings {
            report.add(finding);
        }

        let hosts = if tier >= ValidationTier::NixEval {
            affected_hosts(&txn)
        } else {
            Vec::new()
        };

        if tier >= ValidationTier::NixEval {
            let eval_report = self
                .validation_service
                .validate_nix_eval(&self.nix_service, &hosts);
            for finding in eval_report.findings {
                report.add(finding);
            }
            for host in &hosts {
                let assertions = self
                    .validation_service
                    .validate_assertions(&self.nix_service, host);
                for finding in assertions.findings {
                    report.add(finding);
                }
            }
        }

        if tier >= ValidationTier::FlakeCheck {
            let flake_report = self
                .validation_service
                .validate_flake_check(&self.nix_service);
            for finding in flake_report.findings {
                report.add(finding);
            }
        }

        if tier >= ValidationTier::Build {
            let build_report = self
                .validation_service
                .validate_build(&self.nix_service, &hosts);
            for finding in build_report.findings {
                report.add(finding);
            }
        }

        txn.validation = Some(report);
        Ok(txn)
    }

    pub fn validate_transaction(&self, txn: &Transaction) -> ValidationReport {
        let hosts = affected_hosts(txn);

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

fn affected_hosts(txn: &Transaction) -> Vec<String> {
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
#[serde(rename_all = "camelCase")]
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

    pub fn search_applications<'a>(
        &self,
        index: &'a RepositoryIndex,
        query: &str,
    ) -> Vec<&'a Application> {
        let q = query.to_lowercase();
        index
            .applications
            .iter()
            .filter(|app| {
                if q.is_empty() {
                    return true;
                }
                let scope_str = match app.id.scope {
                    Scope::System => "system",
                    Scope::Home => "home",
                };
                app.id.name.to_lowercase().contains(&q)
                    || app.id.category.to_lowercase().contains(&q)
                    || scope_str.contains(&q)
                    || app.module_path.to_lowercase().contains(&q)
                    || app.id.requester_name().to_lowercase().contains(&q)
                    || app.id.option_path().to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn set_application_enabled(
        &self,
        index: &RepositoryIndex,
        app: &Application,
        host: &str,
        enabled: bool,
    ) -> Result<Transaction, WriteError> {
        let enable_path = format!("{}.enable", app.id.option_path());
        let intent = if enabled {
            MutationIntent::SetHostOverride {
                path: enable_path,
                value: serde_json::json!(true),
                host: host.to_string(),
            }
        } else {
            MutationIntent::ResetHostOverride {
                path: enable_path,
                host: host.to_string(),
            }
        };
        self.execute_mutation(index, vec![intent])
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedIntent {
    edits: Vec<ResolvedEdit>,
    creates: Vec<NewFile>,
}

fn single(edit: ResolvedEdit) -> ResolvedIntent {
    ResolvedIntent {
        edits: vec![edit],
        creates: Vec::new(),
    }
}

fn resolve_intent(
    index: &RepositoryIndex,
    registry: &OwnershipRegistry,
    intent: &MutationIntent,
) -> Result<ResolvedIntent, WriteError> {
    match intent {
        MutationIntent::SetOption { path, value, scope } => Ok(single(resolve_set_option(
            index, registry, path, value, scope,
        )?)),
        MutationIntent::SetProfileDefault {
            path,
            value,
            profile,
        } => Ok(single(resolve_set_profile_default(
            index, path, value, profile,
        )?)),
        MutationIntent::SetHostOverride { path, value, host } => {
            Ok(single(resolve_set_host_override(index, path, value, host)?))
        }
        MutationIntent::ResetHostOverride { path, host } => {
            Ok(single(resolve_reset_host_override(index, path, host)?))
        }
        MutationIntent::EnableCapability {
            capability,
            requester,
        } => Ok(single(resolve_add_request(index, capability, requester)?)),
        MutationIntent::DisableCapability {
            capability,
            requester,
        } => Ok(single(resolve_remove_request(
            index, capability, requester,
        )?)),
        MutationIntent::AddRequest {
            capability,
            requester,
        } => Ok(single(resolve_add_request(index, capability, requester)?)),
        MutationIntent::RemoveRequest {
            capability,
            requester,
        } => Ok(single(resolve_remove_request(
            index, capability, requester,
        )?)),
        MutationIntent::CreateHost { name } => resolve_create_host(index, name),
        MutationIntent::CreateUser { name } => resolve_create_user(index, name),
        MutationIntent::CreateModule { name, kind } => resolve_create_module(index, name, kind),
    }
}

fn validate_entity_name(kind: &str, name: &str) -> Result<(), WriteError> {
    let valid = !name.is_empty()
        && name
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        return Err(WriteError::EditFailed(format!(
            "invalid {kind} name '{name}'; names must be Nix identifiers using letters, digits, '-' or '_'; repository was not modified"
        )));
    }
    Ok(())
}

fn index_has_file(index: &RepositoryIndex, rel_path: &str) -> bool {
    let normalized = rel_path.replace('\\', "/");
    index
        .static_index
        .parsed_files
        .iter()
        .any(|f| f.relative_path.replace('\\', "/") == normalized)
        || index
            .static_index
            .parse_errors
            .iter()
            .any(|e| e.file.replace('\\', "/") == normalized)
}

fn resolve_create_host(index: &RepositoryIndex, name: &str) -> Result<ResolvedIntent, WriteError> {
    validate_entity_name("host", name)?;
    let host_file = format!("hosts/{name}/default.nix");
    if index_has_file(index, &host_file) {
        return Err(WriteError::EditFailed(format!(
            "host '{name}' already exists; repository was not modified"
        )));
    }
    let registration = "hosts/default.nix";
    if !index_has_file(index, registration) {
        return Err(WriteError::EditFailed(format!(
            "hosts/default.nix is missing; cannot register host '{name}'; repository was not modified"
        )));
    }

    let edit = ResolvedEdit {
        file: registration.to_string(),
        option_path: name.to_string(),
        kind: EditKind::InsertBinding,
        value: Some(serde_json::json!({
            "system": "x86_64-linux",
            "class": "desktop",
            "roles": ["workstation"],
            "desktop": "none",
            "shell": "zsh",
            "stateVersion": "25.05",
            "hardware": {
                "cpu": "generic",
                "gpu": "none",
                "hasBattery": false,
                "hasBluetooth": false,
                "hasTouchpad": false,
                "hasPrinter": false
            },
            "extraModules": []
        })),
        render: RenderHint::Literal,
        layer: Layer::Host,
        scope: Scope::System,
        host: Some(name.to_string()),
        user: None,
        profile: None,
        reason: format!("register host {name} in hosts/default.nix"),
    };

    Ok(ResolvedIntent {
        edits: vec![edit],
        creates: vec![NewFile {
            path: host_file,
            content: format!("{{\n  gb.host.name = \"{name}\";\n}}\n"),
        }],
    })
}

fn resolve_create_user(index: &RepositoryIndex, name: &str) -> Result<ResolvedIntent, WriteError> {
    validate_entity_name("user", name)?;
    let user_file = format!("users/{name}.nix");
    if index_has_file(index, &user_file) {
        return Err(WriteError::EditFailed(format!(
            "user '{name}' already exists; repository was not modified"
        )));
    }
    let registration = "users/default.nix";
    if !index_has_file(index, registration) {
        return Err(WriteError::EditFailed(format!(
            "users/default.nix is missing; cannot register user '{name}'; repository was not modified"
        )));
    }

    let edit = ResolvedEdit {
        file: registration.to_string(),
        option_path: name.to_string(),
        kind: EditKind::InsertBinding,
        value: Some(serde_json::json!({
            "username": name,
            "fullName": "",
            "email": "",
            "isMain": false,
            "homeModule": {
                "__nix_type__": "path",
                "value": format!("./{name}.nix")
            }
        })),
        render: RenderHint::Literal,
        layer: Layer::User,
        scope: Scope::Home,
        host: None,
        user: Some(name.to_string()),
        profile: None,
        reason: format!("register user {name} in users/default.nix"),
    };

    Ok(ResolvedIntent {
        edits: vec![edit],
        creates: vec![NewFile {
            path: user_file,
            content: format!(
                "{{\n  home.username = \"{name}\";\n  home.homeDirectory = \"/home/{name}\";\n}}\n"
            ),
        }],
    })
}

fn resolve_create_module(
    index: &RepositoryIndex,
    name: &str,
    kind: &str,
) -> Result<ResolvedIntent, WriteError> {
    match kind.to_lowercase().as_str() {
        "program" => resolve_create_program(index, name),
        "capability" => resolve_create_capability(index, name),
        "profile" => resolve_create_profile(index, name),
        _ => Err(WriteError::EditFailed(format!(
            "unknown module kind '{kind}'; expected program|capability|profile"
        ))),
    }
}

fn parse_scope_prefix(name: &str) -> (&str, &str) {
    if let Some((scope, rest)) = name.split_once(':') {
        (scope, rest)
    } else {
        ("system", name)
    }
}

fn extract_import_paths(source: &str) -> Option<Vec<String>> {
    let file = avalanche_source::parse(source).ok()?;
    find_imports_list(&file.expr)
}

fn find_imports_list(expr: &avalanche_source::Expr) -> Option<Vec<String>> {
    match expr {
        avalanche_source::Expr::Lambda { body, .. } => find_imports_list(body),
        avalanche_source::Expr::Attrs(bindings) => {
            for b in bindings {
                if b.path.len() == 1 && b.path[0] == "imports" {
                    if let Some(avalanche_source::Expr::List(items)) = &b.value {
                        let mut paths = Vec::new();
                        for item in items {
                            match item {
                                avalanche_source::Expr::Path(p) => paths.push(p.clone()),
                                _ => return None,
                            }
                        }
                        return Some(paths);
                    }
                    return None;
                }
            }
            None
        }
        _ => None,
    }
}

fn read_imports_from_aggregate(
    index: &RepositoryIndex,
    agg_path: &str,
) -> Result<Vec<String>, WriteError> {
    let source = DiskFileSource::new(&index.repository.root)
        .read(agg_path)
        .map_err(|e| {
            WriteError::EditFailed(format!(
                "cannot read aggregate '{agg_path}': {e}; repository was not modified"
            ))
        })?;
    extract_import_paths(&source).ok_or_else(|| {
        WriteError::EditFailed(format!(
            "aggregate '{agg_path}' has no parsable imports list of paths; repository was not modified"
        ))
    })
}

fn make_imports_edit(
    file: &str,
    paths: &[String],
    layer: Layer,
    scope: Scope,
    reason: &str,
) -> ResolvedEdit {
    let value = serde_json::Value::Array(
        paths
            .iter()
            .map(|p| serde_json::json!({"__nix_type__": "path", "value": p}))
            .collect(),
    );
    ResolvedEdit {
        file: file.to_string(),
        option_path: "imports".to_string(),
        kind: EditKind::ReplaceValue,
        value: Some(value),
        render: RenderHint::Literal,
        layer,
        scope,
        host: None,
        user: None,
        profile: None,
        reason: reason.to_string(),
    }
}

fn resolve_create_program(
    index: &RepositoryIndex,
    name: &str,
) -> Result<ResolvedIntent, WriteError> {
    let (scope_str, rest) = parse_scope_prefix(name);
    let scope = parse_scope(scope_str).ok_or_else(|| {
        WriteError::EditFailed(format!(
            "invalid scope '{scope_str}' for program; expected system|home"
        ))
    })?;

    let (category, pname) = match scope {
        Scope::System => {
            if rest.contains('.') {
                return Err(WriteError::EditFailed(format!(
                    "system programs cannot have a category; got '{rest}'"
                )));
            }
            (String::new(), rest.to_string())
        }
        Scope::Home => {
            if let Some((cat, n)) = rest.split_once('.') {
                (cat.to_string(), n.to_string())
            } else {
                (String::new(), rest.to_string())
            }
        }
    };

    let spec = ProgramSpec {
        scope,
        category,
        name: pname,
        description: rest.to_string(),
    };
    spec.validate().map_err(WriteError::EditFailed)?;

    let mp = spec.module_path();
    if index_has_file(index, &mp) {
        return Err(WriteError::EditFailed(format!(
            "module file '{mp}' already exists; repository was not modified"
        )));
    }

    let content = program_module(&spec);
    let mut creates = vec![NewFile {
        path: mp.clone(),
        content,
    }];
    let mut edits = Vec::new();

    let agg_path = spec.aggregate_path();
    let mut existing_imports = read_imports_from_aggregate(index, &agg_path)?;

    let import_to_add = if scope == Scope::Home && !spec.category.is_empty() {
        let cat_default = spec.category_default_path().unwrap();
        if !index_has_file(index, &cat_default) {
            let cat_content = category_default(&[format!("{}.nix", spec.name)]);
            creates.push(NewFile {
                path: cat_default,
                content: cat_content,
            });
        } else {
            let cat_source = DiskFileSource::new(&index.repository.root)
                .read(&cat_default)
                .map_err(|e| {
                    WriteError::EditFailed(format!(
                        "cannot read category default '{cat_default}': {e}"
                    ))
                })?;
            let mut cat_imports = extract_import_paths(&cat_source).ok_or_else(|| {
                WriteError::EditFailed(format!(
                    "category default '{cat_default}' has no parsable imports list"
                ))
            })?;
            let member = format!("./{}.nix", spec.name);
            if cat_imports.contains(&member) {
                return Err(WriteError::EditFailed(format!(
                    "'{member}' already registered in '{cat_default}'; repository was not modified"
                )));
            }
            cat_imports.push(member);
            edits.push(make_imports_edit(
                &cat_default,
                &cat_imports,
                Layer::Program,
                scope,
                &format!("register {} in category default", spec.name),
            ));
        }
        spec.category_relative_import().unwrap()
    } else {
        spec.relative_import()
    };

    existing_imports.push(import_to_add);
    edits.insert(
        0,
        make_imports_edit(
            &agg_path,
            &existing_imports,
            Layer::Program,
            scope,
            &format!("register module {mp} in aggregate"),
        ),
    );

    Ok(ResolvedIntent { edits, creates })
}

fn resolve_create_capability(
    index: &RepositoryIndex,
    name: &str,
) -> Result<ResolvedIntent, WriteError> {
    let (scope_str, rest) = parse_scope_prefix(name);
    let scope = parse_scope(scope_str).ok_or_else(|| {
        WriteError::EditFailed(format!(
            "invalid scope '{scope_str}' for capability; expected system|home"
        ))
    })?;

    let (domain, cname) = rest.split_once('.').ok_or_else(|| {
        WriteError::EditFailed(format!(
            "invalid capability '{name}'; expected '<domain>.<name>' or '<scope>:<domain>.<name>'"
        ))
    })?;

    let spec = CapabilitySpec {
        scope,
        domain: domain.to_string(),
        name: cname.to_string(),
        description: rest.to_string(),
    };
    spec.validate().map_err(WriteError::EditFailed)?;

    let mp = spec.module_path();
    if index_has_file(index, &mp) {
        return Err(WriteError::EditFailed(format!(
            "module file '{mp}' already exists; repository was not modified"
        )));
    }

    let content = capability_module(&spec);
    let creates = vec![NewFile {
        path: mp.clone(),
        content,
    }];

    let agg_path = spec.aggregate_path();
    let mut existing = read_imports_from_aggregate(index, &agg_path)?;
    existing.push(spec.relative_import());

    let agg_edit = make_imports_edit(
        &agg_path,
        &existing,
        Layer::Capability,
        scope,
        &format!("register module {mp} in aggregate"),
    );

    Ok(ResolvedIntent {
        edits: vec![agg_edit],
        creates,
    })
}

fn resolve_create_profile(
    index: &RepositoryIndex,
    name: &str,
) -> Result<ResolvedIntent, WriteError> {
    let (scope_str, pname) = parse_scope_prefix(name);
    let scope = parse_scope(scope_str).ok_or_else(|| {
        WriteError::EditFailed(format!(
            "invalid scope '{scope_str}' for profile; expected system|home"
        ))
    })?;

    let spec = ProfileSpec {
        scope,
        name: pname.to_string(),
    };
    spec.validate().map_err(WriteError::EditFailed)?;

    let mp = spec.module_path();
    if index_has_file(index, &mp) {
        return Err(WriteError::EditFailed(format!(
            "module file '{mp}' already exists; repository was not modified"
        )));
    }

    let content = profile_module(&spec);
    let creates = vec![NewFile {
        path: mp.clone(),
        content,
    }];

    let agg_path = spec.aggregate_path();
    let mut existing = read_imports_from_aggregate(index, &agg_path)?;
    existing.push(spec.relative_import());

    let agg_edit = make_imports_edit(
        &agg_path,
        &existing,
        Layer::Profile,
        scope,
        &format!("register module {mp} in aggregate"),
    );

    Ok(ResolvedIntent {
        edits: vec![agg_edit],
        creates,
    })
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

fn resolve_capability_path(capability: &str) -> Result<String, WriteError> {
    let (scope_str, rest) = if let Some(r) = capability.strip_prefix("home:") {
        ("home", r)
    } else {
        ("system", capability)
    };
    let (domain, name) = rest.split_once('.').ok_or_else(|| {
        WriteError::EditFailed(format!(
            "invalid capability '{capability}': expected '<domain>.<name>' or 'home:<domain>.<name>'"
        ))
    })?;
    if domain.is_empty() || name.is_empty() {
        return Err(WriteError::EditFailed(format!(
            "invalid capability '{capability}': domain and name must be non-empty"
        )));
    }
    Ok(format!("gb.requires.{scope_str}.{domain}.{name}"))
}

fn resolve_requester_file(index: &RepositoryIndex, requester: &str) -> Result<String, WriteError> {
    let normalized = requester.replace('\\', "/");

    if let Some(rest) = normalized.strip_prefix("profiles.") {
        if let Some((scope_str, name)) = rest.split_once('.') {
            if let Some(scope) = parse_scope(scope_str) {
                if let Some(profile) = index
                    .profiles
                    .iter()
                    .find(|p| p.id.scope == scope && p.id.name == name)
                {
                    return Ok(profile.source_file.clone());
                }
            }
        }
    }

    if let Some(name) = normalized.strip_prefix("hosts.") {
        if !name.contains('/') {
            return Ok(format!("hosts/{name}/default.nix"));
        }
    }

    if let Some(name) = normalized.strip_prefix("users.") {
        if !name.contains('/') {
            return Ok(format!("users/{name}.nix"));
        }
    }

    let candidate = normalized.replace('.', "/");
    if let Some(module) = index.modules.iter().find(|m| m.id.name == candidate) {
        return Ok(module.path.clone());
    }

    Err(WriteError::EditFailed(format!(
        "cannot resolve requester '{requester}' to a file: searched profiles, modules (as '{candidate}'), hosts, and users"
    )))
}

fn layer_for_requester_file(index: &RepositoryIndex, file: &str) -> Layer {
    if let Some(module) = index.modules.iter().find(|m| m.path == file) {
        return match module.id.kind {
            ModuleKind::Core => Layer::External,
            ModuleKind::Profile => Layer::Profile,
            ModuleKind::Capability => Layer::Capability,
            ModuleKind::Program => Layer::Program,
            ModuleKind::Desktop => Layer::Desktop,
            ModuleKind::Aggregate => Layer::Host,
        };
    }
    let normalized = file.replace('\\', "/");
    if normalized.starts_with("hosts/") {
        if normalized.contains("/home/") {
            return Layer::User;
        }
        return Layer::Host;
    }
    if normalized.starts_with("users/") {
        return Layer::User;
    }
    Layer::Host
}

fn requester_provenance(requester: &str) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(name) = requester.strip_prefix("hosts.") {
        return (Some(name.to_string()), None, None);
    }
    if let Some(name) = requester.strip_prefix("users.") {
        return (None, Some(name.to_string()), None);
    }
    if let Some(rest) = requester.strip_prefix("profiles.") {
        if let Some((_scope, name)) = rest.split_once('.') {
            return (None, None, Some(name.to_string()));
        }
    }
    (None, None, None)
}

fn resolve_add_request(
    index: &RepositoryIndex,
    capability: &str,
    requester: &str,
) -> Result<ResolvedEdit, WriteError> {
    let option_path = resolve_capability_path(capability)?;
    let file = resolve_requester_file(index, requester)?;

    let existing = index
        .provenance
        .get(&option_path)
        .and_then(|p| p.definitions.iter().find(|d| d.source.file == file));

    if let Some(def) = existing {
        let kind = edit_kind_for(index, &option_path, &file, true)?;
        let current_list = match &def.value {
            Some(serde_json::Value::Array(items)) => items.clone(),
            Some(_) => {
                return Err(WriteError::EditFailed(format!(
                    "existing definition of '{option_path}' in {file} is not a list; refusing to modify"
                )));
            }
            None => {
                return Err(WriteError::EditFailed(format!(
                    "existing definition of '{option_path}' in {file} has no extractable value; refusing to modify"
                )));
            }
        };
        let requester_val = serde_json::Value::String(requester.to_string());
        if current_list.contains(&requester_val) {
            return Err(WriteError::EditFailed(format!(
                "requester '{requester}' already present in '{option_path}' in {file}; nothing was modified"
            )));
        }
        let mut new_list = current_list;
        new_list.push(requester_val);
        let reason = format!("add request {requester} to {option_path}");

        Ok(ResolvedEdit {
            file,
            option_path,
            kind,
            value: Some(serde_json::Value::Array(new_list)),
            render: RenderHint::Literal,
            layer: def.layer.clone(),
            scope: def.scope,
            host: def.host.clone(),
            user: def.user.clone(),
            profile: def.profile.clone(),
            reason,
        })
    } else {
        let kind = edit_kind_for(index, &option_path, &file, false)?;
        let layer = layer_for_requester_file(index, &file);
        let scope = infer_scope_from_path(&option_path);
        let (host, user, profile) = requester_provenance(requester);
        let reason = format!("insert request {requester} into {option_path}");

        Ok(ResolvedEdit {
            file,
            option_path,
            kind,
            value: Some(serde_json::json!([requester])),
            render: RenderHint::Literal,
            layer,
            scope,
            host,
            user,
            profile,
            reason,
        })
    }
}

fn resolve_remove_request(
    index: &RepositoryIndex,
    capability: &str,
    requester: &str,
) -> Result<ResolvedEdit, WriteError> {
    let option_path = resolve_capability_path(capability)?;
    let file = resolve_requester_file(index, requester)?;

    let def = index
        .provenance
        .get(&option_path)
        .and_then(|p| p.definitions.iter().find(|d| d.source.file == file))
        .ok_or_else(|| {
            WriteError::EditFailed(format!(
                "no definition of '{option_path}' in {file}; requester '{requester}' not found, nothing was modified"
            ))
        })?;

    let current_list = match &def.value {
        Some(serde_json::Value::Array(items)) => items.clone(),
        Some(_) => {
            return Err(WriteError::EditFailed(format!(
                "existing definition of '{option_path}' in {file} is not a list; refusing to modify"
            )));
        }
        None => {
            return Err(WriteError::EditFailed(format!(
                "existing definition of '{option_path}' in {file} has no extractable value; refusing to modify"
            )));
        }
    };

    let requester_val = serde_json::Value::String(requester.to_string());
    let pos = current_list.iter().position(|v| *v == requester_val).ok_or_else(|| {
        WriteError::EditFailed(format!(
            "requester '{requester}' not found in request list for '{option_path}'; nothing was modified"
        ))
    })?;

    let mut new_list = current_list;
    new_list.remove(pos);

    if new_list.is_empty() {
        let reason = format!("remove last request {requester} from {option_path} (delete binding)");
        Ok(ResolvedEdit {
            file,
            option_path,
            kind: EditKind::RemoveBinding,
            value: None,
            render: RenderHint::Literal,
            layer: def.layer.clone(),
            scope: def.scope,
            host: def.host.clone(),
            user: def.user.clone(),
            profile: def.profile.clone(),
            reason,
        })
    } else {
        let kind = edit_kind_for(index, &option_path, &file, true)?;
        let reason = format!("remove request {requester} from {option_path}");
        Ok(ResolvedEdit {
            file,
            option_path,
            kind,
            value: Some(serde_json::Value::Array(new_list)),
            render: RenderHint::Literal,
            layer: def.layer.clone(),
            scope: def.scope,
            host: def.host.clone(),
            user: def.user.clone(),
            profile: def.profile.clone(),
            reason,
        })
    }
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
