use std::collections::{HashMap, HashSet};

use avalanche_model::validation::{ValidationFinding, ValidationSeverity, ValidationTier};
use avalanche_model::{
    parse_requester, Layer, Module, ModuleKind, OptionSchema, Provenance, Repository,
};
use avalanche_ownership::{build_ownership_registry, OwnershipStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairAction {
    AddAggregateImport { module_path: String },
    RemoveOrphanModule { module_path: String },
    FixRequesterString { path: String, requester: String },
    AddSchemaVersion { version: u32 },
    ResolveAmbiguousOwnership { option_path: String },
    DeleteStaleDefinition { option_path: String, file: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairProposal {
    pub action: RepairAction,
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnosis {
    pub findings: Vec<ValidationFinding>,
    pub repairs: Vec<RepairProposal>,
    pub tier: ValidationTier,
}

impl Default for Diagnosis {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnosis {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            repairs: Vec::new(),
            tier: ValidationTier::Structural,
        }
    }

    pub fn add(&mut self, finding: ValidationFinding) {
        self.findings.push(finding);
    }

    pub fn propose(&mut self, proposal: RepairProposal) {
        self.repairs.push(proposal);
    }

    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == ValidationSeverity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == ValidationSeverity::Warning)
            .count()
    }
}

#[derive(Debug, Clone)]
pub struct DoctorInputs {
    pub repository: Repository,
    pub provenance: HashMap<String, Provenance>,
    pub modules: Vec<Module>,
    pub imports: HashMap<String, Vec<String>>,
    pub schemas: HashMap<String, OptionSchema>,
}

pub struct DoctorService;

impl Default for DoctorService {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorService {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnose(&self, inputs: &DoctorInputs) -> Diagnosis {
        let mut diag = Diagnosis::new();

        self.check_schema(inputs, &mut diag);
        self.check_imports(inputs, &mut diag);
        self.check_aggregate_registration(inputs, &mut diag);
        self.check_orphan_modules(inputs, &mut diag);
        self.check_ownership(inputs, &mut diag);
        self.check_duplicate_ownership(inputs, &mut diag);
        self.check_path_drift(inputs, &mut diag);
        self.check_copy_paste_descriptions(inputs, &mut diag);
        self.check_empty_profiles(inputs, &mut diag);
        self.check_broken_requests(inputs, &mut diag);
        self.check_untracked_configuration(inputs, &mut diag);

        diag
    }

    fn check_schema(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        if inputs.repository.schema_version.is_none() {
            diag.add(ValidationFinding {
                severity: ValidationSeverity::Error,
                message: "repository does not declare gb.schemaVersion".to_string(),
                path: Some("gb.schemaVersion".to_string()),
                file: None,
            });
            diag.propose(RepairProposal {
                action: RepairAction::AddSchemaVersion { version: 1 },
                description: "add gb.schemaVersion = 1; to the repository root".to_string(),
                confidence: 0.95,
            });
        }

        for option_path in inputs.provenance.keys() {
            if option_path.starts_with("gb.") && !is_valid_gb_namespace(option_path) {
                diag.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: format!(
                        "option '{}' is in an undeclared gb.* namespace",
                        option_path
                    ),
                    path: Some(option_path.clone()),
                    file: None,
                });
            }
        }
    }

    fn check_imports(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        let module_paths: HashSet<&str> = inputs.modules.iter().map(|m| m.path.as_str()).collect();

        for (file, imported) in &inputs.imports {
            if !file.starts_with("modules/") {
                continue;
            }
            for imp in imported {
                let resolved = resolve_import_target(file, imp);
                if resolved.starts_with("modules/") && !module_paths.contains(resolved.as_str()) {
                    diag.add(ValidationFinding {
                        severity: ValidationSeverity::Warning,
                        message: format!(
                            "{} imports '{}' which is not a registered module",
                            file, imp
                        ),
                        path: None,
                        file: Some(file.clone()),
                    });
                }
            }
        }
    }

    fn check_aggregate_registration(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        let mut aggregated: HashSet<String> = HashSet::new();
        for (file, imps) in inputs
            .imports
            .iter()
            .filter(|(file, _)| file.contains("aggregate"))
        {
            for imp in imps {
                aggregated.insert(resolve_import_target(file, imp));
            }
        }

        for module in &inputs.modules {
            if module.id.kind == ModuleKind::Aggregate {
                continue;
            }
            if !aggregated.contains(module.path.as_str()) {
                diag.add(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    message: format!("module '{}' is not imported by any aggregate", module.path),
                    path: None,
                    file: Some(module.path.clone()),
                });
                diag.propose(RepairProposal {
                    action: RepairAction::AddAggregateImport {
                        module_path: module.path.clone(),
                    },
                    description: format!("add {} to the appropriate aggregate", module.path),
                    confidence: 0.8,
                });
            }
        }
    }

    fn check_orphan_modules(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        let defined_options: HashSet<&str> = inputs
            .provenance
            .values()
            .flat_map(|p| p.definitions.iter().map(|d| d.source.file.as_str()))
            .collect();

        for module in &inputs.modules {
            if module.id.kind == ModuleKind::Aggregate || module.id.kind == ModuleKind::Core {
                continue;
            }
            if !defined_options.contains(module.path.as_str()) {
                diag.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: format!("module '{}' defines no configuration", module.path),
                    path: None,
                    file: Some(module.path.clone()),
                });
                diag.propose(RepairProposal {
                    action: RepairAction::RemoveOrphanModule {
                        module_path: module.path.clone(),
                    },
                    description: format!("remove or populate orphan module {}", module.path),
                    confidence: 0.5,
                });
            }
        }
    }

    fn check_ownership(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        let requests: Vec<avalanche_model::ResolvedRequest> = Vec::new();
        let registry = build_ownership_registry(&inputs.provenance, &inputs.modules, &requests);

        for (option_path, record) in registry.all_records() {
            match record.status {
                OwnershipStatus::Ambiguous => {
                    diag.add(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        message: format!(
                            "option '{}' has ambiguous ownership: {}",
                            option_path, record.owner_module
                        ),
                        path: Some(option_path.clone()),
                        file: None,
                    });
                    diag.propose(RepairProposal {
                        action: RepairAction::ResolveAmbiguousOwnership {
                            option_path: option_path.clone(),
                        },
                        description: format!(
                            "assign '{}' to exactly one owning module",
                            option_path
                        ),
                        confidence: 0.6,
                    });
                }
                OwnershipStatus::WrongOwner => {
                    diag.add(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        message: format!(
                            "option '{}' is implemented by a file outside its owner '{}'",
                            option_path, record.owner_module
                        ),
                        path: Some(option_path.clone()),
                        file: None,
                    });
                }
                OwnershipStatus::Unowned => {
                    let has_impl_layer = inputs
                        .provenance
                        .get(option_path)
                        .map(|p| {
                            p.definitions
                                .iter()
                                .any(|d| d.layer == Layer::Capability || d.layer == Layer::Program)
                        })
                        .unwrap_or(false);
                    if has_impl_layer {
                        diag.add(ValidationFinding {
                            severity: ValidationSeverity::Warning,
                            message: format!(
                                "option '{}' has capability/program definitions but no registered owner",
                                option_path
                            ),
                            path: Some(option_path.clone()),
                            file: None,
                        });
                    }
                }
                OwnershipStatus::External => {
                    diag.add(ValidationFinding {
                        severity: ValidationSeverity::Info,
                        message: format!(
                            "option '{}' originates from an external module; configure the local repository layer instead of editing the dependency",
                            option_path
                        ),
                        path: Some(option_path.clone()),
                        file: None,
                    });
                }
                OwnershipStatus::Owned => {}
            }
        }
    }

    fn check_duplicate_ownership(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        for (option_path, prov) in &inputs.provenance {
            let mut counts: HashMap<&str, usize> = HashMap::new();

            for def in &prov.definitions {
                if def.layer == Layer::Capability || def.layer == Layer::Program {
                    *counts.entry(def.source.file.as_str()).or_insert(0) += 1;
                }
            }

            for (file, count) in counts {
                if count > 1 {
                    diag.add(ValidationFinding {
                        severity: ValidationSeverity::Warning,
                        message: format!(
                            "option '{}' is declared {} times by module '{}'",
                            option_path, count, file
                        ),
                        path: Some(option_path.clone()),
                        file: Some(file.to_string()),
                    });
                }
            }
        }
    }

    fn check_path_drift(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        for (option_path, prov) in &inputs.provenance {
            for def in &prov.definitions {
                if def.layer == Layer::Capability || def.layer == Layer::Program {
                    if let Some(expected) = option_path_to_expected_dir(option_path) {
                        if !def.source.file.contains(&expected) {
                            diag.add(ValidationFinding {
                                severity: ValidationSeverity::Info,
                                message: format!(
                                    "option '{}' is defined in '{}' but path suggests '{}'",
                                    option_path, def.source.file, expected
                                ),
                                path: Some(option_path.clone()),
                                file: Some(def.source.file.clone()),
                            });
                        }
                    }
                }
            }
        }
    }

    fn check_copy_paste_descriptions(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        let mut desc_map: HashMap<&str, Vec<&str>> = HashMap::new();
        for (path, schema) in &inputs.schemas {
            if let Some(ref desc) = schema.description {
                if desc.len() > 10 {
                    desc_map
                        .entry(desc.as_str())
                        .or_default()
                        .push(path.as_str());
                }
            }
        }

        for (desc, paths) in &desc_map {
            if paths.len() > 1 {
                diag.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: format!(
                        "description '{}' is shared by {} options: {:?}",
                        desc,
                        paths.len(),
                        paths
                    ),
                    path: None,
                    file: None,
                });
            }
        }
    }

    fn check_empty_profiles(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        for profile in &inputs.repository.profiles {
            let has_defs = inputs.provenance.values().any(|p| {
                p.definitions
                    .iter()
                    .any(|d| d.profile.as_deref() == Some(profile.id.name.as_str()))
            });
            if !has_defs {
                diag.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: format!(
                        "profile '{}' (source: {}) declares no configuration",
                        profile.id.name, profile.source_file
                    ),
                    path: None,
                    file: Some(profile.source_file.clone()),
                });
            }
        }
    }

    fn check_broken_requests(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        for (option_path, prov) in &inputs.provenance {
            if !option_path.starts_with("gb.requires.") {
                continue;
            }
            for def in &prov.definitions {
                let Some(serde_json::Value::Array(items)) = def.value.as_ref() else {
                    continue;
                };
                for item in items {
                    let serde_json::Value::String(requester) = item else {
                        continue;
                    };
                    if parse_requester(requester).is_err() {
                        diag.add(ValidationFinding {
                            severity: ValidationSeverity::Error,
                            message: format!(
                                "invalid requester '{}' in {}",
                                requester, option_path
                            ),
                            path: Some(option_path.clone()),
                            file: Some(def.source.file.clone()),
                        });
                        diag.propose(RepairProposal {
                            action: RepairAction::FixRequesterString {
                                path: option_path.clone(),
                                requester: requester.clone(),
                            },
                            description: format!(
                                "fix requester '{}' to match grammar <kind>.<scope>[.<domain>].<name>",
                                requester
                            ),
                            confidence: 0.7,
                        });
                    }
                }
            }
        }
    }

    fn check_untracked_configuration(&self, inputs: &DoctorInputs, diag: &mut Diagnosis) {
        let module_paths: HashSet<&str> = inputs.modules.iter().map(|m| m.path.as_str()).collect();

        for (option_path, prov) in &inputs.provenance {
            for def in &prov.definitions {
                if (def.layer == Layer::Capability || def.layer == Layer::Program)
                    && !module_paths.contains(def.source.file.as_str())
                {
                    diag.add(ValidationFinding {
                        severity: ValidationSeverity::Warning,
                        message: format!(
                            "option '{}' is implemented in '{}' which is not a registered module",
                            option_path, def.source.file
                        ),
                        path: Some(option_path.clone()),
                        file: Some(def.source.file.clone()),
                    });
                }
            }
        }
    }
}

fn is_valid_gb_namespace(path: &str) -> bool {
    const VALID_PREFIXES: &[&str] = &[
        "gb.schemaVersion",
        "gb.host",
        "gb.user",
        "gb.requires.system",
        "gb.requires.home",
        "gb.programs.system",
        "gb.home.programs",
        "gb.home.desktop",
        "gb.debug",
    ];
    VALID_PREFIXES
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{}.", p)))
}

fn resolve_import_target(importer: &str, raw: &str) -> String {
    let path = raw.replace('\\', "/");
    if path.starts_with('<') || path.starts_with("http://") || path.starts_with("https://") {
        return path;
    }
    let dir = importer.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let joined = if dir.is_empty() {
        path
    } else {
        format!("{}/{}", dir, path)
    };
    normalize_segments(&joined)
}

fn normalize_segments(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

fn option_path_to_expected_dir(option_path: &str) -> Option<String> {
    if option_path.starts_with("services.") {
        let domain = option_path.split('.').nth(1)?;
        return Some(format!("capabilities/system/{}/", domain));
    }
    if option_path.starts_with("programs.") {
        let domain = option_path.split('.').nth(1)?;
        return Some(format!("programs/home/{}/", domain));
    }
    if option_path.starts_with("hardware.") {
        return Some("capabilities/system/hardware/".to_string());
    }
    if option_path.starts_with("networking.") {
        return Some("capabilities/system/networking/".to_string());
    }
    if option_path.starts_with("security.") {
        return Some("capabilities/system/security/".to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_model::source::{Position, SourceLocation, SourceSpan};
    use avalanche_model::{
        DefinitionType, ModuleId, OptionDefinition, Priority, Profile, ProfileId, Scope,
    };

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
            id: ModuleId {
                kind: ModuleKind::Capability,
                scope: Some(Scope::System),
                name: name.to_string(),
            },
            path: path.to_string(),
            imported: true,
        }
    }

    fn empty_inputs() -> DoctorInputs {
        let mut repository = Repository::new("/tmp/test");
        repository.schema_version = Some(1);
        DoctorInputs {
            repository,
            provenance: HashMap::new(),
            modules: Vec::new(),
            imports: HashMap::new(),
            schemas: HashMap::new(),
        }
    }

    fn svc() -> DoctorService {
        DoctorService::new()
    }

    #[test]
    fn clean_repository_has_no_errors() {
        let inputs = empty_inputs();
        let diag = svc().diagnose(&inputs);
        assert!(!diag.has_errors());
    }

    #[test]
    fn missing_schema_version_is_error() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = None;
        let diag = svc().diagnose(&inputs);
        assert!(diag.has_errors());
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("gb.schemaVersion")));
        assert!(diag
            .repairs
            .iter()
            .any(|r| matches!(r.action, RepairAction::AddSchemaVersion { version: 1 })));
    }

    #[test]
    fn valid_gb_namespace_not_flagged() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mut prov = Provenance::new("gb.host.name");
        prov.add(make_def(
            "gb.host.name",
            "hosts/pc/default.nix",
            Layer::Host,
        ));
        inputs.provenance.insert("gb.host.name".to_string(), prov);
        let diag = svc().diagnose(&inputs);
        assert!(!diag
            .findings
            .iter()
            .any(|f| f.message.contains("undeclared gb.*")));
    }

    #[test]
    fn invalid_gb_namespace_warns() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mut prov = Provenance::new("gb.unknown.thing");
        prov.add(make_def(
            "gb.unknown.thing",
            "hosts/pc/default.nix",
            Layer::Host,
        ));
        inputs
            .provenance
            .insert("gb.unknown.thing".to_string(), prov);
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("undeclared gb.*")));
    }

    #[test]
    fn unaggregated_module_is_error() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        inputs.modules.push(capability_module(
            "modules/capabilities/system/audio/pipewire.nix",
            "audio.pipewire",
        ));
        let diag = svc().diagnose(&inputs);
        assert!(diag.has_errors());
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("not imported by any aggregate")));
        assert!(diag
            .repairs
            .iter()
            .any(|r| matches!(r.action, RepairAction::AddAggregateImport { .. })));
    }

    #[test]
    fn aggregated_module_passes() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mod_path = "modules/capabilities/system/audio/pipewire.nix";
        inputs
            .modules
            .push(capability_module(mod_path, "audio.pipewire"));
        inputs.imports.insert(
            "modules/aggregate/nixos.nix".to_string(),
            vec![format!("../{}", mod_path.strip_prefix("modules/").unwrap())],
        );
        let diag = svc().diagnose(&inputs);
        assert!(!diag
            .findings
            .iter()
            .any(|f| f.message.contains("not imported by any aggregate")));
    }

    #[test]
    fn orphan_module_warns() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mod_path = "modules/capabilities/system/audio/pipewire.nix";
        inputs
            .modules
            .push(capability_module(mod_path, "audio.pipewire"));
        inputs.imports.insert(
            "modules/aggregate/nixos.nix".to_string(),
            vec![format!("../{}", mod_path.strip_prefix("modules/").unwrap())],
        );
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("defines no configuration")));
    }

    #[test]
    fn ambiguous_ownership_is_error() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
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
        inputs
            .provenance
            .insert("services.xserver.enable".to_string(), prov);
        inputs.modules.push(capability_module(
            "modules/capabilities/system/desktop/x11.nix",
            "desktop.x11",
        ));
        inputs.modules.push(capability_module(
            "modules/capabilities/system/graphics/nvidia.nix",
            "graphics.nvidia",
        ));
        let agg_imports: Vec<String> = inputs
            .modules
            .iter()
            .map(|m| format!("../{}", m.path.strip_prefix("modules/").unwrap()))
            .collect();
        inputs
            .imports
            .insert("modules/aggregate/nixos.nix".to_string(), agg_imports);
        let diag = svc().diagnose(&inputs);
        assert!(diag.has_errors());
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("ambiguous ownership")));
    }

    #[test]
    fn copy_paste_descriptions_warn() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        inputs.schemas.insert(
            "a.one".to_string(),
            OptionSchema {
                path: "a.one".to_string(),
                scope: Scope::System,
                nix_type: avalanche_model::NixType::Bool,
                default: None,
                description: Some(
                    "This is a shared description that is long enough to flag".to_string(),
                ),
                example: None,
                internal: false,
                read_only: false,
            },
        );
        inputs.schemas.insert(
            "b.two".to_string(),
            OptionSchema {
                path: "b.two".to_string(),
                scope: Scope::System,
                nix_type: avalanche_model::NixType::Bool,
                default: None,
                description: Some(
                    "This is a shared description that is long enough to flag".to_string(),
                ),
                example: None,
                internal: false,
                read_only: false,
            },
        );
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("shared by 2 options")));
    }

    #[test]
    fn empty_profile_warns() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        inputs.repository.profiles.push(Profile {
            id: ProfileId::new(Scope::System, "gaming"),
            source_file: "modules/profiles/system/gaming.nix".to_string(),
        });
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("declares no configuration")));
    }

    #[test]
    fn broken_requester_is_error() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mut prov = Provenance::new("gb.requires.system.audio.pipewire");
        let mut def = make_def(
            "gb.requires.system.audio.pipewire",
            "modules/profiles/system/workstation.nix",
            Layer::Profile,
        );
        def.value = Some(serde_json::json!(["profile.system.workstation"]));
        prov.add(def);
        inputs
            .provenance
            .insert("gb.requires.system.audio.pipewire".to_string(), prov);
        let diag = svc().diagnose(&inputs);
        assert!(diag.has_errors());
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("invalid requester")));
        assert!(diag
            .repairs
            .iter()
            .any(|r| matches!(r.action, RepairAction::FixRequesterString { .. })));
    }

    #[test]
    fn valid_requester_not_flagged() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mut prov = Provenance::new("gb.requires.system.audio.pipewire");
        let mut def = make_def(
            "gb.requires.system.audio.pipewire",
            "modules/profiles/system/workstation.nix",
            Layer::Profile,
        );
        def.value = Some(serde_json::json!(["profiles.system.workstation"]));
        prov.add(def);
        inputs
            .provenance
            .insert("gb.requires.system.audio.pipewire".to_string(), prov);
        let diag = svc().diagnose(&inputs);
        assert!(!diag
            .findings
            .iter()
            .any(|f| f.message.contains("invalid requester")));
    }

    #[test]
    fn untracked_capability_definition_warns() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mut prov = Provenance::new("services.pipewire.enable");
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/capabilities/system/audio/pipewire.nix",
            Layer::Capability,
        ));
        inputs
            .provenance
            .insert("services.pipewire.enable".to_string(), prov);
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("not a registered module")));
    }

    #[test]
    fn diagnosis_counts() {
        let mut diag = Diagnosis::new();
        diag.add(ValidationFinding {
            severity: ValidationSeverity::Error,
            message: "err".to_string(),
            path: None,
            file: None,
        });
        diag.add(ValidationFinding {
            severity: ValidationSeverity::Warning,
            message: "warn".to_string(),
            path: None,
            file: None,
        });
        diag.add(ValidationFinding {
            severity: ValidationSeverity::Warning,
            message: "warn2".to_string(),
            path: None,
            file: None,
        });
        assert_eq!(diag.error_count(), 1);
        assert_eq!(diag.warning_count(), 2);
        assert!(diag.has_errors());
    }

    #[test]
    fn repair_proposals_have_confidence() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = None;
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .repairs
            .iter()
            .all(|r| r.confidence > 0.0 && r.confidence <= 1.0));
    }

    #[test]
    fn path_drift_info_for_misplaced_option() {
        let mut inputs = empty_inputs();
        inputs.repository.schema_version = Some(1);
        let mut prov = Provenance::new("services.pipewire.enable");
        prov.add(make_def(
            "services.pipewire.enable",
            "modules/capabilities/system/gaming/steam.nix",
            Layer::Capability,
        ));
        inputs
            .provenance
            .insert("services.pipewire.enable".to_string(), prov);
        inputs.modules.push(capability_module(
            "modules/capabilities/system/gaming/steam.nix",
            "gaming.steam",
        ));
        inputs.imports.insert(
            "modules/aggregate/nixos.nix".to_string(),
            vec!["../capabilities/system/gaming/steam.nix".to_string()],
        );
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("path suggests")));
    }

    #[test]
    fn resolve_import_target_is_relative_to_importer() {
        assert_eq!(
            resolve_import_target(
                "modules/aggregate/nixos.nix",
                "../capabilities/system/audio/pipewire.nix"
            ),
            "modules/capabilities/system/audio/pipewire.nix"
        );
        assert_eq!(
            resolve_import_target("hosts/pc/default.nix", "./hardware.nix"),
            "hosts/pc/hardware.nix"
        );
        assert_eq!(resolve_import_target("flake.nix", "<nixpkgs>"), "<nixpkgs>");
    }

    #[test]
    fn duplicate_ownership_is_detected() {
        let mut inputs = empty_inputs();
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
        inputs
            .provenance
            .insert("services.pipewire.enable".to_string(), prov);
        inputs.modules.push(capability_module(
            "modules/capabilities/system/audio/pipewire.nix",
            "audio.pipewire",
        ));
        inputs.imports.insert(
            "modules/aggregate/nixos.nix".to_string(),
            vec!["../capabilities/system/audio/pipewire.nix".to_string()],
        );
        let diag = svc().diagnose(&inputs);
        assert!(diag
            .findings
            .iter()
            .any(|f| f.message.contains("declared 2 times")));
    }

    #[test]
    fn is_valid_gb_namespace_checks() {
        assert!(is_valid_gb_namespace("gb.host.name"));
        assert!(is_valid_gb_namespace("gb.requires.system.audio.pipewire"));
        assert!(is_valid_gb_namespace("gb.schemaVersion"));
        assert!(!is_valid_gb_namespace("gb.unknown.thing"));
        assert!(!is_valid_gb_namespace("services.pipewire.enable"));
    }
}
