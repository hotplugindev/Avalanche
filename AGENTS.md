# Avalanche — AGENTS.md

This file defines the rules for AI coding agents and automated development agents working on Avalanche.

Agents must treat the repository as a safety-critical configuration-management project.

---

# 1. Primary Objective

Make correct, maintainable changes without weakening Avalanche's architectural guarantees.

The agent must optimize for:

1. correctness
2. safety
3. architectural consistency
4. tests
5. clarity
6. implementation speed

Speed must never override correctness.

---

# 2. Source of Truth

The Nix repository managed by Avalanche is the source of truth.

Never introduce an architectural dependency where:

```text
Avalanche database
    > Nix repository
```

The correct relationship is:

```text
Nix repository
      ↓
Avalanche index
      ↓
Avalanche UI
```

The index must be rebuildable.

---

# 3. Read Before Editing

Before modifying architecture or core code, read:

```text
README.md
ARCHITECTURE.md
FEATURES.md
CONTRIBUTING.md
ROADMAP.md
```

Then inspect the relevant source files.

Do not assume that a feature described in documentation already exists.

---

# 4. Architecture Boundaries

Respect these boundaries:

```text
GUI
  ↓
Tauri API
  ↓
Avalanche Core
  ↓
Domain services
  ↓
Nix / Source / Git
```

The frontend must not implement configuration semantics.

Tauri handlers should be thin.

Core logic belongs in Rust services.

---

# 5. Rust Crate Responsibilities

## `avalanche-model`

Domain types.

Do not put filesystem or UI behavior here.

## `avalanche-core`

Application orchestration.

## `avalanche-index`

Repository discovery and derived model construction.

## `avalanche-nix`

Nix evaluation and Nix process interaction.

## `avalanche-source`

Nix parsing, AST/source locations, and safe source editing.

## `avalanche-ownership`

Ownership rules and queries.

## `avalanche-graph`

Request/dependency graph.

## `avalanche-write`

Intent-based mutation and transactions.

## `avalanche-validate`

Validation tiers.

## `avalanche-git`

Git operations.

## `avalanche-doctor`

Repository diagnostics and proposed repairs.

## `avalanche-cli`

CLI adapter around the same core.

---

# 6. Never Bypass the Write Engine

Agents must never implement a GUI feature by directly modifying:

```text
Nix files
JSON index
Git state
```

The normal path is:

```text
UI intent
 ↓
Avalanche command
 ↓
write service
 ↓
transaction
 ↓
validation
 ↓
diff
 ↓
apply
```

---

# 7. Nix Editing Rules

Never use naive string replacement for arbitrary Nix source.

Use:

- AST/source spans
- known generated patterns
- structured templates
- specialized editors

Classify unfamiliar expressions as:

```text
Computed
Unknown
```

and use a raw fallback.

Never silently transform an unknown expression.

---

# 8. Ownership Rules

Before modifying an upstream option, determine its owner.

Do not infer:

```text
owner = nearest filename
```

Ownership must come from the repository model/contract.

If ownership is:

```text
Ambiguous
Unowned
WrongOwner
External
```

automatic modification should not proceed without an explicit safe path.

---

# 9. Provenance Rules

Never discard:

- source file
- source span
- layer
- priority
- host
- user
- profile
- condition
- definition type

When modifying merged options, preserve contribution-level provenance where possible.

---

# 10. `mkDefault`

Do not represent `mkDefault` as merely:

```text
isProfileDefault = true
```

It is a Nix priority.

The model must retain priority information.

The UI can project that into:

```text
Profile default
Host override
Effective
```

---

# 11. Multi-Host Rules

"Reset to profile default" means:

```text
delete host override
```

It does not mean:

```text
write false
```

Agents must not implement reset semantics incorrectly.

---

# 12. External Dependencies

Never modify external flake inputs directly.

If an option originates from an external module:

```text
origin = external
```

configure the local repository layer instead.

---

# 13. Index Freshness

Before a mutation:

1. inspect repository fingerprint
2. compare against indexed fingerprint
3. re-index if stale
4. only then plan the mutation

Never mutate from a known-stale index.

---

# 14. Transactions

Every mutation must be reversible.

Preferred flow:

```text
intent
 ↓
plan
 ↓
temporary changes
 ↓
format
 ↓
validate
 ↓
diff
 ↓
apply
```

A failed validation must not leave a half-written repository.

---

# 15. Validation

Use the smallest validation sufficient during development, but ensure the final behavior follows the appropriate tier:

```text
Tier 1: structural
Tier 2: nix evaluation
Tier 3: flake check
Tier 4: build
```

Do not claim a full build passed if only evaluation was run.

---

# 16. Tests

For core behavior, prefer real Nix fixture repositories.

A good test is:

```text
fixture repository
+
intent
=
expected model / diff / final repository
```

Add regression fixtures for bugs found in the real configuration.

---

# 17. Before Writing Code

Agents should:

1. identify the relevant architecture layer
2. inspect existing implementations
3. search for similar behavior
4. determine whether the change needs an ADR
5. determine which tests are required
6. implement the smallest coherent change

Do not create a new subsystem when an existing one is appropriate.

---

# 18. Avoid Overengineering

Do not introduce:

- generic plugin frameworks
- distributed architecture
- remote services
- complex event buses
- speculative abstractions

unless a concrete requirement needs them.

The initial system is a local desktop application.

---

# 19. Frontend Rules

The frontend should:

- consume structured data
- submit typed commands
- display validation
- display provenance
- display diffs

The frontend should not:

- parse Nix
- determine ownership
- decide target files
- perform Git mutations directly
- implement validation semantics

---

# 20. Documentation

When behavior changes, update:

```text
FEATURES.md
```

When architecture changes, update:

```text
ARCHITECTURE.md
```

When contributor workflow changes, update:

```text
CONTRIBUTING.md
```

When roadmap status changes, update:

```text
ROADMAP.md
```

---

# 21. Git

Agents should keep changes focused.

Do not make unrelated formatting changes.

Do not rewrite unrelated Nix configuration.

Do not commit generated artifacts unless the repository explicitly requires them.

---

# 22. Error Handling

Errors should preserve context.

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

Errors should explain:

- what failed
- why
- what was not changed
- what the user can do next

---

# 23. Security

Never:

- execute arbitrary strings through a shell when a structured process API exists
- trust paths from imported bundles
- allow path traversal
- silently run user-provided commands
- write outside the managed repository

---

# 24. Agent Completion Checklist

Before considering a task complete:

```text
[ ] Correct architectural layer
[ ] No duplicated configuration logic
[ ] Tests added
[ ] Failure path tested
[ ] Provenance preserved
[ ] Ownership respected
[ ] Transaction semantics respected
[ ] Documentation updated
[ ] Formatting passes
[ ] Lint passes
[ ] Tests pass
```

---

# 25. Highest-Priority Rule

If an implementation choice makes Avalanche easier to build but less trustworthy as a configuration manager, do not choose it.

Avalanche's core value is safe, understandable configuration management.
