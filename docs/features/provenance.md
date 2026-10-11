# Provenance

**Status: Implemented** (Phase 6)

Provenance makes definitions first-class. Avalanche can explain **why** an effective value exists.

---

## What it retains

Every `OptionDefinition` carries:

- source file
- source span (line + column range)
- layer (Profile / Host / User / Capability / Program / Desktop / External)
- priority (numeric; lower wins)
- host (if host-layer)
- user (if user-layer)
- profile (if profile-layer)
- condition (mkIf expression + evaluated result)
- definition type (Literal / MkDefault / MkForce / MkIf / MkMerge / MkOverride / Expression / Unknown)
- value (when extractable from source)

---

## The `Provenance` object

```rust
pub struct Provenance {
    pub option_path: String,
    pub definitions: Vec<OptionDefinition>,
    pub effective_value: Option<JsonValue>,
}
```

### Methods

| Method | Purpose |
|---|---|
| `winning_definition()` | The definition that produces the effective value |
| `overridden_definitions()` | All definitions that lose to the winner |
| `contributions()` | Per-element provenance for merged lists/attrsets |
| `sort_definitions()` | Deterministic ordering by priority, layer rank, file, line |
| `set_effective_value(v)` | Attach the Nix-evaluated effective value |

---

## Merged-value contribution provenance

For lists and attribute sets, Nix merges contributions from multiple definitions. Avalanche traces each merged element back to the definition that supplied it:

```rust
pub struct Contribution {
    pub value: JsonValue,
    pub source: Option<SourceLocation>,
    pub layer: Option<Layer>,
    pub priority: Option<Priority>,
    pub definition_type: Option<DefinitionType>,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub condition: Option<Condition>,
}
```

Example:

```text
gb.requires.system.audio.pipewire = [ "profiles.system.workstation", "programs.system.steam" ]

Contribution 1: "profiles.system.workstation"
  source: modules/profiles/system/workstation.nix:4
  layer: Profile, priority: 1500 (mkDefault)

Contribution 2: "programs.system.steam"
  source: modules/programs/system/steam.nix:7
  layer: Program, priority: 1000 (Literal)
```

---

## How it is built

The `build_provenance_index` function in `avalanche-index`:

1. Iterates every parsed file in the static index.
2. For each assignment, calls `analyze_definition` (source engine) to extract DefinitionType, Priority, Condition, and value.
3. Determines Layer, host, user, profile from the file's repository path via `context_for_file`.
4. Infers Scope from the option path or file location.
5. Filters out non-configuration paths (`imports`, `_module`, `options`, `meta`).
6. Collects into `Provenance` per option path.
7. Merges effective values from the semantic index.
8. Sorts definitions deterministically.

---

## File context classification

| Path pattern | Layer | Scope | Host/User/Profile |
|---|---|---|---|
| `hosts/<h>/default.nix` | Host | System | host=`<h>` |
| `hosts/<h>/home/<u>.nix` | User | Home | host=`<h>`, user=`<u>` |
| `users/<u>.nix` | User | Home | user=`<u>` |
| `modules/profiles/<scope>/<name>.nix` | Profile | from path | profile=`<name>` |
| `modules/capabilities/<scope>/...` | Capability | from path | - |
| `modules/programs/<scope>/...` | Program | from path | - |
| `modules/desktop/<scope>/...` | Desktop | from path | - |
| everything else | Host | inferred | - |

---

## CLI exposure

```bash
avalanche inspect option services.pipewire.enable
```

Outputs the winning definition, all definitions with layer/type/value, and (with `--json`) the full `Provenance` object.

---

## GUI exposure

The Provenance page (`apps/desktop/frontend/src/pages/Provenance.tsx`) displays definitions, winning value, overridden definitions, and contributions.

---

## Exit criteria

Avalanche can explain why an effective value exists.

---

## See also

- [Concepts: Provenance](../concepts/provenance.md)
- [Concepts: Layering & Priority](../concepts/layering-and-priority.md)
- [Repository Index](repository-index.md)
