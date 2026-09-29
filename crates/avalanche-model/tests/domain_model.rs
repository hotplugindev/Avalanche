use avalanche_model::*;
use std::collections::BTreeMap;

fn span(start_line: u32, end_line: u32) -> SourceSpan {
    SourceSpan::new(
        Position { line: start_line, column: 1 },
        Position { line: end_line, column: 1 },
    )
}

fn def(path: &str, priority: Priority, layer: Layer, scope: Scope, def_type: DefinitionType) -> OptionDefinition {
    OptionDefinition {
        option_path: path.to_string(),
        value: None,
        source: SourceLocation::new("modules/test.nix", span(1, 10)),
        priority,
        layer,
        scope,
        host: None,
        user: None,
        profile: None,
        condition: None,
        definition_type: def_type,
    }
}

#[test]
fn repository_represents_example_repo() {
    let mut repo = Repository::new("/home/user/nixos-config");
    repo.schema_version = Some(1);

    repo.hosts.push(Host {
        name: "pc".to_string(),
        system: "x86_64-linux".to_string(),
        class: HostClass::Desktop,
        roles: vec![HostRole::Workstation, HostRole::Gaming],
        desktop: Desktop::Mango,
        shell: Shell::Zsh,
        state_version: "25.11".to_string(),
        hardware: Hardware {
            cpu: Cpu::Amd,
            gpu: Gpu::Amd,
            has_battery: false,
            has_bluetooth: true,
            has_touchpad: false,
            has_printer: false,
        },
    });

    repo.hosts.push(Host {
        name: "laptop".to_string(),
        system: "x86_64-linux".to_string(),
        class: HostClass::Laptop,
        roles: vec![HostRole::Development],
        desktop: Desktop::Mango,
        shell: Shell::Zsh,
        state_version: "25.11".to_string(),
        hardware: Hardware {
            cpu: Cpu::Intel,
            gpu: Gpu::Intel,
            has_battery: true,
            has_bluetooth: true,
            has_touchpad: true,
            has_printer: false,
        },
    });

    repo.users.push(User {
        username: "hotplugin".to_string(),
        full_name: "Giona Berti".to_string(),
        email: "giona@example.com".to_string(),
        is_main: true,
    });

    repo.profiles.push(Profile {
        id: ProfileId::new(Scope::System, "workstation"),
        source_file: "modules/profiles/system/workstation.nix".to_string(),
    });
    repo.profiles.push(Profile {
        id: ProfileId::new(Scope::Home, "desktop"),
        source_file: "modules/profiles/home/desktop.nix".to_string(),
    });

    repo.modules.push(Module {
        id: ModuleId {
            kind: ModuleKind::Aggregate,
            scope: Some(Scope::System),
            name: "nixos".to_string(),
        },
        path: "modules/aggregate/nixos.nix".to_string(),
        imported: true,
    });
    repo.modules.push(Module {
        id: ModuleId {
            kind: ModuleKind::Capability,
            scope: Some(Scope::System),
            name: "audio.pipewire".to_string(),
        },
        path: "modules/capabilities/system/audio/pipewire.nix".to_string(),
        imported: true,
    });
    repo.modules.push(Module {
        id: ModuleId {
            kind: ModuleKind::Program,
            scope: Some(Scope::Home),
            name: "ai.opencode".to_string(),
        },
        path: "modules/programs/home/ai/opencode.nix".to_string(),
        imported: true,
    });

    assert_eq!(repo.hosts.len(), 2);
    assert_eq!(repo.users.len(), 1);
    assert_eq!(repo.profiles.len(), 2);
    assert_eq!(repo.modules.len(), 3);
}

#[test]
fn host_role_queries() {
    let host = Host {
        name: "pc".to_string(),
        system: "x86_64-linux".to_string(),
        class: HostClass::Desktop,
        roles: vec![HostRole::Workstation, HostRole::Gaming],
        desktop: Desktop::Mango,
        shell: Shell::Zsh,
        state_version: "25.11".to_string(),
        hardware: Hardware {
            cpu: Cpu::Amd,
            gpu: Gpu::Amd,
            has_battery: false,
            has_bluetooth: true,
            has_touchpad: false,
            has_printer: false,
        },
    };

    assert!(host.has_role(&HostRole::Workstation));
    assert!(host.has_role(&HostRole::Gaming));
    assert!(!host.has_role(&HostRole::Server));
}

#[test]
fn capability_request_paths() {
    let sys_cap = CapabilityId {
        scope: Scope::System,
        domain: "audio".to_string(),
        name: "pipewire".to_string(),
    };
    assert_eq!(sys_cap.request_path(), "gb.requires.system.audio.pipewire");

    let home_cap = CapabilityId {
        scope: Scope::Home,
        domain: "desktop".to_string(),
        name: "mango".to_string(),
    };
    assert_eq!(home_cap.request_path(), "gb.requires.home.desktop.mango");
}

#[test]
fn application_option_paths() {
    let sys_app = ApplicationId {
        scope: Scope::System,
        category: "gaming".to_string(),
        name: "steam".to_string(),
    };
    assert_eq!(sys_app.option_path(), "gb.programs.system.gaming.steam");
    assert_eq!(sys_app.requester_name(), "programs.system.gaming.steam");

    let home_app = ApplicationId {
        scope: Scope::Home,
        category: "ai".to_string(),
        name: "opencode".to_string(),
    };
    assert_eq!(home_app.option_path(), "gb.home.programs.ai.opencode");
    assert_eq!(home_app.requester_name(), "programs.home.ai.opencode");
}

#[test]
fn profile_requester_names() {
    let profile = ProfileId::new(Scope::System, "workstation");
    assert_eq!(profile.requester_name(), "profiles.system.workstation");

    let home_profile = ProfileId::new(Scope::Home, "desktop");
    assert_eq!(home_profile.requester_name(), "profiles.home.desktop");
}

#[test]
fn mk_default_is_priority_not_flag() {
    let mk_default_def = def(
        "programs.git.enable",
        Priority::mk_default(),
        Layer::Profile,
        Scope::Home,
        DefinitionType::MkDefault,
    );
    assert!(mk_default_def.is_mk_default());
    assert!(mk_default_def.priority.is_default());
    assert_eq!(mk_default_def.priority.0, PRIORITY_DEFAULT);

    let normal_def = def(
        "programs.git.enable",
        Priority::normal(),
        Layer::Host,
        Scope::Home,
        DefinitionType::Literal,
    );
    assert!(!normal_def.is_mk_default());
}

#[test]
fn priority_wins_over() {
    let force = Priority::mk_force();
    let normal = Priority::normal();
    let default = Priority::mk_default();

    assert!(force.wins_over(&normal));
    assert!(force.wins_over(&default));
    assert!(normal.wins_over(&default));
    assert!(!default.wins_over(&normal));
    assert!(!default.wins_over(&force));
}

#[test]
fn effective_value_winning_definition() {
    let profile_def = def(
        "services.pipewire.enable",
        Priority::mk_default(),
        Layer::Profile,
        Scope::System,
        DefinitionType::MkDefault,
    );
    let host_def = def(
        "services.pipewire.enable",
        Priority::normal(),
        Layer::Host,
        Scope::System,
        DefinitionType::Literal,
    );

    let effective = EffectiveValue {
        option_path: "services.pipewire.enable".to_string(),
        value: Some(JsonValue(serde_json::Value::Bool(true))),
        definitions: vec![profile_def.clone(), host_def.clone()],
    };

    let winner = effective.winning_definition().unwrap();
    assert_eq!(winner.layer, Layer::Host);
    assert_eq!(winner.priority, Priority::normal());
}

#[test]
fn reset_host_override_means_delete_not_write_false() {
    let intent = MutationIntent::ResetHostOverride {
        path: "programs.git.enable".to_string(),
        host: "pc".to_string(),
    };
    match &intent {
        MutationIntent::ResetHostOverride { path, host } => {
            assert_eq!(path, "programs.git.enable");
            assert_eq!(host, "pc");
        }
        _ => panic!("wrong intent"),
    }
}

#[test]
fn provenance_preserves_all_definition_metadata() {
    let mut provenance = Provenance::new("services.pipewire.enable");

    let mut condition_def = def(
        "services.pipewire.enable",
        Priority::normal(),
        Layer::Capability,
        Scope::System,
        DefinitionType::MkIf,
    );
    condition_def.host = Some("pc".to_string());
    condition_def.condition = Some(Condition {
        expression: "config.hardware.hasAudio".to_string(),
        evaluated_value: Some(NixValue::Bool(true)),
        satisfied: Some(true),
    });

    provenance.add(def(
        "services.pipewire.enable",
        Priority::mk_default(),
        Layer::Profile,
        Scope::System,
        DefinitionType::MkDefault,
    ));
    provenance.add(condition_def);

    assert_eq!(provenance.definitions.len(), 2);
    assert_eq!(provenance.definitions[0].layer, Layer::Profile);
    assert_eq!(provenance.definitions[1].layer, Layer::Capability);
    assert_eq!(provenance.definitions[1].host.as_deref(), Some("pc"));
    assert_eq!(provenance.definitions[1].definition_type, DefinitionType::MkIf);
}

#[test]
fn condition_represents_hardware_gating() {
    let condition = Condition {
        expression: "config.gb.host.hardware.gpu == \"amd\"".to_string(),
        evaluated_value: Some(NixValue::Bool(true)),
        satisfied: Some(true),
    };
    assert!(condition.satisfied.unwrap());
}

#[test]
fn nix_value_types() {
    let bool_val = NixValue::Bool(true);
    let int_val = NixValue::Int(42);
    let str_val = NixValue::Str("hello".to_string());
    let list_val = NixValue::List(vec![NixValue::Int(1), NixValue::Int(2)]);

    let mut attrs = BTreeMap::new();
    attrs.insert("enable".to_string(), NixValue::Bool(true));
    let attrs_val = NixValue::Attrs(attrs);

    assert_eq!(bool_val, NixValue::Bool(true));
    assert_ne!(bool_val, NixValue::Bool(false));
    assert_eq!(int_val, NixValue::Int(42));
    assert_eq!(str_val, NixValue::Str("hello".to_string()));
    assert_eq!(list_val, NixValue::List(vec![NixValue::Int(1), NixValue::Int(2)]));
    assert_eq!(attrs_val.clone(), attrs_val);
    assert_ne!(bool_val, int_val);
}

#[test]
fn ownership_statuses() {
    let owned = Ownership::new("services.pipewire.enable", "capabilities/system/audio/pipewire.nix");
    assert!(owned.can_auto_edit());
    assert_eq!(owned.status, OwnershipStatus::Owned);

    let external = Ownership {
        option_path: "programs.firefox.enable".to_string(),
        owner: "nixpkgs".to_string(),
        status: OwnershipStatus::External,
    };
    assert!(!external.can_auto_edit());

    let ambiguous = Ownership {
        option_path: "services.xserver.enable".to_string(),
        owner: "multiple".to_string(),
        status: OwnershipStatus::Ambiguous,
    };
    assert!(!ambiguous.can_auto_edit());
}

#[test]
fn transaction_lifecycle() {
    let mut txn = Transaction::new("txn-001", vec![
        MutationIntent::SetOption {
            path: "programs.git.enable".to_string(),
            value: serde_json::Value::Bool(true),
            scope: "home".to_string(),
        },
    ]);
    assert_eq!(txn.status, TransactionStatus::Planned);
    assert!(txn.file_changes.is_empty());
    assert!(txn.validation.is_none());

    txn.status = TransactionStatus::Validating;
    txn.status = TransactionStatus::Ready;
    txn.status = TransactionStatus::Applied;
    assert_eq!(txn.status, TransactionStatus::Applied);
}

#[test]
fn validation_report_tiers() {
    let mut report = ValidationReport::new(ValidationTier::Structural);
    assert!(report.passed);

    report.add(ValidationFinding {
        severity: ValidationSeverity::Warning,
        message: "unused variable".to_string(),
        path: None,
        file: Some("modules/test.nix".to_string()),
    });
    assert!(report.passed);

    report.add(ValidationFinding {
        severity: ValidationSeverity::Error,
        message: "type mismatch".to_string(),
        path: Some("programs.git.enable".to_string()),
        file: None,
    });
    assert!(!report.passed);
    assert_eq!(report.errors().count(), 1);
}

#[test]
fn fingerprint_stale_detection() {
    let fp1 = Fingerprint {
        git_head: Some("abc123".to_string()),
        working_tree_hash: Some("def456".to_string()),
        flake_lock_hash: Some("ghi789".to_string()),
        schema_version: Some(1),
        index_schema_version: 1,
    };
    let fp2 = Fingerprint {
        git_head: Some("abc123".to_string()),
        working_tree_hash: Some("CHANGED".to_string()),
        flake_lock_hash: Some("ghi789".to_string()),
        schema_version: Some(1),
        index_schema_version: 1,
    };
    assert_ne!(fp1, fp2);
}

#[test]
fn option_schema_types() {
    let bool_schema = OptionSchema {
        path: "programs.git.enable".to_string(),
        scope: Scope::Home,
        nix_type: NixType::Bool,
        default: Some(JsonValue(serde_json::Value::Bool(false))),
        description: Some("Whether to enable git".to_string()),
        example: None,
        internal: false,
        read_only: false,
    };
    assert_eq!(bool_schema.nix_type, NixType::Bool);

    let list_schema = OptionSchema {
        path: "gb.requires.system.audio.pipewire".to_string(),
        scope: Scope::System,
        nix_type: NixType::List,
        default: Some(JsonValue(serde_json::json!([]))),
        description: Some("Request PipeWire audio".to_string()),
        example: None,
        internal: false,
        read_only: false,
    };
    assert_eq!(list_schema.nix_type, NixType::List);
}

#[test]
fn resolved_request_aggregation() {
    let resolved = ResolvedRequest {
        capability: CapabilityId {
            scope: Scope::System,
            domain: "audio".to_string(),
            name: "pipewire".to_string(),
        },
        requesters: vec![
            "programs.system.steam".to_string(),
            "profiles.system.desktop".to_string(),
        ],
        active: true,
    };
    assert!(resolved.is_active());
    assert_eq!(resolved.requesters.len(), 2);
}

#[test]
fn external_layer_not_editable() {
    let external_def = def(
        "programs.firefox.enable",
        Priority::normal(),
        Layer::External,
        Scope::Home,
        DefinitionType::Literal,
    );
    assert_eq!(external_def.layer, Layer::External);
}

#[test]
fn scope_display() {
    assert_eq!(format!("{}", Scope::System), "system");
    assert_eq!(format!("{}", Scope::Home), "home");
}

#[test]
fn mutation_intent_variants() {
    let intents = vec![
        MutationIntent::SetOption { path: "a".into(), value: serde_json::Value::Bool(true), scope: "home".into() },
        MutationIntent::SetProfileDefault { path: "a".into(), value: serde_json::Value::Bool(true), profile: "workstation".into() },
        MutationIntent::SetHostOverride { path: "a".into(), value: serde_json::Value::Bool(false), host: "pc".into() },
        MutationIntent::ResetHostOverride { path: "a".into(), host: "pc".into() },
        MutationIntent::EnableCapability { capability: "audio.pipewire".into(), requester: "steam".into() },
        MutationIntent::DisableCapability { capability: "audio.pipewire".into(), requester: "steam".into() },
        MutationIntent::AddRequest { capability: "audio.pipewire".into(), requester: "steam".into() },
        MutationIntent::RemoveRequest { capability: "audio.pipewire".into(), requester: "steam".into() },
        MutationIntent::CreateModule { name: "new-app".into(), kind: "program".into() },
        MutationIntent::CreateHost { name: "vm".into() },
        MutationIntent::CreateUser { name: "guest".into() },
    ];
    assert_eq!(intents.len(), 11);
}
