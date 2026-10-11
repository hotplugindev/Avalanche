use std::collections::{HashMap, HashSet};

use avalanche_model::{
    parse_requester, Layer, Module, ModuleKind, Provenance, RequesterKind, ResolvedRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Profile,
    Capability,
    Program,
    Module,
    Option,
    Host,
    User,
    Desktop,
    Aggregate,
    Core,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    pub kind: NodeKind,
    pub name: String,
}

impl NodeId {
    pub fn new(kind: NodeKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }

    pub fn display(&self) -> String {
        format!("{:?}:{}", self.kind, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Requests,
    Implements,
    Owns,
    Overrides,
    Imports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Default)]
pub struct RequestGraph {
    pub nodes: HashMap<NodeId, String>,
    pub edges: Vec<Edge>,
}

impl RequestGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: NodeId, label: impl Into<String>) {
        self.nodes.insert(id, label.into());
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, kind: EdgeKind) {
        self.edges.push(Edge { from, to, kind });
    }

    pub fn requesters(&self, target: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.to == *target && e.kind == EdgeKind::Requests)
            .map(|e| &e.from)
            .collect()
    }

    pub fn dependents(&self, source: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.from == *source && e.kind == EdgeKind::Requests)
            .map(|e| &e.to)
            .collect()
    }

    pub fn implementers(&self, capability: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.to == *capability && e.kind == EdgeKind::Implements)
            .map(|e| &e.from)
            .collect()
    }

    pub fn implementation(&self, option: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.to == *option && e.kind == EdgeKind::Implements)
            .map(|e| &e.from)
            .collect()
    }

    pub fn owners(&self, option: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.to == *option && e.kind == EdgeKind::Owns)
            .map(|e| &e.from)
            .collect()
    }

    pub fn overriders(&self, option: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.to == *option && e.kind == EdgeKind::Overrides)
            .map(|e| &e.from)
            .collect()
    }

    pub fn imports(&self, module: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.from == *module && e.kind == EdgeKind::Imports)
            .map(|e| &e.to)
            .collect()
    }

    pub fn why(&self, node: &NodeId) -> Vec<Vec<NodeId>> {
        let mut paths = Vec::new();
        let mut on_path = HashSet::new();
        self.why_walk(node, Vec::new(), &mut paths, &mut on_path);
        paths
    }

    fn why_walk(
        &self,
        node: &NodeId,
        current: Vec<NodeId>,
        paths: &mut Vec<Vec<NodeId>>,
        on_path: &mut HashSet<NodeId>,
    ) {
        if on_path.contains(node) {
            return;
        }
        on_path.insert(node.clone());

        let requesters = self.requesters(node);
        if requesters.is_empty() {
            if !current.is_empty() {
                let mut path = current.clone();
                path.push(node.clone());
                path.reverse();
                paths.push(path);
            }
        } else {
            for requester in requesters {
                let mut next = current.clone();
                next.push(node.clone());
                self.why_walk(requester, next, paths, on_path);
            }
        }

        on_path.remove(node);
    }
}

pub fn build_request_graph(
    provenance: &HashMap<String, Provenance>,
    requests: &[ResolvedRequest],
    modules: &[Module],
    imports: &HashMap<String, Vec<String>>,
) -> RequestGraph {
    let module_by_path: HashMap<&str, &Module> =
        modules.iter().map(|m| (m.path.as_str(), m)).collect();

    let mut graph = RequestGraph::new();

    for req in requests {
        let cap_name = format!("{}.{}", req.capability.domain, req.capability.name);
        let cap_node = NodeId::new(NodeKind::Capability, cap_name.clone());
        graph.add_node(cap_node.clone(), cap_name);

        for requester in &req.requesters {
            let requester_node = requester_node(requester);
            graph.add_node(requester_node.clone(), requester.clone());
            graph.add_edge(requester_node, cap_node.clone(), EdgeKind::Requests);
        }
    }

    for (option_path, prov) in provenance {
        if let Some(cap_name) = capability_name_from_request_path(option_path) {
            let cap_node = NodeId::new(NodeKind::Capability, cap_name.clone());
            graph.add_node(cap_node.clone(), cap_name);
            for def in &prov.definitions {
                if let Some(serde_json::Value::Array(items)) = &def.value {
                    for item in items {
                        if let Some(req) = item.as_str() {
                            let rn = requester_node(req);
                            graph.add_node(rn.clone(), req.to_string());
                            graph.add_edge(rn, cap_node.clone(), EdgeKind::Requests);
                        }
                    }
                }
            }
        }

        let option_node = NodeId::new(NodeKind::Option, option_path.clone());
        graph.add_node(option_node.clone(), option_path.clone());

        for def in &prov.definitions {
            let file = def.source.file.as_str();
            match def.layer {
                Layer::Capability | Layer::Program => {
                    let module_node = module_node_for_path(file, &module_by_path);
                    graph.add_node(module_node.clone(), file.to_string());
                    graph.add_edge(
                        module_node.clone(),
                        option_node.clone(),
                        EdgeKind::Implements,
                    );
                    graph.add_edge(module_node, option_node.clone(), EdgeKind::Owns);
                }
                Layer::Host | Layer::User => {
                    let host_node = NodeId::new(NodeKind::Module, file.to_string());
                    graph.add_node(host_node.clone(), file.to_string());
                    graph.add_edge(host_node, option_node.clone(), EdgeKind::Overrides);
                }
                _ => {}
            }
        }
    }

    for (file, imported) in imports {
        let from_node = NodeId::new(NodeKind::Module, file.clone());
        graph.add_node(from_node.clone(), file.clone());
        for imp in imported {
            let to_node = NodeId::new(NodeKind::Module, imp.clone());
            graph.add_node(to_node.clone(), imp.clone());
            graph.add_edge(from_node.clone(), to_node, EdgeKind::Imports);
        }
    }

    let mut seen: HashSet<(NodeId, NodeId, EdgeKind)> = HashSet::new();
    graph
        .edges
        .retain(|e| seen.insert((e.from.clone(), e.to.clone(), e.kind.clone())));

    graph
}

fn capability_name_from_request_path(path: &str) -> Option<String> {
    let rest = path
        .strip_prefix("gb.requires.system.")
        .or_else(|| path.strip_prefix("gb.requires.home."))?;
    if rest.is_empty() || !rest.contains('.') {
        return None;
    }
    Some(rest.to_string())
}

fn requester_node(requester: &str) -> NodeId {
    let kind = match parse_requester(requester) {
        Ok(RequesterKind::Program) => NodeKind::Program,
        Ok(RequesterKind::Profile) => NodeKind::Profile,
        Ok(RequesterKind::Capability) => NodeKind::Capability,
        Ok(RequesterKind::Host) => NodeKind::Host,
        Ok(RequesterKind::User) => NodeKind::User,
        Err(_) => NodeKind::Module,
    };
    NodeId::new(kind, requester.to_string())
}

fn module_node_for_path(path: &str, module_by_path: &HashMap<&str, &Module>) -> NodeId {
    if let Some(module) = module_by_path.get(path) {
        let kind = match module.id.kind {
            ModuleKind::Capability => NodeKind::Capability,
            ModuleKind::Program => NodeKind::Program,
            ModuleKind::Profile => NodeKind::Profile,
            ModuleKind::Desktop => NodeKind::Desktop,
            ModuleKind::Aggregate => NodeKind::Aggregate,
            ModuleKind::Core => NodeKind::Core,
        };
        return NodeId::new(kind, module.id.name.clone());
    }
    NodeId::new(NodeKind::Module, path.to_string())
}

pub struct GraphService {
    graph: RequestGraph,
}

impl Default for GraphService {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphService {
    pub fn new() -> Self {
        Self {
            graph: RequestGraph::new(),
        }
    }

    pub fn with_graph(graph: RequestGraph) -> Self {
        Self { graph }
    }

    pub fn graph(&self) -> &RequestGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut RequestGraph {
        &mut self.graph
    }

    pub fn why(&self, node: &NodeId) -> Vec<Vec<NodeId>> {
        self.graph.why(node)
    }

    pub fn requesters(&self, node: &NodeId) -> Vec<&NodeId> {
        self.graph.requesters(node)
    }

    pub fn dependents(&self, node: &NodeId) -> Vec<&NodeId> {
        self.graph.dependents(node)
    }

    pub fn owners(&self, node: &NodeId) -> Vec<&NodeId> {
        self.graph.owners(node)
    }

    pub fn implementation(&self, node: &NodeId) -> Vec<&NodeId> {
        self.graph.implementation(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_model::source::{Position, SourceLocation, SourceSpan};
    use avalanche_model::{CapabilityId, DefinitionType, OptionDefinition, Priority, Scope};

    fn span() -> SourceSpan {
        SourceSpan::new(
            Position { line: 1, column: 1 },
            Position {
                line: 1,
                column: 10,
            },
        )
    }

    fn make_def(path: &str, file: &str, layer: Layer) -> OptionDefinition {
        OptionDefinition {
            option_path: path.to_string(),
            value: None,
            source: SourceLocation::new(file, span()),
            priority: Priority::normal(),
            layer,
            scope: Scope::System,
            host: None,
            user: None,
            profile: None,
            condition: None,
            definition_type: DefinitionType::Literal,
        }
    }

    fn capability_module(path: &str, name: &str) -> Module {
        Module {
            id: avalanche_model::ModuleId {
                kind: ModuleKind::Capability,
                scope: Some(Scope::System),
                name: name.to_string(),
            },
            path: path.to_string(),
            imported: true,
        }
    }

    #[test]
    fn why_traverses_request_edges() {
        let requests = vec![ResolvedRequest {
            capability: CapabilityId {
                scope: Scope::System,
                domain: "audio".to_string(),
                name: "pipewire".to_string(),
            },
            requesters: vec!["profiles.system.workstation".to_string()],
            active: true,
        }];

        let graph = build_request_graph(&HashMap::new(), &requests, &[], &HashMap::new());
        let cap = NodeId::new(NodeKind::Capability, "audio.pipewire");
        let paths = graph.why(&cap);
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0][0],
            NodeId::new(NodeKind::Profile, "profiles.system.workstation")
        );
        assert_eq!(paths[0][1], cap);
    }

    #[test]
    fn requesters_and_dependents_are_symmetric() {
        let requests = vec![ResolvedRequest {
            capability: CapabilityId {
                scope: Scope::System,
                domain: "audio".to_string(),
                name: "pipewire".to_string(),
            },
            requesters: vec![
                "profiles.system.workstation".to_string(),
                "programs.system.steam".to_string(),
            ],
            active: true,
        }];

        let graph = build_request_graph(&HashMap::new(), &requests, &[], &HashMap::new());
        let cap = NodeId::new(NodeKind::Capability, "audio.pipewire");
        assert_eq!(graph.requesters(&cap).len(), 2);

        let profile = NodeId::new(NodeKind::Profile, "profiles.system.workstation");
        assert_eq!(graph.dependents(&profile), vec![&cap]);
    }

    #[test]
    fn implementation_and_owners_from_provenance() {
        let mut prov = Provenance::new("services.pipewire.enable");
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/capabilities/system/audio/pipewire.nix",
            Layer::Capability,
        ));

        let modules = vec![capability_module(
            "modules/capabilities/system/audio/pipewire.nix",
            "audio.pipewire",
        )];

        let mut provenance = HashMap::new();
        provenance.insert("services.pipewire.enable".to_string(), prov);

        let graph = build_request_graph(&provenance, &[], &modules, &HashMap::new());
        let option = NodeId::new(NodeKind::Option, "services.pipewire.enable");
        let owners = graph.owners(&option);
        assert_eq!(owners.len(), 1);
        assert_eq!(owners[0].kind, NodeKind::Capability);
        assert_eq!(owners[0].name, "audio.pipewire");
        assert_eq!(graph.implementation(&option).len(), 1);
    }

    #[test]
    fn host_definitions_produce_override_edges() {
        let mut prov = Provenance::new("services.openssh.enable");
        prov.add(make_def(
            "services.openssh.enable",
            "modules/capabilities/system/networking/openssh.nix",
            Layer::Capability,
        ));
        prov.add(make_def(
            "services.openssh.enable",
            "hosts/pc/default.nix",
            Layer::Host,
        ));

        let modules = vec![capability_module(
            "modules/capabilities/system/networking/openssh.nix",
            "networking.openssh",
        )];

        let mut provenance = HashMap::new();
        provenance.insert("services.openssh.enable".to_string(), prov);

        let graph = build_request_graph(&provenance, &[], &modules, &HashMap::new());
        let option = NodeId::new(NodeKind::Option, "services.openssh.enable");
        let overriders = graph.overriders(&option);
        assert_eq!(overriders.len(), 1);
        assert_eq!(overriders[0].name, "hosts/pc/default.nix");
    }

    #[test]
    fn imports_edges_recorded() {
        let mut imports = HashMap::new();
        imports.insert(
            "modules/aggregate/nixos.nix".to_string(),
            vec!["modules/capabilities/system/audio/pipewire.nix".to_string()],
        );

        let graph = build_request_graph(&HashMap::new(), &[], &[], &imports);
        let agg = NodeId::new(NodeKind::Module, "modules/aggregate/nixos.nix");
        let imported = graph.imports(&agg);
        assert_eq!(imported.len(), 1);
        assert_eq!(
            imported[0].name,
            "modules/capabilities/system/audio/pipewire.nix"
        );
    }

    #[test]
    fn unknown_requester_falls_back_to_module_node() {
        let node = requester_node("weird.thing");
        assert_eq!(node.kind, NodeKind::Module);
    }

    #[test]
    fn graph_service_delegates() {
        let requests = vec![ResolvedRequest {
            capability: CapabilityId {
                scope: Scope::System,
                domain: "audio".to_string(),
                name: "pipewire".to_string(),
            },
            requesters: vec!["profiles.system.workstation".to_string()],
            active: true,
        }];
        let graph = build_request_graph(&HashMap::new(), &requests, &[], &HashMap::new());
        let svc = GraphService::with_graph(graph);
        let cap = NodeId::new(NodeKind::Capability, "audio.pipewire");
        assert_eq!(svc.why(&cap).len(), 1);
        assert_eq!(svc.requesters(&cap).len(), 1);
    }

    #[test]
    fn requests_derived_from_provenance_without_semantic() {
        let mut prov = Provenance::new("gb.requires.system.audio.pipewire");
        let mut def = make_def(
            "gb.requires.system.audio.pipewire",
            "modules/profiles/system/workstation.nix",
            Layer::Profile,
        );
        def.value = Some(serde_json::json!(["profiles.system.workstation"]));
        prov.add(def);

        let mut provenance = HashMap::new();
        provenance.insert("gb.requires.system.audio.pipewire".to_string(), prov);

        let graph = build_request_graph(&provenance, &[], &[], &HashMap::new());
        let cap = NodeId::new(NodeKind::Capability, "audio.pipewire");
        let paths = graph.why(&cap);
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0][0],
            NodeId::new(NodeKind::Profile, "profiles.system.workstation")
        );
    }

    #[test]
    fn why_is_cycle_safe() {
        let requests = vec![
            ResolvedRequest {
                capability: CapabilityId {
                    scope: Scope::System,
                    domain: "a".into(),
                    name: "b".into(),
                },
                requesters: vec!["capabilities.system.c.d".into()],
                active: true,
            },
            ResolvedRequest {
                capability: CapabilityId {
                    scope: Scope::System,
                    domain: "c".into(),
                    name: "d".into(),
                },
                requesters: vec!["capabilities.system.a.b".into()],
                active: true,
            },
        ];
        let graph = build_request_graph(&HashMap::new(), &requests, &[], &HashMap::new());
        let cap = NodeId::new(NodeKind::Capability, "a.b");
        let paths = graph.why(&cap);
        assert!(!paths.is_empty());
    }
}
