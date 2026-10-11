# Repository Index

**Status: Implemented** (Phase 5)

The repository index lives in `avalanche-index`. It builds the complete derived repository model from static source analysis and Nix evaluation.

---

## Structure

```rust
pub struct RepositoryIndex {
    pub repository: Repository,
    pub options: HashMap<String, OptionSchema>,
    pub definitions: HashMap<String, Vec<OptionDefinition>>,
    pub effective: HashMap<String, EffectiveValue>,
    pub provenance: HashMap<String, Provenance>,
    pub capabilities: Vec<Capability>,
    pub applications: Vec<Application>,
    pub profiles: Vec<Profile>,
    pub hosts: Vec<Host>,
    pub users: Vec<User>,
    pub modules: Vec<Module>,
    pub assignments: HashMap<String, Vec<Assignment>>,
    pub source_locations: HashMap<String, Vec<SourceLocation>>,
    pub imports: HashMap<String, Vec<String>>,
    pub assertions: HashMap<String, Vec<Assertion>>,
    pub requests: Vec<ResolvedRequest>,
    pub flake_inputs: Vec<FlakeInput>,
    pub fingerprint: Fingerprint,
    pub static_index: StaticIndex,
    pub semantic_index: SemanticIndex,
}
```

---

## Static index

Built by walking every `.nix` file in the repository (skipping `.git`, `target`, `node_modules`), parsing each with the source engine, and extracting:

- assignments (by dotted path)
- source locations
- imports
- module classification (kind + scope from path)
- parse errors (recorded, not swallowed)

---

## Semantic index

Built by running Nix evaluation (when `--nix` is used):

- effective values (from `gb.*` config tree)
- option schemas
- assertions per host
- resolved requests
- flake inputs
- eval errors (recorded)

Without Nix, the semantic index is empty and the engine operates on static data only.

---

## Provenance index

Built from the static index's parsed files:

1. Walk every assignment in every file.
2. Classify the expression (`analyze_definition`) into DefinitionType + Priority + Condition.
3. Determine Layer, host, user, profile from the file's repository location.
4. Attach the evaluated value (if extractable).
5. Collect all definitions per option path into a `Provenance`.
6. Sort by priority (ascending) then layer rank (descending).
7. Merge effective values from the semantic index.

---

## Fingerprint

```text
git HEAD
working tree hash
flake.lock hash
GB schema version
index schema version
```

Computed via `compute_fingerprint(repo_path)`. Used for staleness detection.

---

## Stale detection

`is_stale(stored, current)` compares two fingerprints. If any component differs, the index is stale and must be rebuilt before any mutation.

---

## Queries

| Method | Returns |
|---|---|
| `option(path)` | OptionSchema |
| `definitions_for(path)` | Vec<OptionDefinition> |
| `effective_value(path)` | EffectiveValue |
| `provenance_for(path)` | Provenance |
| `assignment_for(path)` | [Assignment] |
| `source_location_for(path)` | [SourceLocation] |
| `imports_for(file)` | [String] |
| `assertions_for_host(host)` | [Assertion] |
| `failed_assertions()` | Vec<(host, Assertion)> |
| `active_requests()` | Vec<&ResolvedRequest> |
| `modules_of_kind(kind)` | Vec<&Module> |
| `host_by_name(name)` | Option<&Host> |
| `user_by_username(name)` | Option<&User> |
| `profile_by_name(name)` | Option<&Profile> |
| `where_is_value_written(path)` | Vec<&Assignment> |
| `auto_editable(path)` | bool |

---

## Build modes

| Mode | Method | Requires Nix? |
|---|---|---|
| Full | `IndexService::build(repo, nix)` | Yes |
| Static only | `IndexService::build_static_only(repo)` | No |

The CLI defaults to static-only. Pass `--nix` for full evaluation.

---

## Exit criteria

The index can answer:

- what exists
- where it is
- what it means
- current value
- source location

---

## See also

- [Nix Engine](nix-engine.md)
- [Source Engine](source-engine.md)
- [Provenance](provenance.md)
