# Layering & Priority

**Status: Implemented** (Phases 2, 6, 13)

Avalanche models configuration as a stack of *layers*, each contributing
definitions at a numeric *priority*. The effective value of an option is the
winning definition after Nix's priority merge.

---

## The layer model

```text
External        lowest rank - options from external flake inputs
Profile         reusable role defaults (mkDefault values, capability requests)
Capability      capability implementations
Program         application modules
Desktop         desktop-environment modules
User            per-user Home Manager configuration
Host            per-host overrides
```

The UI simplifies this into three views:

```text
Profile default
Host override
Effective
```

The engine retains the complete definition/priority model underneath.

---

## Priority arithmetic

Priorities are integers. **Lower wins.**

| Constant | Value | Meaning |
|---|---|---|
| `PRIORITY_VM_OVERRIDE` | 10 | VM-specific override |
| `PRIORITY_OVERRIDE` / `PRIORITY_FORCE` | 50 | `mkForce` / `mkOverride` |
| `PRIORITY_NORMAL` | 1000 | ordinary assignment |
| `PRIORITY_DEFAULT` | 1500 | `mkDefault` |

```rust
pub struct Priority(pub i16);

impl Priority {
    pub fn normal() -> Self { Self(1000) }
    pub fn mk_default() -> Self { Self(1500) }
    pub fn mk_force() -> Self { Self(50) }
    pub fn mk_override() -> Self { Self(50) }
    pub fn wins_over(&self, other: &Priority) -> bool { self.0 < other.0 }
}
```

The winning definition is the one with the **lowest** priority number. If two
definitions share the same priority, Nix's merge semantics apply (and for
lists/attrsets, contributions merge).

---

## `mkDefault` is a priority, not a flag

A common mistake is to represent `mkDefault` as:

```text
isProfileDefault = true
```

That loses information. `mkDefault` is a Nix priority (1500). The model must
retain the priority value. The UI can *project* it into:

```text
Profile default
Host override
Effective
```

but the engine stores the real priority so that `mkForce`, `mkOverride`, and
future custom priorities all work correctly.

---

## Host override semantics

A host override is a definition at host priority (normally 1000, or whatever
the assignment uses). It sits above profile defaults (1500) and below forced
values (50).

```text
Profile default:  ON   (priority 1500)
PC override:      ON   (priority 1000)
Laptop override:  OFF  (priority 1000)

PC effective:     ON   (host def wins over profile def)
Laptop effective: OFF  (host def wins over profile def)
```

---

## Reset semantics

> "Reset to profile default" means: **delete the host override**.

It does **not** mean:

```text
write false
```

When the host definition is removed, the profile/default definition becomes
effective again. This is a hard invariant. Agents and the write engine must
never implement reset as writing `false`.

```text
resetHostOverride
    ↓
delete host definition
    ↓
profile/default definition becomes effective
```

---

## Definition type

Each `OptionDefinition` carries a `definition_type`:

```text
Literal       plain value (true, "hello", [ 1 2 3 ])
MkDefault     lib.mkDefault wrapper
MkForce       lib.mkForce wrapper
MkIf          lib.mkIf conditional
MkMerge       lib.mkMerge
MkOverride    lib.mkOverride N wrapper
Expression    identifier / binary op / known function call
Unknown       inherit / unrecognized structure
```

The definition type drives the write policy:

| Definition type | Automatic editing? |
|---|---|
| Literal | Yes |
| MkDefault / MkForce / MkOverride | Yes (preserve wrapper) |
| MkIf | No - raw/specialized |
| MkMerge | No - raw/specialized |
| Expression | Only when understood |
| Unknown | No - raw only |

---

## Multi-host layering

The same option can have different effective values per host because each host
contributes its own definitions. Avalanche evaluates per host and retains the
host provenance on every definition.

```text
services.pipewire.enable
  ← modules/capabilities/system/audio/pipewire.nix  (Layer::Capability, priority 1000)
  ← hosts/pc/default.nix                            (Layer::Host, priority 1000, host="pc")
  ← hosts/laptop/default.nix                        (Layer::Host, priority 1000, host="laptop")
```

The winning definition for host `pc` is the one from `hosts/pc/default.nix`
(same priority, but host layer outranks capability layer in the UI projection;
the engine resolves by priority first, then by layer rank).

---

## Layer rank (tie-breaking)

When two definitions share the same priority, the engine breaks ties by layer
rank (higher rank wins):

```text
External   0
Profile    1
Capability 2
Program    3
Desktop    4
User       5
Host       6
```

This is an implementation detail of `Provenance::sort_definitions` and
`winning_definition`. It ensures deterministic resolution.

---

## See also

- [Provenance](provenance.md) - how definitions and winning values are tracked
- [Priority Values reference](../reference/priority-values.md)
- [Write Engine](../features/write-engine.md) - how mutations respect layering
