use std::path::{Path, PathBuf};

use avalanche_index::IndexService;
use avalanche_model::Repository;
use avalanche_ownership::{build_ownership_registry, OwnershipStatus};

fn provenance_fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/provenance")
}

#[test]
fn ownership_is_derived_from_provenance_not_filenames() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();
    assert!(
        !index.static_index.has_parse_errors(),
        "provenance fixture must parse cleanly: {:?}",
        index.static_index.parse_errors
    );

    let registry = build_ownership_registry(&index.provenance, &index.modules, &index.requests);

    let pipewire_session = "services.pipewire.media-session.enable";
    assert_eq!(
        registry.resolve(pipewire_session),
        OwnershipStatus::Owned,
        "the capability module that writes the option owns it"
    );
    assert_eq!(
        registry.owner_of(pipewire_session),
        Some("modules/capabilities/system/audio/pipewire.nix")
    );
    assert!(registry.can_auto_edit(pipewire_session));

    let pulseaudio = "hardware.pulseaudio.enable";
    assert_eq!(registry.resolve(pulseaudio), OwnershipStatus::Owned);
    assert_eq!(
        registry.owner_of(pulseaudio),
        Some("modules/capabilities/system/audio/pipewire.nix")
    );
}

#[test]
fn profile_and_host_definitions_do_not_own_upstream_options() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }

    let registry = build_ownership_registry(&index.provenance, &index.modules, &index.requests);

    let openssh = "services.openssh.enable";
    assert_eq!(
        registry.resolve(openssh),
        OwnershipStatus::Unowned,
        "a profile mkDefault plus a host override confer no capability/program owner"
    );
    assert!(!registry.can_auto_edit(openssh));
    assert!(registry
        .record_for(openssh)
        .unwrap()
        .contributing_files
        .contains(&"modules/profiles/system/workstation.nix".to_string()));
    assert!(registry
        .record_for(openssh)
        .unwrap()
        .contributing_files
        .contains(&"hosts/pc/default.nix".to_string()));

    let host_name = "networking.hostName";
    assert_eq!(registry.resolve(host_name), OwnershipStatus::Unowned);
}

#[test]
fn ambiguous_ownership_blocks_auto_edit() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let svc = IndexService::new();
    let index = svc.build_static_only(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }

    let registry = build_ownership_registry(&index.provenance, &index.modules, &index.requests);

    for path in registry.ambiguous_options() {
        assert!(
            !registry.can_auto_edit(path),
            "ambiguous ownership must never be auto-editable: {path}"
        );
    }
    for path in registry.external_options() {
        assert!(
            !registry.can_auto_edit(path),
            "external ownership must never be auto-editable: {path}"
        );
    }
}
