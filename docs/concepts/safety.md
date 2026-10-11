# Safety

**Status: Implemented** (architectural invariants enforced across all phases)

Avalanche handles configuration that can affect an entire operating system.
Safety is not a feature; it is the set of invariants the whole system is built
around.

---

## Architectural invariants

1. **Repository is source of truth.** The index is a cache. Never introduce a
   dependency where `Avalanche database > Nix repository`.
2. **Index is disposable.** It can always be deleted and rebuilt from the
   repository.
3. **GUI does not write Nix directly.** Every mutation goes through the Rust
   engine.
4. **Every mutation is transactional.** No partial writes.
5. **Ownership is explicit.** Never inferred from filenames.
6. **Provenance is retained.** Never flatten definitions into one effective
   value.
7. **Effective value does not replace definitions.**
8. **Unknown expressions are never silently rewritten.** Classify as
   Computed/Unknown and route to raw editing.
9. **External dependencies are never edited.** Configure the local layer.
10. **CLI and GUI share one engine.** No duplicated configuration logic.
11. **Doctor does not silently repair.** It proposes; the user applies.
12. **Specialized editors use the same write engine.**

---

## Fail-closed behavior

Dangerous operations fail closed:

```text
Unknown owner       → no automatic write
Unknown expression  → raw editor
External source     → local configuration only
Validation failure  → transaction blocked
Stale index         → re-index before write
Ambiguous scope     → require explicit choice
```

---

## Nix editing rules

Never use naive string replacement for arbitrary Nix source. Use:

- AST/source spans
- known generated patterns
- structured templates
- specialized editors

Classify unfamiliar expressions as `Computed` or `Unknown` and use a raw
fallback. Never silently transform an unknown expression.

---

## Security

Never:

- execute arbitrary strings through a shell when a structured process API
  exists
- trust paths from imported bundles
- allow path traversal
- silently run user-provided commands
- write outside the managed repository

All Nix interactions go through a structured process API with configurable
binary path, timeout, and experimental-feature flags. No shell interpolation.

---

## Error handling

Errors preserve context:

Bad:

```text
failed
```

Good:

```text
Failed to update programs.git.enable:
the current definition in hosts/pc/programs.nix
changed after the index was created.
Repository was not modified.
```

Errors explain:

- what failed
- why
- what was not changed
- what the user can do next

See [Error Messages reference](../reference/error-messages.md).

---

## Highest-priority rule

> If an implementation choice makes Avalanche easier to build but less
> trustworthy as a configuration manager, do not choose it.

Avalanche's core value is safe, understandable configuration management.

---

## See also

- [Architecture](../architecture.md)
- [Transactions](transactions.md)
- [Provenance](provenance.md)
- [Ownership](ownership.md)
