use std::collections::{HashMap, HashSet};

use avalanche_doctor::{Diagnosis, RepairAction};
use avalanche_graph::{EdgeKind, NodeId, NodeKind, RequestGraph};
use avalanche_model::{
    ApplicationId, CapabilityId, Condition, DefinitionType, Host, Layer, ModuleKind, NixValue,
    OptionSchema, OwnershipStatus, ProfileId, Provenance, Scope, User, ValidationSeverity,
    ValidationTier,
};
use serde::Serialize;
use serde_json::Value;

use crate::{ConfigurationEngine, OwnerEntry, RepositoryIndex};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryView {
    pub root: String,
    pub schema_version: Option<u32>,
    pub fingerprint: RepositoryFingerprintView,
    pub host_count: usize,
    pub user_count: usize,
    pub profile_count: usize,
    pub application_count: usize,
    pub capability_count: usize,
    pub option_count: usize,
    pub active_request_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFingerprintView {
    pub git_head: Option<String>,
    pub working_tree_hash: Option<String>,
    pub flake_lock_hash: Option<String>,
    pub schema_version: Option<u32>,
    pub index_schema_version: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostView {
    pub name: String,
    pub system: String,
    pub class: avalanche_model::HostClass,
    pub roles: Vec<avalanche_model::HostRole>,
    pub desktop: avalanche_model::Desktop,
    pub shell: avalanche_model::Shell,
    pub state_version: String,
    pub hardware: HardwareView,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareView {
    pub cpu: avalanche_model::Cpu,
    pub gpu: avalanche_model::Gpu,
    pub has_battery: bool,
    pub has_bluetooth: bool,
    pub has_touchpad: bool,
    pub has_printer: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub is_main: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionView {
    pub expression: String,
    pub evaluated_value: Option<NixValue>,
    pub satisfied: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphView {
    pub nodes: Vec<NodeView>,
    pub edges: Vec<EdgeView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeView {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorView {
    pub findings: Vec<FindingView>,
    pub repairs: Vec<RepairView>,
    pub tier: ValidationTier,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingView {
    pub severity: ValidationSeverity,
    pub message: String,
    pub path: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairView {
    pub action: String,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityView {
    pub id: CapabilityId,
    pub request_path: String,
    pub module_path: String,
    pub requesters: Vec<String>,
    pub active: bool,
    pub owned_options: Vec<String>,
    pub ownership_status: Option<OwnershipStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationView {
    pub id: ApplicationId,
    pub module_path: String,
    pub option_path: String,
    pub requester_name: String,
    pub requested_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub id: ProfileId,
    pub source_file: String,
    pub requester_name: String,
    pub requested_capabilities: Vec<String>,
    pub default_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionView {
    pub path: String,
    pub schema: Option<OptionSchema>,
    pub effective: Option<Value>,
    pub provenance: Option<Provenance>,
    pub owner: Option<OwnerEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceView {
    pub option_path: String,
    pub effective_value: Option<Value>,
    pub definitions: Vec<DefinitionView>,
    pub winning: Option<DefinitionView>,
    pub overridden: Vec<DefinitionView>,
    pub contributions: Vec<ContributionView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionView {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub value: Option<Value>,
    pub layer: Layer,
    pub priority: i16,
    pub is_default: bool,
    pub scope: Scope,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub condition: Option<ConditionView>,
    pub definition_type: DefinitionType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionView {
    pub value: Value,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub layer: Option<Layer>,
    pub priority: Option<i16>,
    pub definition_type: Option<DefinitionType>,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String,
    pub name: String,
    pub detail: String,
}

impl ConfigurationEngine {
    pub fn repository_view(&self, index: &RepositoryIndex) -> RepositoryView {
        let diag = self.diagnose(index);
        RepositoryView {
            root: index.repository.root.clone(),
            schema_version: index.repository.schema_version,
            fingerprint: RepositoryFingerprintView {
                git_head: index.fingerprint.git_head.clone(),
                working_tree_hash: index.fingerprint.working_tree_hash.clone(),
                flake_lock_hash: index.fingerprint.flake_lock_hash.clone(),
                schema_version: index.fingerprint.schema_version,
                index_schema_version: index.fingerprint.index_schema_version,
            },
            host_count: index.hosts.len(),
            user_count: index.users.len(),
            profile_count: index.profiles.len(),
            application_count: index.applications.len(),
            capability_count: self.capabilities_view(index).len(),
            option_count: index.provenance.len(),
            active_request_count: index.requests.iter().filter(|r| r.active).count(),
            error_count: diag.error_count(),
            warning_count: diag.warning_count(),
        }
    }

    pub fn hosts_view(&self, index: &RepositoryIndex) -> Vec<HostView> {
        index.hosts.iter().map(host_view).collect()
    }

    pub fn users_view(&self, index: &RepositoryIndex) -> Vec<UserView> {
        index.users.iter().map(user_view).collect()
    }

    pub fn profiles_view(&self, index: &RepositoryIndex) -> Vec<ProfileView> {
        let rm = request_map(index);
        index
            .profiles
            .iter()
            .map(|p| {
                let requester = p.id.requester_name();
                let default_options = index
                    .provenance
                    .iter()
                    .filter(|(_, prov)| {
                        prov.definitions
                            .iter()
                            .any(|d| d.source.file == p.source_file && d.layer == Layer::Profile)
                    })
                    .map(|(k, _)| k.clone())
                    .collect();
                ProfileView {
                    id: p.id.clone(),
                    source_file: p.source_file.clone(),
                    requester_name: requester.clone(),
                    requested_capabilities: rm.get(&requester).cloned().unwrap_or_default(),
                    default_options,
                }
            })
            .collect()
    }

    pub fn applications_view(&self, index: &RepositoryIndex) -> Vec<ApplicationView> {
        let rm = request_map(index);
        index
            .applications
            .iter()
            .map(|a| {
                let requester = a.id.requester_name();
                ApplicationView {
                    id: a.id.clone(),
                    module_path: a.module_path.clone(),
                    option_path: a.id.option_path(),
                    requester_name: requester.clone(),
                    requested_capabilities: rm.get(&requester).cloned().unwrap_or_default(),
                }
            })
            .collect()
    }

    pub fn capabilities_view(&self, index: &RepositoryIndex) -> Vec<CapabilityView> {
        let mut out: Vec<CapabilityView> = Vec::new();
        let mut seen: HashSet<(u8, String, String)> = HashSet::new();

        for (path, prov) in &index.provenance {
            let Some((scope, rest)) = require_parts(path) else {
                continue;
            };
            let (domain, name) = split_cap(rest);
            let id = CapabilityId {
                scope,
                domain: domain.to_string(),
                name: name.to_string(),
            };
            if !seen.insert((scope_key(scope), id.domain.clone(), id.name.clone())) {
                continue;
            }

            let suffix = format!("capabilities/{}/{}/{}.nix", scope_dir(scope), domain, name);
            let module_path = index
                .modules
                .iter()
                .find(|m| m.id.kind == ModuleKind::Capability && m.path.ends_with(&suffix))
                .map(|m| m.path.clone())
                .unwrap_or_default();

            let mut requesters: Vec<String> = prov
                .definitions
                .iter()
                .filter_map(|d| match &d.value {
                    Some(Value::Array(items)) => Some(
                        items
                            .iter()
                            .filter_map(|i| i.as_str().map(String::from))
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                })
                .flatten()
                .collect();
            for r in &index.requests {
                if r.capability == id {
                    requesters.extend(r.requesters.iter().cloned());
                }
            }
            requesters.sort();
            requesters.dedup();

            let owned_options = index
                .provenance
                .iter()
                .filter(|(_, p)| {
                    p.definitions.iter().any(|d| {
                        d.source.file == module_path
                            && (d.layer == Layer::Capability || d.layer == Layer::Program)
                    })
                })
                .map(|(k, _)| k.clone())
                .collect();

            let ownership_status = self.owners(index, path).ok().map(|o| o.status);

            let active = !requesters.is_empty();
            out.push(CapabilityView {
                request_path: id.request_path(),
                id,
                module_path,
                requesters,
                active,
                owned_options,
                ownership_status,
            });
        }

        out.sort_by(|a, b| {
            (scope_key(a.id.scope), &a.id.domain, &a.id.name).cmp(&(
                scope_key(b.id.scope),
                &b.id.domain,
                &b.id.name,
            ))
        });
        out
    }

    pub fn option_view(&self, index: &RepositoryIndex, path: &str) -> Result<OptionView, String> {
        let provenance = index.provenance_for(path).cloned();
        if provenance.is_none() && index.option(path).is_none() {
            return Err(format!("unknown option '{path}'"));
        }
        let effective = provenance
            .as_ref()
            .and_then(|p| p.effective_value.clone())
            .map(|v| v.0)
            .or_else(|| {
                index
                    .effective_value(path)
                    .and_then(|e| e.value.clone().map(|v| v.0))
            });
        let owner = self.owners(index, path).ok();
        Ok(OptionView {
            path: path.to_string(),
            schema: index.option(path).cloned(),
            effective,
            provenance,
            owner,
        })
    }

    pub fn provenance_view(
        &self,
        index: &RepositoryIndex,
        path: &str,
    ) -> Result<ProvenanceView, String> {
        let prov = index
            .provenance_for(path)
            .ok_or_else(|| format!("no provenance for '{path}'"))?;
        Ok(project_provenance(prov))
    }

    pub fn graph_view(&self, index: &RepositoryIndex) -> GraphView {
        graph_view(&self.graph(index))
    }

    pub fn doctor_view(&self, index: &RepositoryIndex) -> DoctorView {
        doctor_view(&self.diagnose(index))
    }

    pub fn search(&self, index: &RepositoryIndex, query: &str) -> Vec<SearchHit> {
        let q = query.to_lowercase();
        let mut hits: Vec<SearchHit> = Vec::new();
        let matches = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);

        for h in &index.hosts {
            if matches(&h.name) {
                hits.push(SearchHit {
                    kind: "host".into(),
                    name: h.name.clone(),
                    detail: format!("{:?}", h.class),
                });
            }
        }
        for u in &index.users {
            if matches(&u.username) {
                hits.push(SearchHit {
                    kind: "user".into(),
                    name: u.username.clone(),
                    detail: u.full_name.clone(),
                });
            }
        }
        for p in &index.profiles {
            if matches(&p.id.name) {
                hits.push(SearchHit {
                    kind: "profile".into(),
                    name: p.id.name.clone(),
                    detail: p.source_file.clone(),
                });
            }
        }
        for a in self.applications_view(index) {
            if matches(&a.id.name) || matches(&a.option_path) {
                hits.push(SearchHit {
                    kind: "application".into(),
                    name: a.id.name.clone(),
                    detail: a.option_path,
                });
            }
        }
        for c in self.capabilities_view(index) {
            let label = format!("{}.{}", c.id.domain, c.id.name);
            if matches(&label) || matches(&c.request_path) {
                hits.push(SearchHit {
                    kind: "capability".into(),
                    name: label,
                    detail: c.request_path,
                });
            }
        }
        for path in index.provenance.keys() {
            if matches(path) {
                hits.push(SearchHit {
                    kind: "option".into(),
                    name: path.clone(),
                    detail: String::new(),
                });
            }
        }
        hits.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
        hits.dedup_by(|a, b| a.kind == b.kind && a.name == b.name);
        hits
    }
}

fn project_provenance(prov: &Provenance) -> ProvenanceView {
    ProvenanceView {
        option_path: prov.option_path.clone(),
        effective_value: prov.effective_value.clone().map(|v| v.0),
        definitions: prov.definitions.iter().map(definition_view).collect(),
        winning: prov.winning_definition().map(definition_view),
        overridden: prov
            .overridden_definitions()
            .iter()
            .copied()
            .map(definition_view)
            .collect(),
        contributions: prov
            .contributions()
            .into_iter()
            .map(contribution_view)
            .collect(),
    }
}

fn definition_view(d: &avalanche_model::OptionDefinition) -> DefinitionView {
    DefinitionView {
        file: d.source.file.clone(),
        line: d.source.span.start.line,
        column: d.source.span.start.column,
        end_line: d.source.span.end.line,
        value: d.value.clone(),
        layer: d.layer.clone(),
        priority: d.priority.0,
        is_default: d.priority.is_default(),
        scope: d.scope,
        host: d.host.clone(),
        user: d.user.clone(),
        profile: d.profile.clone(),
        condition: d.condition.as_ref().map(condition_view),
        definition_type: d.definition_type.clone(),
    }
}

fn contribution_view(c: avalanche_model::Contribution) -> ContributionView {
    let file = c.source.as_ref().map(|s| s.file.clone());
    let line = c.source.as_ref().map(|s| s.span.start.line);
    ContributionView {
        value: c.value.0,
        file,
        line,
        layer: c.layer,
        priority: c.priority.map(|p| p.0),
        definition_type: c.definition_type,
        host: c.host,
        user: c.user,
        profile: c.profile,
    }
}

fn request_map(index: &RepositoryIndex) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for (path, prov) in &index.provenance {
        if let Some(cap) = require_parts(path).map(|(_, rest)| rest.to_string()) {
            for d in &prov.definitions {
                if let Some(Value::Array(items)) = &d.value {
                    for it in items {
                        if let Some(r) = it.as_str() {
                            m.entry(r.to_string()).or_default().push(cap.clone());
                        }
                    }
                }
            }
        }
    }
    for r in &index.requests {
        let cap = format!("{}.{}", r.capability.domain, r.capability.name);
        for rq in &r.requesters {
            m.entry(rq.clone()).or_default().push(cap.clone());
        }
    }
    for v in m.values_mut() {
        v.sort();
        v.dedup();
    }
    m
}

fn require_parts(path: &str) -> Option<(Scope, &str)> {
    if let Some(r) = path.strip_prefix("gb.requires.system.") {
        if r.contains('.') {
            return Some((Scope::System, r));
        }
    }
    if let Some(r) = path.strip_prefix("gb.requires.home.") {
        if r.contains('.') {
            return Some((Scope::Home, r));
        }
    }
    None
}

fn split_cap(rest: &str) -> (&str, &str) {
    match rest.rsplit_once('.') {
        Some((d, n)) => (d, n),
        None => ("", rest),
    }
}

fn scope_dir(scope: Scope) -> &'static str {
    match scope {
        Scope::System => "system",
        Scope::Home => "home",
    }
}

fn scope_key(scope: Scope) -> u8 {
    match scope {
        Scope::System => 0,
        Scope::Home => 1,
    }
}

fn host_view(h: &Host) -> HostView {
    HostView {
        name: h.name.clone(),
        system: h.system.clone(),
        class: h.class.clone(),
        roles: h.roles.clone(),
        desktop: h.desktop.clone(),
        shell: h.shell.clone(),
        state_version: h.state_version.clone(),
        hardware: HardwareView {
            cpu: h.hardware.cpu.clone(),
            gpu: h.hardware.gpu.clone(),
            has_battery: h.hardware.has_battery,
            has_bluetooth: h.hardware.has_bluetooth,
            has_touchpad: h.hardware.has_touchpad,
            has_printer: h.hardware.has_printer,
        },
    }
}

fn user_view(u: &User) -> UserView {
    UserView {
        username: u.username.clone(),
        full_name: u.full_name.clone(),
        email: u.email.clone(),
        is_main: u.is_main,
    }
}

fn condition_view(c: &Condition) -> ConditionView {
    ConditionView {
        expression: c.expression.clone(),
        evaluated_value: c.evaluated_value.clone(),
        satisfied: c.satisfied,
    }
}

fn graph_view(graph: &RequestGraph) -> GraphView {
    let mut nodes: Vec<NodeView> = graph
        .nodes
        .keys()
        .map(|n| NodeView {
            id: node_id_str(n),
            kind: n.kind.clone(),
            label: n.name.clone(),
        })
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let edges = graph
        .edges
        .iter()
        .map(|e| EdgeView {
            from: node_id_str(&e.from),
            to: node_id_str(&e.to),
            kind: e.kind.clone(),
        })
        .collect();
    GraphView { nodes, edges }
}

fn node_id_str(n: &NodeId) -> String {
    format!("{}:{}", kind_str(&n.kind), n.name)
}

fn kind_str(k: &NodeKind) -> &'static str {
    match k {
        NodeKind::Profile => "profile",
        NodeKind::Capability => "capability",
        NodeKind::Program => "program",
        NodeKind::Module => "module",
        NodeKind::Option => "option",
        NodeKind::Host => "host",
        NodeKind::User => "user",
        NodeKind::Desktop => "desktop",
        NodeKind::Aggregate => "aggregate",
        NodeKind::Core => "core",
    }
}

fn doctor_view(diag: &Diagnosis) -> DoctorView {
    DoctorView {
        findings: diag
            .findings
            .iter()
            .map(|f| FindingView {
                severity: f.severity,
                message: f.message.clone(),
                path: f.path.clone(),
                file: f.file.clone(),
            })
            .collect(),
        repairs: diag
            .repairs
            .iter()
            .map(|r| RepairView {
                action: repair_action_str(&r.action),
                description: r.description.clone(),
                confidence: r.confidence,
            })
            .collect(),
        tier: diag.tier,
        error_count: diag.error_count(),
        warning_count: diag.warning_count(),
    }
}

fn repair_action_str(action: &RepairAction) -> String {
    match action {
        RepairAction::AddAggregateImport { module_path } => {
            format!("add-aggregate-import:{module_path}")
        }
        RepairAction::RemoveOrphanModule { module_path } => {
            format!("remove-orphan-module:{module_path}")
        }
        RepairAction::FixRequesterString { path, requester } => {
            format!("fix-requester:{path}:{requester}")
        }
        RepairAction::AddSchemaVersion { version } => format!("add-schema-version:{version}"),
        RepairAction::ResolveAmbiguousOwnership { option_path } => {
            format!("resolve-ambiguous-ownership:{option_path}")
        }
        RepairAction::DeleteStaleDefinition { option_path, file } => {
            format!("delete-stale-definition:{option_path}:{file}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_model::source::{Position, SourceSpan};
    use avalanche_model::{Cpu, Desktop, Gpu, Hardware, HostClass, HostRole, Shell};

    fn sample_host() -> Host {
        Host {
            name: "pc".into(),
            system: "x86_64-linux".into(),
            class: HostClass::Desktop,
            roles: vec![HostRole::Workstation],
            desktop: Desktop::Mango,
            shell: Shell::Zsh,
            state_version: "24.11".into(),
            hardware: Hardware {
                cpu: Cpu::Amd,
                gpu: Gpu::Amd,
                has_battery: false,
                has_bluetooth: true,
                has_touchpad: false,
                has_printer: false,
            },
        }
    }

    #[test]
    fn host_view_serializes_camel_case() {
        let v = serde_json::to_value(host_view(&sample_host())).unwrap();
        assert_eq!(v["stateVersion"], "24.11");
        assert_eq!(v["hardware"]["hasBattery"], false);
        assert_eq!(v["hardware"]["hasBluetooth"], true);
        assert!(v.get("state_version").is_none());
    }

    #[test]
    fn user_view_serializes_camel_case() {
        let u = User {
            username: "alice".into(),
            full_name: "Alice".into(),
            email: "a@b.c".into(),
            is_main: true,
        };
        let v = serde_json::to_value(user_view(&u)).unwrap();
        assert_eq!(v["fullName"], "Alice");
        assert_eq!(v["isMain"], true);
    }

    #[test]
    fn graph_view_projects_map_keys_to_arrays() {
        let mut g = RequestGraph::new();
        g.add_node(
            NodeId::new(NodeKind::Capability, "audio.pipewire"),
            "pipewire",
        );
        g.add_node(NodeId::new(NodeKind::Program, "steam"), "steam");
        g.add_edge(
            NodeId::new(NodeKind::Program, "steam"),
            NodeId::new(NodeKind::Capability, "audio.pipewire"),
            EdgeKind::Requests,
        );
        let v = serde_json::to_value(graph_view(&g)).unwrap();
        assert!(v["nodes"].is_array());
        assert_eq!(v["edges"].as_array().unwrap().len(), 1);
        assert_eq!(v["edges"][0]["kind"], "requests");
        assert_eq!(v["nodes"][0]["id"], "capability:audio.pipewire");
    }

    #[test]
    fn doctor_view_serializes_camel_case() {
        let v = serde_json::to_value(doctor_view(&Diagnosis::new())).unwrap();
        assert_eq!(v["errorCount"], 0);
        assert_eq!(v["warningCount"], 0);
        assert!(v["findings"].is_array());
        assert!(v["repairs"].is_array());
    }

    #[test]
    fn definition_condition_is_camel_case() {
        let d = avalanche_model::OptionDefinition {
            option_path: "services.x.enable".into(),
            value: None,
            source: avalanche_model::SourceLocation::new(
                "m.nix",
                SourceSpan::new(
                    Position { line: 1, column: 1 },
                    Position { line: 1, column: 2 },
                ),
            ),
            priority: avalanche_model::Priority::normal(),
            layer: Layer::Host,
            scope: Scope::System,
            host: None,
            user: None,
            profile: None,
            condition: Some(Condition {
                expression: "cfg.x".into(),
                evaluated_value: Some(NixValue::Bool(true)),
                satisfied: Some(true),
            }),
            definition_type: DefinitionType::Literal,
        };
        let v = serde_json::to_value(definition_view(&d)).unwrap();
        assert_eq!(v["condition"]["expression"], "cfg.x");
        assert_eq!(v["condition"]["satisfied"], true);
        assert!(v["condition"].get("evaluated_value").is_none());
    }
}
