use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::{MutationIntent, Profile, ProfileId, Repository, Scope, TransactionStatus};
use avalanche_write::WriteError;

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-mutate-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_file(
        &dir,
        "modules/capabilities/system/audio/pipewire.nix",
        "{\n  services.pipewire.enable = true;\n}\n",
    );
    write_file(
        &dir,
        "modules/capabilities/system/complex/thing.nix",
        "{\n  services.complex.enable = pkgs.foo;\n}\n",
    );
    write_file(
        &dir,
        "modules/profiles/system/workstation.nix",
        "{\n  networking.hostName = \"base\";\n}\n",
    );
    write_file(
        &dir,
        "hosts/pc/default.nix",
        "{\n  networking.hostName = \"pc-host\";\n  services.foo.enable = true;\n}\n",
    );
    dir
}

fn repo_with_profile(root: &Path) -> Repository {
    let mut r = Repository::new(root.display().to_string());
    r.profiles.push(Profile {
        id: ProfileId::new(Scope::System, "workstation"),
        source_file: "modules/profiles/system/workstation.nix".to_string(),
    });
    r
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn set_option_replace_produces_diff_and_captures_fingerprint() {
    let dir = fixture("happy");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let plan = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(plan.status, TransactionStatus::Ready);
    assert_eq!(plan.file_changes.len(), 1);
    assert_eq!(
        plan.file_changes[0].path,
        "modules/capabilities/system/audio/pipewire.nix"
    );
    assert!(plan.file_changes[0]
        .after
        .contains("services.pipewire.enable = false;"));
    assert!(plan.diff.contains('-') && plan.diff.contains('+'));
    assert_eq!(plan.fingerprint, index.fingerprint);
    assert_eq!(plan.edits.len(), 1);

    cleanup(&dir);
}

#[test]
fn set_option_denied_when_not_owned() {
    let dir = fixture("owndeny");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.foo.enable".to_string(),
                value: serde_json::json!(true),
                scope: "system".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::OwnershipDenied(_)));
    cleanup(&dir);
}

#[test]
fn set_option_refuses_computed_or_unknown_definition() {
    let dir = fixture("computed");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.complex.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::Unsupported(_)));
    cleanup(&dir);
}

#[test]
fn reset_host_override_deletes_not_writes_false() {
    let dir = fixture("reset");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let plan = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::ResetHostOverride {
                path: "networking.hostName".to_string(),
                host: "pc".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(plan.file_changes.len(), 1);
    let after = &plan.file_changes[0].after;
    assert!(!after.contains("networking.hostName = \"pc-host\";"));
    assert!(!after.contains("= false"));
    assert!(after.contains("services.foo.enable = true;"));

    cleanup(&dir);
}

#[test]
fn reset_host_override_without_override_is_error() {
    let dir = fixture("resetnone");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::ResetHostOverride {
                path: "services.pipewire.enable".to_string(),
                host: "pc".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::EditFailed(_)));
    cleanup(&dir);
}

#[test]
fn set_profile_default_renders_mk_default() {
    let dir = fixture("mkdefault");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let plan = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::SetProfileDefault {
                path: "networking.hostName".to_string(),
                value: serde_json::json!("pc"),
                profile: "workstation".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(
        plan.file_changes[0].path,
        "modules/profiles/system/workstation.nix"
    );
    assert!(plan.file_changes[0].after.contains("lib.mkDefault"));
    assert_eq!(plan.edits[0].render, avalanche_model::RenderHint::MkDefault);

    cleanup(&dir);
}

#[test]
fn set_host_override_inserts_into_host_file() {
    let dir = fixture("hostins");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let plan = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::SetHostOverride {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                host: "pc".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(plan.file_changes[0].path, "hosts/pc/default.nix");
    assert!(plan.file_changes[0]
        .after
        .contains("services.pipewire.enable = false;"));

    cleanup(&dir);
}

#[test]
fn stale_index_is_rejected_before_planning() {
    let dir = fixture("stale");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let mut index = engine.index_static(&repo).unwrap();
    index.fingerprint.git_head = Some("stale-head".to_string());

    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::StaleIndex));
    cleanup(&dir);
}

#[test]
fn unsupported_intent_reports_no_change() {
    let dir = fixture("unsup");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::EnableCapability {
                capability: "audio.pipewire".to_string(),
                requester: "profiles.system.workstation".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::Unsupported(_)));
    cleanup(&dir);
}
