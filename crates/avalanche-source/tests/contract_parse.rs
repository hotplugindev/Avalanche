use avalanche_model::RepositoryContract;
use avalanche_source::{parse, Expr};

fn contract_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schemas/contract.nix");
    std::fs::read_to_string(path).expect("schemas/contract.nix must exist")
}

#[test]
fn contract_is_parseable_nix() {
    let src = contract_source();
    let file = parse(&src).expect("contract.nix must parse");
    match &file.expr {
        Expr::Attrs(bindings) => {
            let keys: Vec<&str> = bindings.iter().map(|b| b.path[0].as_str()).collect();
            assert!(keys.contains(&"schemaVersion"));
            assert!(keys.contains(&"namespaces"));
            assert!(keys.contains(&"requesterGrammar"));
            assert!(keys.contains(&"ownership"));
            assert!(keys.contains(&"capabilityRegistration"));
            assert!(keys.contains(&"aggregateRegistration"));
            assert!(keys.contains(&"profileSemantics"));
            assert!(keys.contains(&"hostOverrideSemantics"));
            assert!(keys.contains(&"userScoping"));
            assert!(keys.contains(&"externalModules"));
        }
        other => panic!("contract top level must be an attrset, got {:?}", other),
    }
}

#[test]
fn contract_declares_every_model_namespace() {
    let src = contract_source();
    parse(&src).expect("contract.nix must parse");
    let c = RepositoryContract::v1();
    for n in &c.namespaces {
        let literal = format!("\"{}\"", n.path);
        assert!(
            src.contains(&literal),
            "schemas/contract.nix must declare namespace {}",
            n.path
        );
    }
}

#[test]
fn contract_schema_version_matches_model() {
    let src = contract_source();
    assert!(src.contains("schemaVersion = 1;"));
    assert_eq!(RepositoryContract::v1().schema_version, 1);
}
