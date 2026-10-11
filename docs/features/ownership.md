# Ownership

**Status: Implemented** (Phase 7)

Ownership answers: **Who owns this upstream setting?**

---

## Registry

```rust
pub struct OwnershipRegistry {
    records: HashMap<String, OwnershipRecord>,
    requesters: HashMap<String, Vec<String>>,
    implementations: HashMap<String, Vec<String>>,
}
```

Built by `build_ownership_registry(provenance, modules, requests)`.

---

## Classification algorithm

For each option path:

1. Collect definitions whose layer is `Capability` or `Program`.
2. Map each definition's source file to a registered `Module`.
3. Classify:

| Condition | Status |
|---|---|
| Zero claiming modules, all definitions External | `External` |
| Zero claiming modules, otherwise | `Unowned` |
| Two or more distinct module paths claim it | `Ambiguous` |
| One module claims it, but another Capability/Program definition exists outside it | `WrongOwner` |
| Exactly one module, no outsiders | `Owned` |

---

## Queries

| Method | Returns |
|---|---|
| `resolve(path)` | OwnershipStatus |
| `owner_of(path)` | Option<&str> (module path, only if Owned) |
| `can_auto_edit(path)` | bool (true only if Owned) |
| `requesters_of(path)` | &[String] |
| `implementations_of(path)` | &[String] |
| `ambiguous_options()` | Vec<&str> |
| `external_options()` | Vec<&str> |
| `unowned_options()` | Vec<&str> |

---

## Ownership service

```rust
pub struct OwnershipService { registry: OwnershipRegistry }
```

Delegates to the registry. Used by the core's `plan_mutation` to gate writes.

---

## Write gating

Before any mutation of an upstream option:

```text
resolve ownership
    ↓
if status != Owned → reject with OwnershipDenied
    ↓
proceed with edit
```

This prevents the GUI from silently editing an option that belongs to a capability it does not control.

---

## CLI exposure

```bash
avalanche owners services.pipewire.enable
```

Outputs: option path, owner module, owner kind, status, contributing files.

---

## Exit criteria

Avalanche can answer: *Who owns this upstream setting?*

---

## See also

- [Concepts: Ownership](../concepts/ownership.md)
- [Provenance](provenance.md)
- [Write Engine](write-engine.md)
