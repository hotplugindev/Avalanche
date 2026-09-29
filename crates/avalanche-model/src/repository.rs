use serde::{Deserialize, Serialize};

use crate::host::Host;
use crate::module::Module;
use crate::profile::Profile;
use crate::user::User;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub git_head: Option<String>,
    pub working_tree_hash: Option<String>,
    pub flake_lock_hash: Option<String>,
    pub schema_version: Option<u32>,
    pub index_schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub root: String,
    pub schema_version: Option<u32>,
    pub hosts: Vec<Host>,
    pub users: Vec<User>,
    pub profiles: Vec<Profile>,
    pub modules: Vec<Module>,
    pub fingerprint: Fingerprint,
}

impl Repository {
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            schema_version: None,
            hosts: Vec::new(),
            users: Vec::new(),
            profiles: Vec::new(),
            modules: Vec::new(),
            fingerprint: Fingerprint {
                git_head: None,
                working_tree_hash: None,
                flake_lock_hash: None,
                schema_version: None,
                index_schema_version: 1,
            },
        }
    }
}
