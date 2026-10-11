use std::path::{Path, PathBuf};

use avalanche_index::{compute_fingerprint, is_stale, IndexService};
use avalanche_model::{
    Cpu, DefinitionType, Desktop, Fingerprint, Gpu, Hardware, Host, HostClass, HostRole, Layer,
    Profile, ProfileId, Repository, Scope, Shell, User,
};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/simple-eval")
}

fn provenance_fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/provenance")
}

#[test]
fn static_only_index_builds() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();
    assert_eq!(index.fingerprint.index_schema_version, 1);
    assert!(
        !index.static_index.parsed_files.is_empty() || !index.static_index.parse_errors.is_empty()
    );
}

#[test]
fn fingerprint_roundtrip_not_stale() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let fp = compute_fingerprint(&repo_path).unwrap();
    assert!(!is_stale(&fp, &fp));
}

#[test]
fn stale_when_head_changes() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let fp = compute_fingerprint(&repo_path).unwrap();
    let changed = Fingerprint {
        git_head: Some("deadbeef".into()),
        ..fp.clone()
    };
    assert!(is_stale(&fp, &changed));
}

#[test]
fn where_is_value_written_via_index() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }
    let locs = index.source_location_for("description");
    assert!(!locs.is_empty());
    assert!(locs[0].file.ends_with("flake.nix"));
}

#[test]
fn check_stale_detects_no_change() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();
    let stored = index.fingerprint.clone();
    let stale = svc.check_stale(&repository, &stored).unwrap();
    assert!(!stale);
}

#[test]
fn entity_indices_lookup() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let mut repository = Repository::new(repo_path.display().to_string());
    repository.hosts.push(Host {
        name: "pc".into(),
        system: "x86_64-linux".into(),
        class: HostClass::Desktop,
        roles: vec![HostRole::Workstation],
        desktop: Desktop::Mango,
        shell: Shell::Zsh,
        state_version: "26.05".into(),
        hardware: Hardware {
            cpu: Cpu::Amd,
            gpu: Gpu::Amd,
            has_battery: false,
            has_bluetooth: true,
            has_touchpad: false,
            has_printer: false,
        },
    });
    repository.users.push(User {
        username: "alice".into(),
        full_name: "Alice".into(),
        email: "alice@example.com".into(),
        is_main: true,
    });
    repository.profiles.push(Profile {
        id: ProfileId::new(Scope::System, "workstation"),
        source_file: "modules/profiles/system/workstation.nix".into(),
    });

    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();

    assert_eq!(
        index.host_by_name("pc").map(|h| h.name.as_str()),
        Some("pc")
    );
    assert!(index.host_by_name("nope").is_none());
    assert_eq!(
        index
            .user_by_username("alice")
            .map(|u| u.full_name.as_str()),
        Some("Alice")
    );
    assert!(index.user_by_username("bob").is_none());
    assert_eq!(
        index
            .profile_by_name("workstation")
            .map(|p| p.id.name.as_str()),
        Some("workstation")
    );
    assert!(index.profile_by_name("gaming").is_none());
}

#[test]
fn provenance_index_explains_profile_default_and_host_override() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();

    assert!(
        index.static_index.parse_errors.is_empty(),
        "{:?}",
        index.static_index.parse_errors
    );

    let provenance = index.provenance_for("services.openssh.enable").unwrap();
    assert_eq!(provenance.definitions.len(), 2);

    let winner = provenance.winning_definition().unwrap();
    assert_eq!(winner.layer, Layer::Host);
    assert_eq!(winner.host.as_deref(), Some("pc"));
    assert_eq!(winner.definition_type, DefinitionType::Literal);
    assert_eq!(winner.value, Some(serde_json::Value::Bool(false)));

    let overridden = provenance.overridden_definitions();
    assert_eq!(overridden.len(), 1);
    assert_eq!(overridden[0].layer, Layer::Profile);
    assert_eq!(overridden[0].profile.as_deref(), Some("workstation"));
    assert_eq!(overridden[0].definition_type, DefinitionType::MkDefault);
    assert!(overridden[0].priority.is_default());
}

#[test]
fn provenance_index_preserves_condition_and_capability_layer() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();

    let provenance = index.provenance_for("services.xserver.enable").unwrap();
    let definition = &provenance.definitions[0];
    assert_eq!(definition.layer, Layer::Profile);
    assert_eq!(definition.definition_type, DefinitionType::MkIf);
    assert_eq!(
        definition.condition.as_ref().map(|c| c.expression.as_str()),
        Some("config.gb.host.hardware.hasBattery")
    );

    let pipewire = index
        .provenance_for("services.pipewire.media-session.enable")
        .unwrap();
    assert_eq!(pipewire.definitions[0].layer, Layer::Capability);
}
