# Ownership

**Status: Implemented** (Phase 7)

Avalanche distinguishes three separate roles for every upstream configuration
option. Conflating them is the most common architectural error in configuration
managers.

---

## Three roles

```text
Owner
    the module that is *responsible* for the option
    (usually a capability or program module)

Requester
    the entity that *asks* for the capability to be active
    (a profile, a program, a host, a user)

Implementer
    the module that *writes* the upstream option
    (the capability module that sets services.pipewire.enable = true)
```

Example:

```text
Application "steam"
    │
    │ requests
    ▼
Capability "audio.pipewire"
    │
    │ implemented by
    ▼
Capability module modules/capabilities/system/audio/pipewire.nix
    │
    │ owns
    ▼
Upstream options services.pipewire.*, hardware.pulseaudio.*
```

- **Requester** of `audio.pipewire`: `programs.system.steam`
- **Implementer** of `services.pipewire.enable`: the pipewire capability module
- **Owner** of `services.pipewire.enable`: the pipewire capability module

---

## Ownership states

| State | Meaning | Auto-edit? |
|---|---|---|
| `Owned` | exactly one registered capability/program module owns it | Yes |
| `Unowned` | no registered module owns it | No |
| `WrongOwner` | a capability/program definition comes from a file outside the owner | No |
| `Ambiguous` | two or more distinct modules claim ownership | No |
| `External` | originates from an external flake input | No (configure local layer) |

---

## How ownership is determined

The `OwnershipRegistry` is built from provenance + modules + requests:

1. For each option, collect all definitions whose layer is `Capability` or
   `Program`.
2. Map each such definition's file to a registered `Module`.
3. If zero modules claim it:
   - if all definitions are `External` → `External`
   - otherwise → `Unowned`
4. If two or more *distinct* module paths claim it → `Ambiguous`.
5. If exactly one module claims it, but another capability/program definition
   exists in a file that is *not* that module → `WrongOwner`.
6. Otherwise → `Owned`.

Ownership is **never** inferred from the nearest filename. It comes from the
repository model/contract: a module is a registered entity with a kind and a
path.

---

## Duplicate ownership detection

If the same module declares the same option more than once (e.g. two bindings
in one file), the registry still reports `Owned` (same module path), but the
Doctor flags it as a duplicate-declaration warning.

---

## External ownership

Options from external flake inputs are tagged `External`. Avalanche may
configure them through local modules but **never edits the dependency
itself**.

```text
origin = external
    ↓
configure the local repository layer instead
```

---

## Why ownership blocks writes

The write engine resolves ownership before planning any mutation of an upstream
option:

```text
Unknown owner       → no automatic write
Ambiguous ownership → require explicit choice
External source     → local configuration only
```

This prevents the GUI from silently editing an option that belongs to a
capability it does not control.

---

## See also

- [Provenance](provenance.md) - the definitions that feed ownership
- [Ownership feature](../features/ownership.md)
- [Request Graph](../features/request-graph.md) - how requesters/implementers
  appear in the graph
