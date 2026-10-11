# Nix Engine

**Status: Implemented** (Phase 3)

The Nix engine lives in `avalanche-nix`. It provides reliable communication with the Nix package manager.

---

## Process abstraction

All Nix interactions go through `NixProcess`:

- configurable binary path (`nix_binary`)
- configurable timeout
- experimental feature flags (`nix-command`, `flakes`)
- structured error reporting (no shell interpolation)
- `run(args, repo_path) -> ProcessOutput { success, stdout, stderr }`

This satisfies the security rule: never execute arbitrary strings through a shell when a structured process API exists.

---

## Evaluation

| Method | Purpose |
|---|---|
| `eval(expr)` | Evaluate an arbitrary Nix expression, return raw string |
| `eval_json(expr)` | Evaluate and decode JSON |
| `flake_check()` | Run `nix flake check` |
| `build(attr)` | Run `nix build` |
| `build_dry_run(attr)` | Run `nix build --dry-run` |

---

## Option extraction

| Method | Purpose |
|---|---|
| `extract_option(path, scope, host)` | Extract full option schema (type, default, description, example, internal, readOnly) via `nix eval --apply` |
| `extract_option_value(path, scope, host)` | Extract the effective value of an option |

Option schemas are decoded into `OptionSchema` with `NixType` inference from the type description string.

---

## Host evaluation

| Method | Purpose |
|---|---|
| `list_hosts()` | Enumerate `nixosConfigurations` attrnames |
| `eval_host(host)` | Evaluate a host's system info |
| `eval_host_option(host, path)` | Evaluate a single option on a host |

---

## Home Manager evaluation

| Method | Purpose |
|---|---|
| `list_home_configs()` | Enumerate `homeConfigurations` attrnames |
| `eval_home_manager(host, user)` | Evaluate a Home Manager config |
| `eval_home_manager_option(host, user, path)` | Evaluate a single HM option |

---

## Assertions

`get_assertions(host)` retrieves and evaluates host assertions, returning `Vec<Assertion { message, passed }>`. Failures are identified before writes.

---

## Flake discovery

| Method | Purpose |
|---|---|
| `discover_inputs()` | Discover flake inputs from lock file |
| `discover_external_modules()` | Identify external module sources, excluding base dependencies |

---

## Nix type representation

`parse_nix_type_string` maps Nix type descriptions to `NixType`:

```text
"boolean" -> Bool
"signed integer" -> Int
"float" -> Float
"string" -> Str
"path" -> Path
"list of ..." -> List
"attribute set of ..." -> AttrsOf
"attribute set" -> Attrs
"submodule" -> Submodule
"package" -> Package
"one of ..." -> Enum
"function" / "lambda" / "raw" -> Function
otherwise -> Unknown
```

---

## What the Nix engine does NOT do

It answers *what does the configuration mean?* It does **not** answer *where should I edit it?* That is the source engine's job.

---

## See also

- [Source Engine](source-engine.md)
- [Repository Index](repository-index.md)
- [Validation](validation.md)
