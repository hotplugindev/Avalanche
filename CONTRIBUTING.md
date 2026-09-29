# Contributing to Avalanche

Thank you for contributing to Avalanche.

Avalanche is a configuration-management system where correctness matters more than UI speed. Contributions should preserve the repository as the source of truth and should avoid introducing a second, implicit configuration system.

---

# 1. Before You Start

Read:

1. `README.md`
2. `ARCHITECTURE.md`
3. `FEATURES.md`
4. `AGENTS.md`
5. `ROADMAP.md`

For architecture changes, also read the relevant documents under `docs/architecture/`.

---

# 2. Core Rules

## Rule 1 — Do not duplicate configuration logic

If the GUI needs new behavior, put the semantic behavior in the Rust engine.

Do not implement one version in React and another in Rust.

Bad:

```text
React decides which file owns programs.git
Rust independently decides another file
```

Good:

```text
React
  ↓
typed command
  ↓
Rust engine
  ↓
ownership/index
```

---

## Rule 2 — Never make the index authoritative

The index must always be rebuildable.

Do not introduce features that only work because a local cache contains information not present in the repository.

---

## Rule 3 — Never silently rewrite unknown Nix

If the source structure is not understood:

- classify it as unknown/computed
- provide a raw editor
- or implement a specialized editor

Do not guess.

---

## Rule 4 — Preserve provenance

Whenever adding a feature that changes configuration modeling, ask:

- Where was this defined?
- At what priority?
- On which host?
- For which user?
- Which profile?
- Under which condition?

If the feature destroys provenance, it is probably architecturally wrong.

---

## Rule 5 — Ownership matters

Do not add a shortcut that writes an upstream option simply because the UI currently has access to it.

The write engine must resolve ownership.

---

# 3. Development Environment

The project is intended to be developed inside a reproducible Nix environment.

The repository should eventually expose all development dependencies through `flake.nix`.

Typical tools include:

```text
Rust
cargo
rustfmt
clippy
Node.js
package manager
Nix
Git
Tauri dependencies
```

Use the repository's development shell rather than manually installing project-specific dependencies where possible.

---

# 4. Build

Rust:

```bash
cargo build --workspace
```

Tests:

```bash
cargo test --workspace
```

Formatting:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Frontend commands should be defined in the root `package.json` and desktop app package.

---

# 5. Testing

Every behavior change should have tests.

## Unit tests

Use unit tests for:

- domain types
- parsers
- classifiers
- ownership rules
- graph operations
- validation rules

## Integration tests

Use real fixture repositories for:

- indexing
- Nix evaluation
- AST editing
- module generation
- transactions
- validation

## Regression tests

Every discovered repository bug should become a fixture or regression test when practical.

---

# 6. Fixture Repositories

Fixtures should be intentionally small.

Examples:

```text
fixtures/
├── simple/
├── mkdefault/
├── conditional/
├── multi-host/
├── multi-user/
├── computed/
├── external/
└── broken/
```

A fixture should test one architectural behavior clearly.

---

# 7. Adding a Feature

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
9. Update FEATURES.md
10. Update ROADMAP.md
```

Do not start with the UI.

---

# 8. Adding a New Write Operation

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

# 9. Adding a New Option Renderer

A renderer must define:

```text
supported Nix type
display representation
serialization
deserialization
validation
fallback behavior
```

The renderer must never bypass the write engine.

For example:

```text
UI
 ↓
setOption(...)
 ↓
transaction
 ↓
AST edit
```

not:

```text
UI
 ↓
edit Nix string
```

---

# 10. Adding a Specialized Editor

Use a specialized editor when a value has domain semantics that generic Nix forms cannot represent well.

Examples:

- MangoWC monitor strings
- MangoWC binds
- widget ordering
- complex desktop settings

The specialized editor should parse into a semantic representation and then produce a normal Avalanche mutation.

---

# 11. Architecture Decision Records

Significant architectural changes should use an ADR.

Store them under:

```text
docs/adr/
```

Example:

```text
docs/adr/0001-index-is-cache.md
docs/adr/0002-transactional-writes.md
```

An ADR should contain:

- context
- decision
- alternatives
- consequences

---

# 12. Commits

Prefer small, focused commits.

Example:

```text
feat(index): add mkDefault provenance
feat(write): add host override mutation
fix(doctor): detect orphan modules
feat(ui): add capability graph
```

Avoid mixing unrelated refactors with feature changes.

---

# 13. Pull Requests

A PR should explain:

- what changed
- why it changed
- architecture affected
- tests added
- known limitations
- documentation updated

For UI changes, include screenshots when appropriate.

For write-engine changes, include example diffs.

---

# 14. Review Checklist

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

- Is `FEATURES.md` updated?
- Is the architecture affected?
- Is an ADR needed?

---

# 15. What Not To Do

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

# 16. Definition of Done

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

---

# 17. Security and Safety

Avalanche handles configuration that can affect an entire operating system.

Treat all external input as untrusted.

In particular:

- do not execute arbitrary user-provided shell commands as part of normal parsing
- do not interpolate unchecked values into shell commands
- prefer structured process APIs
- validate paths
- validate repository boundaries
- never allow an import/export operation to escape its target repository
- make destructive operations explicit

---

# 18. Contribution Philosophy

Prefer:

```text
simple
explicit
testable
reversible
```

over:

```text
clever
implicit
magical
hard to inspect
```

Avalanche should make configuration behavior more understandable, not more magical.
