use std::path::{Path, PathBuf};

use avalanche_graph::{build_request_graph, NodeId, NodeKind};
use avalanche_index::IndexService;
use avalanche_model::Repository;

fn provenance_fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/provenance")
}

#[test]
fn why_audio_pipewire_works_from_static_analysis() {
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

    let graph = build_request_graph(
        &index.provenance,
        &index.requests,
        &index.modules,
        &index.imports,
    );

    let cap = NodeId::new(NodeKind::Capability, "audio.pipewire");
    let paths = graph.why(&cap);
    assert!(
        !paths.is_empty(),
        "why(audio.pipewire) must resolve a requester chain from static provenance"
    );

    let root = &paths[0][0];
    assert_eq!(root.kind, NodeKind::Profile);
    assert_eq!(root.name, "profiles.system.workstation");
    assert_eq!(paths[0].last(), Some(&cap));
}

#[test]
fn capability_implementation_edges_from_static_provenance() {
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

    let graph = build_request_graph(
        &index.provenance,
        &index.requests,
        &index.modules,
        &index.imports,
    );

    let pipewire_enable = NodeId::new(NodeKind::Option, "services.pipewire.media-session.enable");
    let impls = graph.implementation(&pipewire_enable);
    assert!(
        impls.iter().any(|n| n.name.contains("pipewire")),
        "pipewire capability module must appear as an implementer of its own option"
    );
}

#[test]
fn host_override_edge_from_static_provenance() {
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

    let graph = build_request_graph(
        &index.provenance,
        &index.requests,
        &index.modules,
        &index.imports,
    );

    let ssh = NodeId::new(NodeKind::Option, "services.openssh.enable");
    let overriders = graph.overriders(&ssh);
    assert!(
        overriders.iter().any(|n| n.name.contains("hosts/pc")),
        "the pc host file must appear as an overrider of services.openssh.enable"
    );
}
