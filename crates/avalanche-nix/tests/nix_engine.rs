use avalanche_model::Scope;
use avalanche_nix::NixService;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

static FIXTURE_DIR: OnceLock<PathBuf> = OnceLock::new();

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("simple-eval")
}

fn setup_fixture_git_repo() -> &'static PathBuf {
    FIXTURE_DIR.get_or_init(|| {
        let tmp = std::env::temp_dir().join("avalanche-nix-test-fixture");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let src = fixture_source().join("flake.nix");
        let dst = tmp.join("flake.nix");
        std::fs::copy(&src, &dst).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(&tmp)
            .output()
            .unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&tmp)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "fixture"])
            .current_dir(&tmp)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .output()
            .unwrap();

        tmp
    })
}

fn nix_available() -> bool {
    Command::new("nix")
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn service() -> NixService {
    let path = setup_fixture_git_repo();
    NixService::new(path.to_str().unwrap().to_string())
}

#[test]
fn eval_simple_expression() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval("1 + 1").unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn eval_json_bool() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval_json("true").unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn eval_json_attrs() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval_json(r#"{ x = 1; y = "two"; }"#).unwrap();
    assert_eq!(result["x"], 1);
    assert_eq!(result["y"], "two");
}

#[test]
fn list_hosts_returns_testhost() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let hosts = svc.list_hosts().unwrap();
    assert!(hosts.contains(&"testhost".to_string()));
}

#[test]
fn eval_host_returns_system() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval_host("testhost").unwrap();
    assert_eq!(result.host, "testhost");
    assert_eq!(result.system, "x86_64-linux");
}

#[test]
fn eval_host_option_bool() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc
        .eval_host_option("testhost", "services.pipewire.enable")
        .unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn eval_host_option_gb_schema_version() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval_host_option("testhost", "gb.schemaVersion").unwrap();
    assert_eq!(result, serde_json::json!(1));
}

#[test]
fn extract_option_schema_bool() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let schema = svc
        .extract_option("services.pipewire.enable", Scope::System, "testhost")
        .unwrap();
    assert_eq!(schema.path, "services.pipewire.enable");
    assert_eq!(schema.scope, Scope::System);
    assert_eq!(schema.nix_type, avalanche_model::NixType::Bool);
    assert_eq!(
        schema.description.as_deref(),
        Some("Whether to enable PipeWire.")
    );
    assert!(!schema.internal);
    assert!(!schema.read_only);
}

#[test]
fn extract_option_value() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let value = svc
        .extract_option_value("services.pipewire.enable", Scope::System, "testhost")
        .unwrap();
    assert_eq!(value, serde_json::Value::Bool(true));
}

#[test]
fn get_assertions_from_fixture() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let assertions = svc.get_assertions("testhost").unwrap();
    assert_eq!(assertions.len(), 2);

    let passed = assertions.iter().find(|a| a.passed);
    assert!(passed.is_some());
    assert_eq!(passed.unwrap().message, "test assertion passes");

    let failed = avalanche_nix::failed_assertions(&assertions);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].message, "test assertion fails");
}

#[test]
fn eval_home_manager_option() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc
        .eval_home_manager_option("testhost", "testuser", "programs.git.enable")
        .unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn eval_home_manager_option_string() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc
        .eval_home_manager_option("testhost", "testuser", "programs.git.userName")
        .unwrap();
    assert_eq!(result, serde_json::json!("Test User"));
}

#[test]
fn eval_flake_attr_test_values() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result =
        avalanche_nix::eval_attr(svc.process(), svc.repo_path(), ".#testValues").unwrap();
    assert_eq!(result["bool"], true);
    assert_eq!(result["int"], 42);
    assert_eq!(result["float"], 3.14);
    assert_eq!(result["str"], "hello");
    assert_eq!(result["list"], serde_json::json!([1, 2, 3]));
    assert_eq!(result["attrs"]["a"], 1);
    assert_eq!(result["attrs"]["b"], "two");
}

#[test]
fn eval_flake_attr_nested() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = avalanche_nix::eval_attr(
        svc.process(),
        svc.repo_path(),
        ".#testValues.nested.x.y.z",
    )
    .unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn nix_type_inference() {
    use avalanche_model::NixType;
    use avalanche_nix::infer_nix_type;

    assert_eq!(infer_nix_type(&serde_json::json!(true)), NixType::Bool);
    assert_eq!(infer_nix_type(&serde_json::json!(42)), NixType::Int);
    assert_eq!(infer_nix_type(&serde_json::json!(3.14)), NixType::Float);
    assert_eq!(
        infer_nix_type(&serde_json::json!("hello")),
        NixType::Str
    );
    assert_eq!(
        infer_nix_type(&serde_json::json!([1, 2, 3])),
        NixType::List
    );
    assert_eq!(
        infer_nix_type(&serde_json::json!({"a": 1})),
        NixType::Attrs
    );
    assert_eq!(
        infer_nix_type(&serde_json::json!({"_type": "derivation", "name": "foo"})),
        NixType::Package
    );
}

#[test]
fn json_decode_roundtrip() {
    use avalanche_model::NixValue;
    use avalanche_nix::{decode_json, json_to_nix_value, value_to_json};

    let raw = r#"{"enable": true, "count": 5, "items": ["a", "b"]}"#;
    let json = decode_json(raw).unwrap();
    let nix_val = json_to_nix_value(&json);

    match &nix_val {
        NixValue::Attrs(map) => {
            assert_eq!(map.get("enable"), Some(&NixValue::Bool(true)));
            assert_eq!(map.get("count"), Some(&NixValue::Int(5)));
        }
        _ => panic!("expected attrs"),
    }

    let back = value_to_json(&nix_val);
    assert_eq!(back.0["enable"], true);
    assert_eq!(back.0["count"], 5);
}

#[test]
fn process_config_defaults() {
    use avalanche_nix::NixProcessConfig;
    use std::time::Duration;

    let config = NixProcessConfig::default();
    assert_eq!(config.nix_binary, "nix");
    assert_eq!(config.timeout, Duration::from_secs(300));
    assert!(config.extra_args.is_empty());
}

#[test]
fn eval_failure_returns_error() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval("builtins.throw \"test error\"");
    assert!(result.is_err());
}

#[test]
fn nonexistent_option_fails_gracefully() {
    if !nix_available() {
        return;
    }
    let svc = service();
    let result = svc.eval_host_option("testhost", "nonexistent.option.path");
    assert!(result.is_err());
}
