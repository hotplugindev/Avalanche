# Repository Contract

**Status: Implemented** (Phase 0)

The repository contract makes a managed Nix repository explicit and predictable. It is defined in `schemas/contract.nix` and declared inside each repository via `gb.schemaVersion`.

---

## Purpose

Without a contract, Avalanche would have to guess the shape of a repository. The contract eliminates guessing: every `gb.*` namespace is registered, every requester string follows a canonical grammar, and ownership/registration/profile/host-override/user-scoping/external-module semantics are machine-readable.

A repository that violates the contract is **rejected**, not guessed at.

---

## Namespaces

The contract registers every `gb.*` subtree:

| Namespace | Scope | Kind | Description |
|---|---|---|---|
| `gb.schemaVersion` | both | meta | Contract version (currently `1`) |
| `gb.host` | system | facts | Typed host identity and hardware facts |
| `gb.user` | both | identity | Active user identity |
| `gb.requires.system` | system | request | System capability request lists |
| `gb.requires.home` | home | request | Home capability request lists |
| `gb.programs.system` | system | application | Typed system application settings |
| `gb.home.programs` | home | application | Typed home application settings |
| `gb.home.desktop` | home | desktop | Structured desktop configuration |
| `gb.debug` | system | debug | Diagnostic switches |

Any `gb.*` option outside these namespaces is flagged as an undeclared namespace by the Doctor.

---

## Requester grammar

Every entry in a `gb.requires.*` list is a canonical requester string:

```text
<kind>.<scope>[.<domain>].<name>
```

Kinds (always plural): `programs`, `profiles`, `capabilities`, `hosts`, `users`.

Scoped kinds (`programs`, `profiles`, `capabilities`) require a `system` or `home` segment. Unscoped kinds (`hosts`, `users`) do not.

Valid:

```text
programs.home.ai.codex
profiles.system.gaming
capabilities.system.audio.pipewire
hosts.pc
users.hotplugin
```

Invalid (singular prefix):

```text
profile.system.development
program.home.spotify
host.pc
```

Singular forms are contract violations rejected by the Doctor.

---

## Ownership

Ownership is an explicit declaration, never inferred from the nearest filename. One upstream option has exactly one owning capability or program module.

Statuses: `owned`, `unowned`, `wrongOwner`, `ambiguous`, `external`.

---

## Capability registration

Capabilities live in:

```text
modules/capabilities/<scope>/<domain>/<name>.nix
```

They activate when their request list (`gb.requires.<scope>.<domain>.<name>`) is non-empty or they are explicitly enabled.

---

## Aggregate registration

Every module must be imported by an aggregate:

```text
modules/aggregate/nixos.nix   (system)
modules/aggregate/home.nix    (home)
```

No recursive auto-import. A module absent from an aggregate is inert and flagged by the Doctor.

---

## Profile semantics

Profiles live in:

```text
modules/profiles/<scope>/<name>.nix
```

They express **intent**: request capabilities and set `mkDefault` values. They never own low-level implementation.

---

## Host override semantics

A host override is a definition at host priority:

```text
hosts/<host>/default.nix          (system)
hosts/<host>/home/<username>.nix  (home, per-user)
```

Reset to profile default **deletes** the host definition; it never writes `false`.

---

## User scoping

Per-user configuration has an explicit source location:

```text
users/<username>.nix              (identity)
hosts/<host>/home/<username>.nix  (per-host overrides)
```

The shared `hosts/<host>/home.nix` is deprecated.

---

## External module boundaries

Options from external flake inputs are tagged `external`. Avalanche configures the local repository layer and never edits the dependency.

---

## Rust mirror

The contract is mirrored in `avalanche-model::contract`:

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub struct RepositoryContract { ... }
pub fn parse_requester(requester: &str) -> Result<RequesterKind, ModelError>
```

`RepositoryContract::v1()` returns the full namespace list, requester kinds, and the `ownership_inferred_from_filename = false` invariant.

---

## See also

- [`gb.*` Namespaces reference](../reference/gb-namespaces.md)
- [Requester Grammar reference](../reference/requester-grammar.md)
- [Repo Doctor](repo-doctor.md)
