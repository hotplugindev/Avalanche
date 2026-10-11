# Provenance

**Status: Implemented** (Phase 6)

Effective values are insufficient. Avalanche retains every *definition* and its
origin so it can explain **why** an effective value exists.

---

## What provenance retains

Each `OptionDefinition` carries:

| Field | Meaning |
|---|---|
| `option_path` | the dotted option path |
| `value` | the assigned value (if extractable) |
| `source.file` | the Nix file where it is written |
| `source.span` | exact line/column range |
| `priority` | Nix priority (1000 = normal, 1500 = mkDefault, 50 = mkForce) |
| `layer` | Profile / Host / User / Capability / Program / Desktop / External |
| `scope` | System or Home |
| `host` | host name, if a host-layer definition |
| `user` | user name, if a user-layer definition |
| `profile` | profile name, if a profile-layer definition |
| `condition` | the `mkIf` expression and its evaluated result |
| `definition_type` | Literal / MkDefault / MkForce / MkIf / MkMerge / MkOverride / Expression / Unknown |

---

## The `Provenance` object

```rust
pub struct Provenance {
    pub option_path: String,
    pub definitions: Vec<OptionDefinition>,
    pub effective_value: Option<JsonValue>,
}
```

A `Provenance` collects **all** definitions for one option path, sorted by
priority (ascending) then layer rank (descending) then file then line.

### Winning definition

```rust
pub fn winning_definition(&self) -> Option<&OptionDefinition>
```

The definition with the lowest priority number (and highest layer rank on
 ties). This is what produces the effective value.

### Overridden definitions

```rust
pub fn overridden_definitions(&self) -> Vec<&OptionDefinition>
```

Every definition that is *not* the winner. Useful for UI: "Overridden by
`hosts/pc/programs.nix`".

---

## Merged-value contribution provenance

For lists and attribute sets, Nix *merges* contributions. Avalanche retains
per-contribution provenance where possible:

```rust
pub fn contributions(&self) -> Vec<Contribution>
```

Each `Contribution` links a single merged element back to the definition that
supplied it:

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
  ← modules/profiles/system/workstation.nix  (Layer::Profile, mkDefault)

Contribution 2: "programs.system.steam"
  ← modules/programs/system/steam.nix        (Layer::Program, Literal)
```

---

## How provenance is built

The index walks every parsed Nix file, extracts assignments, and for each
assignment:

1. Classifies the expression (`analyze_definition`) into a `DefinitionType` and
   `Priority` (detecting `mkDefault`, `mkForce`, `mkOverride`, `mkIf`).
2. Determines the `Layer`, `host`, `user`, `profile` from the file's location
   in the repository layout.
3. Records the `SourceLocation` (file + span).
4. Attaches the evaluated value (if a literal/list/attrs) as `value`.

All definitions for the same option path are collected into one `Provenance`.
The semantic index then attaches the *effective* value from Nix evaluation.

---

## Conditions

A `mkIf` definition carries a `Condition`:

```rust
pub struct Condition {
    pub expression: String,          // e.g. "config.gb.host.hardware.hasBattery"
    pub evaluated_value: Option<NixValue>,
    pub satisfied: Option<bool>,
}
```

The expression is preserved from source. The evaluated value and satisfaction
flag come from Nix evaluation (when `--nix` is used). Without evaluation, the
expression is still retained for display.

---

## Why this matters

Without provenance, Avalanche would be a value viewer. With provenance it can
answer:

- Where is this value written?
- Is it a profile default or a host override?
- What overrides it?
- Why is it enabled (which requester/profile)?
- What will change if I edit it?

This is the foundation for ownership, the request graph, and safe mutation.

---

## See also

- [Layering & Priority](layering-and-priority.md)
- [Ownership](ownership.md)
- [Provenance feature](../features/provenance.md)
