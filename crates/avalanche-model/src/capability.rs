use serde::{Deserialize, Serialize};

use crate::scope::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId {
    pub scope: Scope,
    pub domain: String,
    pub name: String,
}

impl CapabilityId {
    pub fn from_module_path(path: &str) -> Option<Self> {
        let normalized = path.replace('\\', "/");
        let rest = normalized.strip_prefix("modules/capabilities/")?;
        let (scope_str, remainder) = rest.split_once('/')?;
        let scope = match scope_str {
            "system" => Scope::System,
            "home" => Scope::Home,
            _ => return None,
        };
        let without_ext = remainder.trim_end_matches(".nix");
        let without_default = without_ext.strip_suffix("/default").unwrap_or(without_ext);
        let (domain, name) = match without_default.rsplit_once('/') {
            Some((d, n)) => (d.replace('/', "."), n.to_string()),
            None => (String::new(), without_default.to_string()),
        };
        if name.is_empty() {
            None
        } else {
            Some(Self {
                scope,
                domain,
                name,
            })
        }
    }

    pub fn request_path(&self) -> String {
        match self.scope {
            Scope::System => format!("gb.requires.system.{}.{}", self.domain, self.name),
            Scope::Home => format!("gb.requires.home.{}.{}", self.domain, self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub module_path: String,
    pub description: String,
}
