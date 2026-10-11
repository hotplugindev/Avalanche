use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::{MutationIntent, Repository};
use avalanche_write::WriteError;

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read_file(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-mg-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_file(
        &dir,
        "modules/aggregate/nixos.nix",
        "{ lib, config, ... }:\n{\n  imports = [ ../core/schema.nix ];\n}\n",
    );
    write_file(
        &dir,
        "modules/aggregate/home.nix",
        "{ lib, config, ... }:\n{\n  imports = [ ../core/schema.nix ];\n}\n",
    );
    write_file(
        &dir,
        "modules/core/schema.nix",
        "{ lib, ... }:\n{\n  config.gb.schemaVersion = 1;\n}\n",
    );
    dir
}

#[test]
fn create_program_module_generates_file_and_registers_in_aggregate() {
    let dir = fixture("prog");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "steam".to_string(),
                kind: "program".to_string(),
            }],
        )
        .unwrap();

    assert!(txn.diff.as_deref().unwrap().contains("steam.nix"));

    let on_disk = read_file(&dir, "modules/programs/system/steam.nix");
    assert!(on_disk.contains("gb.programs.system.steam.enable"));

    let agg = read_file(&dir, "modules/aggregate/nixos.nix");
    assert!(agg.contains("../programs/system/steam.nix"));
    assert!(agg.contains("../core/schema.nix"));
    avalanche_source::parse(&agg).unwrap();

    cleanup(&dir);
}

#[test]
fn create_capability_module_generates_and_registers() {
    let dir = fixture("cap");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "audio.pipewire".to_string(),
                kind: "capability".to_string(),
            }],
        )
        .unwrap();

    let on_disk = read_file(&dir, "modules/capabilities/system/audio/pipewire.nix");
    assert!(on_disk.contains("gb.requires.system.audio.pipewire"));

    let agg = read_file(&dir, "modules/aggregate/nixos.nix");
    assert!(agg.contains("../capabilities/system/audio/pipewire.nix"));
    avalanche_source::parse(&agg).unwrap();

    cleanup(&dir);
}

#[test]
fn create_profile_module_generates_and_registers() {
    let dir = fixture("prof");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "workstation".to_string(),
                kind: "profile".to_string(),
            }],
        )
        .unwrap();

    let on_disk = read_file(&dir, "modules/profiles/system/workstation.nix");
    assert!(on_disk.contains("workstation"));

    let agg = read_file(&dir, "modules/aggregate/nixos.nix");
    assert!(agg.contains("../profiles/system/workstation.nix"));
    avalanche_source::parse(&agg).unwrap();

    cleanup(&dir);
}

#[test]
fn create_module_duplicate_path_is_error_and_leaves_repo_unchanged() {
    let dir = fixture("dup");
    write_file(
        &dir,
        "modules/programs/system/steam.nix",
        "{ lib, config, ... }:\n{\n  options.gb.programs.system.steam.enable = lib.mkEnableOption \"steam\";\n}\n",
    );
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let before = read_file(&dir, "modules/aggregate/nixos.nix");
    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "steam".to_string(),
                kind: "program".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::EditFailed(_)));
    assert!(err.to_string().contains("already exists"));
    assert_eq!(read_file(&dir, "modules/aggregate/nixos.nix"), before);

    cleanup(&dir);
}

#[test]
fn create_module_invalid_kind_is_error() {
    let dir = fixture("badkind");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "foo".to_string(),
                kind: "widget".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::EditFailed(_)));
    assert!(err.to_string().contains("unknown module kind"));

    cleanup(&dir);
}

#[test]
fn create_home_program_with_category_creates_category_default() {
    let dir = fixture("homecat");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "home:web.firefox".to_string(),
                kind: "program".to_string(),
            }],
        )
        .unwrap();

    let module = read_file(&dir, "modules/programs/home/web/firefox.nix");
    assert!(module.contains("gb.home.programs.web.firefox.enable"));

    let cat_default = read_file(&dir, "modules/programs/home/web/default.nix");
    assert!(cat_default.contains("./firefox.nix"));

    let agg = read_file(&dir, "modules/aggregate/home.nix");
    assert!(agg.contains("../programs/home/web/default.nix"));
    assert!(!agg.contains("../programs/home/web/firefox.nix"));
    avalanche_source::parse(&agg).unwrap();

    cleanup(&dir);
}
