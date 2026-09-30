use serde::{Deserialize, Serialize};

use crate::error::ModelError;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequesterKind {
    Program,
    Profile,
    Capability,
    Host,
    User,
}

impl RequesterKind {
    pub fn plural(self) -> &'static str {
        match self {
            RequesterKind::Program => "programs",
            RequesterKind::Profile => "profiles",
            RequesterKind::Capability => "capabilities",
            RequesterKind::Host => "hosts",
            RequesterKind::User => "users",
        }
    }

    fn from_segment(seg: &str) -> Option<Self> {
        match seg {
            "programs" => Some(RequesterKind::Program),
            "profiles" => Some(RequesterKind::Profile),
            "capabilities" => Some(RequesterKind::Capability),
            "hosts" => Some(RequesterKind::Host),
            "users" => Some(RequesterKind::User),
            _ => None,
        }
    }

    fn is_scoped(self) -> bool {
        matches!(
            self,
            RequesterKind::Program | RequesterKind::Profile | RequesterKind::Capability
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamespaceKind {
    Meta,
    Facts,
    Identity,
    Request,
    Application,
    Desktop,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceContract {
    pub path: String,
    pub scope: String,
    pub kind: NamespaceKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryContract {
    pub schema_version: u32,
    pub namespaces: Vec<NamespaceContract>,
    pub requester_kinds: Vec<String>,
    pub ownership_inferred_from_filename: bool,
}

impl RepositoryContract {
    pub fn v1() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            namespaces: vec![
                ns("gb.schemaVersion", "both", NamespaceKind::Meta),
                ns("gb.host", "system", NamespaceKind::Facts),
                ns("gb.user", "both", NamespaceKind::Identity),
                ns("gb.requires.system", "system", NamespaceKind::Request),
                ns("gb.requires.home", "home", NamespaceKind::Request),
                ns("gb.programs.system", "system", NamespaceKind::Application),
                ns("gb.home.programs", "home", NamespaceKind::Application),
                ns("gb.home.desktop", "home", NamespaceKind::Desktop),
                ns("gb.debug", "system", NamespaceKind::Debug),
            ],
            requester_kinds: vec![
                "programs".into(),
                "profiles".into(),
                "capabilities".into(),
                "hosts".into(),
                "users".into(),
            ],
            ownership_inferred_from_filename: false,
        }
    }

    pub fn covers_path(&self, option_path: &str) -> bool {
        self.namespaces.iter().any(|n| {
            option_path == n.path || option_path.starts_with(&format!("{}.", n.path))
        })
    }

    pub fn validate_requester(&self, requester: &str) -> Result<RequesterKind, ModelError> {
        parse_requester(requester)
    }
}

fn ns(path: &str, scope: &str, kind: NamespaceKind) -> NamespaceContract {
    NamespaceContract {
        path: path.to_string(),
        scope: scope.to_string(),
        kind,
        description: String::new(),
    }
}

pub fn parse_requester(requester: &str) -> Result<RequesterKind, ModelError> {
    let segs: Vec<&str> = requester.split('.').filter(|s| !s.is_empty()).collect();
    if segs.is_empty() {
        return Err(ModelError::Invalid("empty requester".into()));
    }

    let kind = RequesterKind::from_segment(segs[0]).ok_or_else(|| {
        ModelError::Invalid(format!("unknown requester kind: {}", segs[0]))
    })?;

    if kind.is_scoped() {
        if segs.len() < 3 {
            return Err(ModelError::Invalid(format!(
                "requester '{}' missing scope or name",
                requester
            )));
        }
        if segs[1] != "system" && segs[1] != "home" {
            return Err(ModelError::Invalid(format!(
                "requester '{}' has invalid scope '{}'",
                requester, segs[1]
            )));
        }
    } else if segs.len() < 2 {
        return Err(ModelError::Invalid(format!(
            "requester '{}' missing name",
            requester
        )));
    }

    Ok(kind)
}
