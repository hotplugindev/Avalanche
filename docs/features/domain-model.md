# Domain Model

**Status: Implemented** (Phase 2)

The domain model lives in `avalanche-model`. It defines every semantic type used across the engine. No filesystem or UI behavior belongs here.

---

## Entities

| Type | Purpose |
|---|---|
| `Repository` | Top-level container: root path, schema version, hosts, users, profiles, modules, fingerprint |
| `Host` | A managed machine: name, system, class, roles, desktop, shell, state version, hardware facts |
| `User` | A managed user: username, full name, email, is_main |
| `Profile` | A reusable role: id (scope + name), source file |
| `Module` | A registered Nix module: id (kind + scope + name), path, imported flag |
| `Application` | A typed application: id (scope + category + name), module path |
| `Capability` | A capability: id (scope + domain + name), module path, description |
| `Request` / `ResolvedRequest` | A capability request: capability id, requesters, active flag |
| `OptionSchema` | Upstream option metadata: path, scope, type, default, description, example, internal, readOnly |
| `OptionDefinition` | One definition of an option: path, value, source, priority, layer, scope, host, user, profile, condition, definition type |
| `EffectiveValue` | The merged result: path, value, contributing definitions |
| `Provenance` | All definitions + effective value for one option path |
| `Contribution` | One merged element traced back to its definition |
| `SourceLocation` | File + span (start/end line/column) |
| `Ownership` | Option path + owner module + status |
| `Layer` | Profile / Host / User / Capability / Program / Desktop / External |
| `Priority` | Numeric priority (i16); lower wins |
| `Condition` | mkIf expression + evaluated value + satisfied flag |
| `MutationIntent` | Typed user intent for the write engine |
| `Transaction` | Planned/applied mutation: id, intents, file changes, validation, diff, status |
| `ValidationReport` | Tier + findings + passed flag |
| `Fingerprint` | git HEAD, working tree hash, flake.lock hash, schema version, index schema version |

---

## Key enums

### Layer

```text
Profile | Host | User | Capability | Program | Desktop | External
```

### DefinitionType

```text
Literal | MkDefault | MkForce | MkIf | MkMerge | MkOverride | Expression | Unknown
```

### NixType

```text
Bool | Int | Float | Str | Path | List | Attrs | AttrsOf | Submodule | Package | Enum | Function | Raw | Unknown
```

### Scope

```text
System | Home
```

### OwnershipStatus

```text
Owned | Unowned | WrongOwner | Ambiguous | External
```

### TransactionStatus

```text
Planned | Validating | Ready | Applied | Rejected | Failed | RolledBack
```

### ValidationTier

```text
Structural | NixEval | FlakeCheck | Build
```

### ModuleKind

```text
Capability | Program | Profile | Desktop | Aggregate | Core
```

---

## Contract types

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub struct RepositoryContract { schema_version, namespaces, requester_kinds, ownership_inferred_from_filename }
pub enum RequesterKind { Program, Profile, Capability, Host, User }
pub enum NamespaceKind { Meta, Facts, Identity, Request, Application, Desktop, Debug }
pub fn parse_requester(requester: &str) -> Result<RequesterKind, ModelError>
```

---

## See also

- [Architecture](../architecture.md)
- [Provenance](provenance.md)
- [Ownership](ownership.md)
