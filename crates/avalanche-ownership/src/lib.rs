use std::collections::{HashMap, HashSet};

use avalanche_model::{Layer, Module, ModuleKind, Provenance, ResolvedRequest};
use serde::{Deserialize, Serialize};

pub use avalanche_model::OwnershipStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnershipRecord {
    pub option_path: String,
    pub owner_module: String,
    pub owner_kind: ModuleKind,
    pub status: OwnershipStatus,
    pub contributing_files: Vec<String>,
}

impl OwnershipRecord {
    pub fn can_auto_edit(&self) -> bool {
        self.status == OwnershipStatus::Owned
    }

    pub fn is_external(&self) -> bool {
        self.status == OwnershipStatus::External
    }

    pub fn is_ambiguous(&self) -> bool {
        self.status == OwnershipStatus::Ambiguous
    }
}

#[derive(Debug, Clone, Default)]
pub struct OwnershipRegistry {
    records: HashMap<String, OwnershipRecord>,
    requesters: HashMap<String, Vec<String>>,
    implementations: HashMap<String, Vec<String>>,
}

impl OwnershipRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(&self, option_path: &str) -> OwnershipStatus {
        self.records
            .get(option_path)
            .map(|r| r.status)
            .unwrap_or(OwnershipStatus::Unowned)
    }

    pub fn record_for(&self, option_path: &str) -> Option<&OwnershipRecord> {
        self.records.get(option_path)
    }

    pub fn owner_of(&self, option_path: &str) -> Option<&str> {
        self.records
            .get(option_path)
            .filter(|r| r.status == OwnershipStatus::Owned)
            .map(|r| r.owner_module.as_str())
    }

    pub fn requesters_of(&self, option_path: &str) -> &[String] {
        self.requesters
            .get(option_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn implementations_of(&self, option_path: &str) -> &[String] {
        self.implementations
            .get(option_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn can_auto_edit(&self, option_path: &str) -> bool {
        matches!(self.resolve(option_path), OwnershipStatus::Owned)
    }

    pub fn all_records(&self) -> impl Iterator<Item = (&String, &OwnershipRecord)> {
        self.records.iter()
    }

    pub fn ambiguous_options(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter(|(_, r)| r.status == OwnershipStatus::Ambiguous)
            .map(|(p, _)| p.as_str())
            .collect()
    }

    pub fn external_options(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter(|(_, r)| r.status == OwnershipStatus::External)
            .map(|(p, _)| p.as_str())
            .collect()
    }

    pub fn unowned_options(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter(|(_, r)| r.status == OwnershipStatus::Unowned)
            .map(|(p, _)| p.as_str())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

pub fn build_ownership_registry(
    provenance: &HashMap<String, Provenance>,
    modules: &[Module],
    requests: &[ResolvedRequest],
) -> OwnershipRegistry {
    let module_by_path: HashMap<&str, &Module> =
        modules.iter().map(|m| (m.path.as_str(), m)).collect();

    let mut registry = OwnershipRegistry::default();

    for (option_path, prov) in provenance {
        let owners = collect_owners(prov, &module_by_path);
        let record = classify_ownership(option_path, prov, &owners);
        registry.records.insert(option_path.clone(), record);
    }

    for req in requests {
        let request_path = req.capability.request_path();
        registry
            .requesters
            .entry(request_path)
            .or_default()
            .extend(req.requesters.iter().cloned());
    }

    for (option_path, prov) in provenance {
        let impls: Vec<String> = prov
            .definitions
            .iter()
            .filter(|d| d.layer == Layer::Capability || d.layer == Layer::Program)
            .map(|d| d.source.file.clone())
            .collect();
        if !impls.is_empty() {
            registry.implementations.insert(option_path.clone(), impls);
        }
    }

    registry
}

fn collect_owners<'a>(
    prov: &Provenance,
    module_by_path: &HashMap<&'a str, &'a Module>,
) -> Vec<(&'a Module, Vec<String>)> {
    let mut seen: HashMap<&str, Vec<String>> = HashMap::new();

    for def in &prov.definitions {
        let file = def.source.file.as_str();
        if let Some(module) = module_by_path.get(file) {
            if matches!(module.id.kind, ModuleKind::Capability | ModuleKind::Program) {
                seen.entry(module.path.as_str())
                    .or_default()
                    .push(file.to_string());
            }
        }
    }

    seen.into_iter()
        .filter_map(|(path, files)| module_by_path.get(path).map(|m| (*m, files)))
        .collect()
}

fn classify_ownership(
    option_path: &str,
    prov: &Provenance,
    owners: &[(&Module, Vec<String>)],
) -> OwnershipRecord {
    let contributing_files: Vec<String> = prov
        .definitions
        .iter()
        .map(|d| d.source.file.clone())
        .collect();

    if owners.is_empty() {
        let all_external = prov.definitions.iter().all(|d| d.layer == Layer::External);
        if all_external && !prov.definitions.is_empty() {
            return OwnershipRecord {
                option_path: option_path.to_string(),
                owner_module: "external".to_string(),
                owner_kind: ModuleKind::Core,
                status: OwnershipStatus::External,
                contributing_files,
            };
        }
        return OwnershipRecord {
            option_path: option_path.to_string(),
            owner_module: String::new(),
            owner_kind: ModuleKind::Core,
            status: OwnershipStatus::Unowned,
            contributing_files,
        };
    }

    let unique_owner_paths: HashSet<&str> = owners.iter().map(|(m, _)| m.path.as_str()).collect();

    if unique_owner_paths.len() > 1 {
        return OwnershipRecord {
            option_path: option_path.to_string(),
            owner_module: unique_owner_paths.into_iter().collect::<Vec<_>>().join(","),
            owner_kind: owners[0].0.id.kind.clone(),
            status: OwnershipStatus::Ambiguous,
            contributing_files,
        };
    }

    let (owner, _) = owners[0];

    let has_wrong_layer = prov.definitions.iter().any(|d| {
        (d.layer == Layer::Capability || d.layer == Layer::Program) && d.source.file != owner.path
    });

    if has_wrong_layer {
        return OwnershipRecord {
            option_path: option_path.to_string(),
            owner_module: owner.path.clone(),
            owner_kind: owner.id.kind.clone(),
            status: OwnershipStatus::WrongOwner,
            contributing_files,
        };
    }

    OwnershipRecord {
        option_path: option_path.to_string(),
        owner_module: owner.path.clone(),
        owner_kind: owner.id.kind.clone(),
        status: OwnershipStatus::Owned,
        contributing_files,
    }
}

pub struct OwnershipService {
    registry: OwnershipRegistry,
}

impl Default for OwnershipService {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnershipService {
    pub fn new() -> Self {
        Self {
            registry: OwnershipRegistry::new(),
        }
    }

    pub fn with_registry(registry: OwnershipRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &OwnershipRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut OwnershipRegistry {
        &mut self.registry
    }

    pub fn resolve(&self, option_path: &str) -> OwnershipStatus {
        self.registry.resolve(option_path)
    }

    pub fn owner_of(&self, option_path: &str) -> Option<&str> {
        self.registry.owner_of(option_path)
    }

    pub fn can_auto_edit(&self, option_path: &str) -> bool {
        self.registry.can_auto_edit(option_path)
    }

    pub fn requesters_of(&self, option_path: &str) -> &[String] {
        self.registry.requesters_of(option_path)
    }

    pub fn implementations_of(&self, option_path: &str) -> &[String] {
        self.registry.implementations_of(option_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_model::source::{Position, SourceLocation, SourceSpan};
    use avalanche_model::{DefinitionType, OptionDefinition, Priority, Scope};

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

    fn program_module(path: &str, name: &str) -> Module {
        Module {
            id: avalanche_model::ModuleId {
                kind: ModuleKind::Program,
                scope: Some(Scope::Home),
                name: name.to_string(),
            },
            path: path.to_string(),
            imported: true,
        }
    }

    #[test]
    fn single_capability_owner_is_owned() {
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

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("services.pipewire.enable"),
            OwnershipStatus::Owned
        );
        assert_eq!(
            registry.owner_of("services.pipewire.enable"),
            Some("modules/capabilities/system/audio/pipewire.nix")
        );
        assert!(registry.can_auto_edit("services.pipewire.enable"));
    }

    #[test]
    fn two_capability_owners_is_ambiguous() {
        let mut prov = Provenance::new("services.xserver.enable");
        prov.add(make_def(
            "services.xserver.enable",
            "modules/capabilities/system/desktop/x11.nix",
            Layer::Capability,
        ));
        prov.add(make_def(
            "services.xserver.enable",
            "modules/capabilities/system/graphics/nvidia.nix",
            Layer::Capability,
        ));

        let modules = vec![
            capability_module("modules/capabilities/system/desktop/x11.nix", "desktop.x11"),
            capability_module(
                "modules/capabilities/system/graphics/nvidia.nix",
                "graphics.nvidia",
            ),
        ];

        let mut provenance = HashMap::new();
        provenance.insert("services.xserver.enable".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("services.xserver.enable"),
            OwnershipStatus::Ambiguous
        );
        assert!(!registry.can_auto_edit("services.xserver.enable"));
        assert_eq!(
            registry.ambiguous_options(),
            vec!["services.xserver.enable"]
        );
    }

    #[test]
    fn external_layer_is_external() {
        let mut prov = Provenance::new("programs.firefox.enable");
        prov.add(make_def(
            "programs.firefox.enable",
            "nixpkgs/modules/firefox.nix",
            Layer::External,
        ));

        let modules: Vec<Module> = vec![];
        let mut provenance = HashMap::new();
        provenance.insert("programs.firefox.enable".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("programs.firefox.enable"),
            OwnershipStatus::External
        );
        assert!(!registry.can_auto_edit("programs.firefox.enable"));
        assert_eq!(registry.external_options(), vec!["programs.firefox.enable"]);
    }

    #[test]
    fn host_override_does_not_change_ownership() {
        let mut prov = Provenance::new("services.pipewire.enable");
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/capabilities/system/audio/pipewire.nix",
            Layer::Capability,
        ));
        prov.add(make_def(
            "services.pipewire.enable",
            "hosts/pc/default.nix",
            Layer::Host,
        ));

        let modules = vec![capability_module(
            "modules/capabilities/system/audio/pipewire.nix",
            "audio.pipewire",
        )];

        let mut provenance = HashMap::new();
        provenance.insert("services.pipewire.enable".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("services.pipewire.enable"),
            OwnershipStatus::Owned
        );
        assert_eq!(
            registry.owner_of("services.pipewire.enable"),
            Some("modules/capabilities/system/audio/pipewire.nix")
        );
    }

    #[test]
    fn profile_only_is_unowned() {
        let mut prov = Provenance::new("networking.hostName");
        prov.add(make_def(
            "networking.hostName",
            "modules/profiles/system/workstation.nix",
            Layer::Profile,
        ));

        let modules = vec![Module {
            id: avalanche_model::ModuleId {
                kind: ModuleKind::Profile,
                scope: Some(Scope::System),
                name: "workstation".to_string(),
            },
            path: "modules/profiles/system/workstation.nix".to_string(),
            imported: true,
        }];

        let mut provenance = HashMap::new();
        provenance.insert("networking.hostName".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("networking.hostName"),
            OwnershipStatus::Unowned
        );
    }

    #[test]
    fn wrong_owner_when_capability_value_comes_from_unregistered_file() {
        let mut prov = Provenance::new("services.pipewire.enable");
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/capabilities/system/audio/pipewire.nix",
            Layer::Capability,
        ));
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/core/misc/leak.nix",
            Layer::Capability,
        ));

        let modules = vec![capability_module(
            "modules/capabilities/system/audio/pipewire.nix",
            "audio.pipewire",
        )];

        let mut provenance = HashMap::new();
        provenance.insert("services.pipewire.enable".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("services.pipewire.enable"),
            OwnershipStatus::WrongOwner
        );
        assert!(!registry.can_auto_edit("services.pipewire.enable"));
    }

    #[test]
    fn program_module_owns_options() {
        let mut prov = Provenance::new("programs.git.enable");
        prov.add(make_def(
            "programs.git.enable",
            "modules/programs/home/git/default.nix",
            Layer::Program,
        ));

        let modules = vec![program_module(
            "modules/programs/home/git/default.nix",
            "git",
        )];

        let mut provenance = HashMap::new();
        provenance.insert("programs.git.enable".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("programs.git.enable"),
            OwnershipStatus::Owned
        );
        assert_eq!(
            registry.owner_of("programs.git.enable"),
            Some("modules/programs/home/git/default.nix")
        );
    }

    #[test]
    fn requesters_populated_from_resolved_requests() {
        let mut prov = Provenance::new("gb.requires.system.audio.pipewire");
        prov.add(make_def(
            "gb.requires.system.audio.pipewire",
            "modules/profiles/system/workstation.nix",
            Layer::Profile,
        ));

        let modules = vec![capability_module(
            "modules/capabilities/system/audio/pipewire.nix",
            "audio.pipewire",
        )];

        let requests = vec![ResolvedRequest {
            capability: avalanche_model::CapabilityId {
                scope: Scope::System,
                domain: "audio".to_string(),
                name: "pipewire".to_string(),
            },
            requesters: vec!["profiles.system.workstation".to_string()],
            active: true,
        }];

        let mut provenance = HashMap::new();
        provenance.insert("gb.requires.system.audio.pipewire".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &requests);
        let requesters = registry.requesters_of("gb.requires.system.audio.pipewire");
        assert_eq!(requesters, &["profiles.system.workstation"]);
    }

    #[test]
    fn implementations_tracked_for_capability_options() {
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

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        let impls = registry.implementations_of("services.pipewire.enable");
        assert_eq!(impls, &["modules/capabilities/system/audio/pipewire.nix"]);
    }

    #[test]
    fn unowned_options_listed() {
        let mut prov = Provenance::new("some.random.option");
        prov.add(make_def(
            "some.random.option",
            "hosts/pc/default.nix",
            Layer::Host,
        ));

        let modules: Vec<Module> = vec![];
        let mut provenance = HashMap::new();
        provenance.insert("some.random.option".to_string(), prov);

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("some.random.option"),
            OwnershipStatus::Unowned
        );
        assert!(registry.unowned_options().contains(&"some.random.option"));
    }

    #[test]
    fn ownership_service_delegates() {
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

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        let svc = OwnershipService::with_registry(registry);

        assert_eq!(
            svc.resolve("services.pipewire.enable"),
            OwnershipStatus::Owned
        );
        assert!(svc.can_auto_edit("services.pipewire.enable"));
        assert_eq!(
            svc.owner_of("services.pipewire.enable"),
            Some("modules/capabilities/system/audio/pipewire.nix")
        );
    }

    #[test]
    fn duplicate_registration_same_module_stays_owned() {
        let mut prov = Provenance::new("services.pipewire.enable");
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/capabilities/system/audio/pipewire.nix",
            Layer::Capability,
        ));
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

        let registry = build_ownership_registry(&provenance, &modules, &[]);
        assert_eq!(
            registry.resolve("services.pipewire.enable"),
            OwnershipStatus::Owned
        );
    }

    #[test]
    fn empty_registry_is_unowned() {
        let registry = OwnershipRegistry::new();
        assert_eq!(registry.resolve("anything"), OwnershipStatus::Unowned);
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
