use avalanche_model::contract::{parse_requester, RepositoryContract, RequesterKind, CURRENT_SCHEMA_VERSION};

#[test]
fn current_schema_version_is_one() {
    assert_eq!(CURRENT_SCHEMA_VERSION, 1);
    assert_eq!(RepositoryContract::v1().schema_version, 1);
}

#[test]
fn valid_requesters_parse() {
    assert_eq!(
        parse_requester("programs.home.ai.codex").unwrap(),
        RequesterKind::Program
    );
    assert_eq!(
        parse_requester("programs.system.wine").unwrap(),
        RequesterKind::Program
    );
    assert_eq!(
        parse_requester("profiles.system.gaming").unwrap(),
        RequesterKind::Profile
    );
    assert_eq!(
        parse_requester("capabilities.system.audio.pipewire").unwrap(),
        RequesterKind::Capability
    );
    assert_eq!(parse_requester("hosts.pc").unwrap(), RequesterKind::Host);
    assert_eq!(
        parse_requester("users.hotplugin").unwrap(),
        RequesterKind::User
    );
}

#[test]
fn singular_kind_prefixes_are_rejected() {
    assert!(parse_requester("profile.system.development").is_err());
    assert!(parse_requester("program.home.spotify").is_err());
    assert!(parse_requester("host.pc").is_err());
    assert!(parse_requester("user.hotplugin").is_err());
}

#[test]
fn malformed_requesters_are_rejected() {
    assert!(parse_requester("").is_err());
    assert!(parse_requester("foo.bar.baz").is_err());
    assert!(parse_requester("programs.x").is_err());
    assert!(parse_requester("programs.nope.codex").is_err());
    assert!(parse_requester("hosts").is_err());
}

#[test]
fn contract_covers_managed_repo_namespaces() {
    let c = RepositoryContract::v1();
    let observed = [
        "gb.schemaVersion",
        "gb.host.name",
        "gb.host.hardware.cpu",
        "gb.user.username",
        "gb.requires.system.audio.pipewire",
        "gb.requires.home.git",
        "gb.programs.system.steam.enable",
        "gb.home.programs.ai.codex.enable",
        "gb.home.desktop.mango.mangowc.monitors",
        "gb.debug.dumpRequests",
    ];
    for path in observed {
        assert!(c.covers_path(path), "contract does not cover {}", path);
    }
}

#[test]
fn contract_rejects_unknown_namespace() {
    let c = RepositoryContract::v1();
    assert!(!c.covers_path("gb.unknown.thing"));
    assert!(!c.covers_path("programs.git.enable"));
}

#[test]
fn ownership_is_never_filename_inferred() {
    let c = RepositoryContract::v1();
    assert!(!c.ownership_inferred_from_filename);
}

#[test]
fn requester_kinds_are_plural() {
    let c = RepositoryContract::v1();
    for kind in &c.requester_kinds {
        assert!(kind.ends_with('s'), "requester kind must be plural: {}", kind);
    }
    assert_eq!(RequesterKind::Program.plural(), "programs");
    assert_eq!(RequesterKind::Profile.plural(), "profiles");
}
