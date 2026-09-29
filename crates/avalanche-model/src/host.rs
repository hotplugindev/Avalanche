use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostClass {
    Desktop,
    Laptop,
    Server,
    Vm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostRole {
    Workstation,
    Desktop,
    Gaming,
    Development,
    Server,
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Desktop {
    Mango,
    Hyprland,
    Gnome,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cpu {
    Amd,
    Intel,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gpu {
    Amd,
    Intel,
    Nvidia,
    Virtio,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hardware {
    pub cpu: Cpu,
    pub gpu: Gpu,
    pub has_battery: bool,
    pub has_bluetooth: bool,
    pub has_touchpad: bool,
    pub has_printer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub system: String,
    pub class: HostClass,
    pub roles: Vec<HostRole>,
    pub desktop: Desktop,
    pub shell: Shell,
    pub state_version: String,
    pub hardware: Hardware,
}

impl Host {
    pub fn has_role(&self, role: &HostRole) -> bool {
        self.roles.contains(role)
    }
}
