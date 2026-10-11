use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::{MutationIntent, Repository};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-cap-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, content).unwrap();
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

fn seed(dir: &Path) {
    write(dir, "flake.nix", "{ }\n");
    write(
        dir,
        "modules/core/schema.nix",
        "{ lib, ... }: { options.gb.schemaVersion = lib.mkOption { type = lib.types.int; default = 1; readOnly = true; }; config.gb.schemaVersion = 1; }\n",
    );
    write(
        dir,
        "modules/core/requests/system.nix",
        "{ lib, ... }: { options.gb.requires.system = lib.mkOption { type = lib.types.attrsOf (lib.types.listOf lib.types.str); default = { }; }; }\n",
    );
    write(
        dir,
        "modules/aggregate/nixos.nix",
        "{ config, lib, ... }: { imports = [ ../core/schema.nix ../core/requests/system.nix ]; }\n",
    );
    write(
        dir,
        "modules/capabilities/system/audio/pipewire.nix",
        "{ config, lib, ... }: { }\n",
    );
    write(
        dir,
        "modules/profiles/system/workstation.nix",
        "{ config, lib, ... }: { }\n",
    );
    write(
        dir,
        "modules/programs/system/steam.nix",
        "{ config, lib, ... }: { }\n",
    );
}

#[test]
fn enable_capability_adds_requester_to_profile_file() {
    let dir = tmp("enable");
    seed(&dir);

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine
        .index_static(&Repository::new(dir.display().to_string()))
        .unwrap();

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::EnableCapability {
                capability: "audio.pipewire".to_string(),
                requester: "profiles.system.workstation".to_string(),
            }],
        )
        .unwrap();
    assert_eq!(txn.status, avalanche_model::TransactionStatus::Applied);

    let after = read(&dir, "modules/profiles/system/workstation.nix");
    assert!(
        after.contains("gb.requires.system.audio.pipewire"),
        "{after}"
    );
    assert!(after.contains("profiles.system.workstation"), "{after}");

    cleanup(&dir);
}

#[test]
fn enable_then_disable_round_trips_the_request_list() {
    let dir = tmp("roundtrip");
    seed(&dir);
    write(
        &dir,
        "modules/profiles/system/workstation.nix",
        "{ config, lib, ... }: { gb.requires.system.audio.pipewire = [ \"profiles.system.workstation\" ]; }\n",
    );

    let engine = ConfigurationEngine::new(dir.display().to_string());

    let index = engine
        .index_static(&Repository::new(dir.display().to_string()))
        .unwrap();
    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::DisableCapability {
                capability: "audio.pipewire".to_string(),
                requester: "profiles.system.workstation".to_string(),
            }],
        )
        .unwrap();
    assert_eq!(txn.status, avalanche_model::TransactionStatus::Applied);

    let after = read(&dir, "modules/profiles/system/workstation.nix");
    assert!(
        !after.contains("profiles.system.workstation"),
        "requester should be removed: {after}"
    );
    assert!(
        after.contains("gb.requires.system.audio.pipewire"),
        "{after}"
    );

    cleanup(&dir);
}

#[test]
fn add_request_is_idempotent() {
    let dir = tmp("idempotent");
    seed(&dir);
    write(
        &dir,
        "modules/programs/system/steam.nix",
        "{ config, lib, ... }: { gb.requires.system.audio.pipewire = [ \"programs.system.steam\" ]; }\n",
    );

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine
        .index_static(&Repository::new(dir.display().to_string()))
        .unwrap();

    let before = read(&dir, "modules/programs/system/steam.nix");
    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::AddRequest {
                capability: "audio.pipewire".to_string(),
                requester: "programs.system.steam".to_string(),
            }],
        )
        .unwrap();
    let after = read(&dir, "modules/programs/system/steam.nix");

    assert_eq!(before, after, "idempotent add must not modify the file");
    assert!(txn.file_changes.is_empty(), "no file changes expected");

    cleanup(&dir);
}

#[test]
fn enable_unknown_capability_fails_closed() {
    let dir = tmp("unknown");
    seed(&dir);

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine
        .index_static(&Repository::new(dir.display().to_string()))
        .unwrap();

    let err = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::EnableCapability {
                capability: "nonexistent.thing".to_string(),
                requester: "profiles.system.workstation".to_string(),
            }],
        )
        .unwrap_err();
    assert!(
        matches!(err, avalanche_write::WriteError::EditFailed(_)),
        "{err:?}"
    );

    let profile = read(&dir, "modules/profiles/system/workstation.nix");
    assert!(
        !profile.contains("gb.requires.system.nonexistent"),
        "nothing written on failure: {profile}"
    );

    cleanup(&dir);
}

#[test]
fn computed_request_list_is_refused() {
    let dir = tmp("computed");
    seed(&dir);
    write(
        &dir,
        "modules/profiles/system/workstation.nix",
        "{ config, lib, ... }: { gb.requires.system.audio.pipewire = if config.gb.host.hasBattery then [ \"profiles.system.workstation\" ] else [ ]; }\n",
    );

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine
        .index_static(&Repository::new(dir.display().to_string()))
        .unwrap();

    let err = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::AddRequest {
                capability: "audio.pipewire".to_string(),
                requester: "programs.system.steam".to_string(),
            }],
        )
        .unwrap_err();
    assert!(
        matches!(err, avalanche_write::WriteError::EditFailed(_)),
        "{err:?}"
    );

    cleanup(&dir);
}

#[test]
fn malformed_requester_is_rejected() {
    let dir = tmp("malformed");
    seed(&dir);

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine
        .index_static(&Repository::new(dir.display().to_string()))
        .unwrap();

    let err = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::AddRequest {
                capability: "audio.pipewire".to_string(),
                requester: "bogus-requester-without-kind".to_string(),
            }],
        )
        .unwrap_err();
    assert!(
        matches!(err, avalanche_write::WriteError::EditFailed(_)),
        "{err:?}"
    );

    cleanup(&dir);
}
