use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn owners(&self, option: &NodeId) -> Vec<&NodeId> {
        self.edges
            .iter()
            .filter(|e| e.to == *option && e.kind == EdgeKind::Owns)
            .map(|e| &e.from)
            .collect()
    }

    pub fn why(&self, node: &NodeId) -> Vec<Vec<NodeId>> {
        let mut paths = Vec::new();
        self.why_walk(node, Vec::new(), &mut paths);
        paths
    }

    fn why_walk(&self, node: &NodeId, current: Vec<NodeId>, paths: &mut Vec<Vec<NodeId>>) {
        let requesters = self.requesters(node);
        if requesters.is_empty() {
            if !current.is_empty() {
                let mut path = current.clone();
                path.push(node.clone());
                paths.push(path);
            }
            return;
        }
        for requester in requesters {
            let mut next = current.clone();
            next.push(node.clone());
            self.why_walk(requester, next, paths);
        }
    }
}

pub struct GraphService {
    graph: RequestGraph,
}

impl GraphService {
    pub fn new() -> Self {
        Self {
            graph: RequestGraph::new(),
        }
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
}
