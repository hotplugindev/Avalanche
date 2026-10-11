# Contributing

Thank you for contributing to Avalanche. Avalanche is a configuration-management system where correctness matters more than UI speed. Contributions should preserve the repository as the source of truth and should avoid introducing a second, implicit configuration system.

---

## Before you start

Read:

1. `README.md`
2. `docs/architecture.md`
3. `docs/features/` (especially the feature you are touching)
4. `AGENTS.md`
5. `ROADMAP.md`

For architecture changes, also read the relevant documents under `docs/` and consider an ADR under `docs/adr/` (when that directory exists).

---

## Core rules

### Rule 1 - Do not duplicate configuration logic

If the GUI needs new behavior, put the semantic behavior in the Rust engine. Do not implement one version in React and another in Rust.

### Rule 2 - Never make the index authoritative

The index must always be rebuildable. Do not introduce features that only work because a local cache contains information not present in the repository.

### Rule 3 - Never silently rewrite unknown Nix

If the source structure is not understood:

- classify it as unknown/computed
- provide a raw editor
- or implement a specialized editor

Do not guess.

### Rule 4 - Preserve provenance

Whenever adding a feature that changes configuration modeling, ask:

- Where was this defined?
- At what priority?
- On which host?
- For which user?
- Which profile?
- Under which condition?

If the feature destroys provenance, it is probably architecturally wrong.

### Rule 5 - Ownership matters

Do not add a shortcut that writes an upstream option simply because the UI currently has access to it. The write engine must resolve ownership.

---

## Development environment

Use the repository's development shell rather than manually installing project-specific dependencies where possible:

```bash
nix develop
```

The dev shell pins Rust, Node.js, and the native libraries required by the Tauri shell.

---

## Build, test, lint

Rust:

```bash
cargo build --workspace
cargo test  --workspace
cargo fmt   --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Frontend:

```bash
cd apps/desktop/frontend
npm install
npm run typecheck
npm run lint
npm run build
```

All gates must pass before a PR is considered ready.

---

## Testing

Every behavior change should have tests.

### Unit tests

Use unit tests for:

- domain types
- parsers
- classifiers
- ownership rules
- graph operations
- validation rules

### Integration tests

Use real fixture repositories for:

- indexing
- Nix evaluation
- AST editing
- module generation
- transactions
- validation

### Regression tests

Every discovered repository bug should become a fixture or regression test when practical.

Fixtures should be intentionally small and test one architectural behavior clearly. See `fixtures/`.

---

## Adding a feature

Use this process:

```text
1. Define domain behavior
2. Add/modify model
3. Add index information
4. Add ownership/provenance if relevant
5. Add command/query
6. Add tests
7. Add CLI support where useful
8. Add GUI
9. Update docs/features/
10. Update ROADMAP.md
```

Do not start with the UI.

---

## Adding a new write operation

New write operations must define:

- user intent
- target scope
- ownership rules
- layer semantics
- source edit strategy
- validation requirements
- rollback behavior
- expected diff

A write operation should be tested against a fixture repository.

---

## Code style

- Follow existing conventions in each crate.
- Mimic code style, use existing libraries and utilities.
- Do not add comments unless requested or required for safety-critical invariants.
- Keep changes focused. Do not mix unrelated refactors with feature changes.
- Never commit secrets or keys.
- Never introduce code that exposes or logs secrets.

---

## Review checklist

Reviewers should ask:

### Architecture

- Is this logic in the correct layer?
- Is the model still authoritative?
- Is the index still disposable?

### Safety

- Can this corrupt configuration?
- What happens on stale index?
- What happens on validation failure?
- Is rollback possible?

### Nix

- Does this preserve source structure?
- Does it handle unknown expressions safely?
- Does it preserve provenance?

### Ownership

- Is ownership respected?
- Could this create duplicate ownership?

### Tests

- Is there a fixture?
- Is the mutation tested?
- Is the failure mode tested?

### Documentation

- Is `docs/features/` updated?
- Is the architecture affected?
- Is an ADR needed?

---

## What not to do

Do not:

- add a hidden database of configuration values
- write Nix directly from React
- use string replacement for arbitrary Nix editing
- infer ownership from filenames alone
- silently repair the repository
- silently modify external dependencies
- treat `mkDefault` as just a boolean
- flatten provenance into one effective value
- make the GUI responsible for Nix semantics
- add a giant generic abstraction before a real use case exists

---

## Definition of done

A feature is done when:

- implementation exists
- tests exist
- failure behavior is defined
- transaction semantics are correct if it writes
- provenance is preserved
- ownership is respected
- documentation is updated
- CLI support exists where appropriate
- GUI support exists where appropriate
- formatting, lint, and tests pass

---

## License

Avalanche is licensed under the Apache License 2.0. Contributions are accepted under the same license.
