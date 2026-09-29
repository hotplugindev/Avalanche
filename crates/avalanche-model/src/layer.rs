use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    Profile,
    Host,
    User,
    Capability,
    Program,
    Desktop,
    External,
}

pub const PRIORITY_OVERRIDE: i16 = 50;
pub const PRIORITY_FORCE: i16 = 50;
pub const PRIORITY_NORMAL: i16 = 1000;
pub const PRIORITY_DEFAULT: i16 = 1500;
pub const PRIORITY_VM_OVERRIDE: i16 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Priority(pub i16);

impl Priority {
    pub fn normal() -> Self {
        Self(PRIORITY_NORMAL)
    }

    pub fn mk_default() -> Self {
        Self(PRIORITY_DEFAULT)
    }

    pub fn mk_force() -> Self {
        Self(PRIORITY_FORCE)
    }

    pub fn mk_override() -> Self {
        Self(PRIORITY_OVERRIDE)
    }

    pub fn is_default(&self) -> bool {
        self.0 == PRIORITY_DEFAULT
    }

    pub fn wins_over(&self, other: &Priority) -> bool {
        self.0 < other.0
    }
}
