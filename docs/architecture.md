# Architecture

Avalanche is a configuration engine with a desktop client. The desktop GUI is **not** the core system. The core is a Rust engine that understands a structured Nix repository and provides semantic inspection, source provenance, ownership, dependency/request graphs, safe mutation, validation, and Git integration. The repository remains the source of truth.

---

## Golden architecture

```text
                         USER
                          |
                          v
                  +---------------+
                  | Avalanche GUI |
                  +-------+-------+
                          |
                    typed API only
                          |
                          v
                 +------------------+
                 | Avalanche Core   |
                 +--------+---------+
                          |
        +------------------+------------------+
        v                  v                  v
      Index             Ownership           Graph
        |                  |                  |
        +------------------+------------------+
                          v
                     Write Planner
                          |
                          v
                     Source Engine
                          |
                          v
                      Transaction
                          |
                          v
                      Validation
                          |
                          v
                         Diff
                          |
                          v
                        Apply
                          |
                          v
                  Nix Repository
                  SOURCE OF TRUTH
```

The frontend never writes Nix directly. Every mutation goes through the Rust configuration engine.

---

## Core principles

1. **Repository is the database.** Avalanche never becomes authoritative over the Nix repository. The index can be deleted and rebuilt.
2. **GUI is a client.** The frontend submits typed intent. It does not parse Nix, determine ownership, select files, execute arbitrary Nix mutations, or implement validation rules.
3. **Index is derived.** The index combines static source analysis and Nix evaluation.
4. **Provenance is first-class.** Effective values are insufficient; the system retains definitions and their origin.
5. **Ownership is explicit.** Owner, requester, and implementer are separate concepts.
6. **Mutations are transactional.** No operation should partially modify the repository.
7. **Repository contract is explicit.** A managed repository declares conformance to `schemas/contract.nix` via `gb.schemaVersion`. Avalanche validates a repository against this contract before indexing; a repository that violates it is rejected, not guessed at.

---

## Workspace layout

```text
avalanche/
├── apps/
│   └── desktop/
│       ├── frontend/          # React + TypeScript client
│       └── src-tauri/         # Tauri 2 shell (thin command handlers)
├── crates/
│   ├── avalanche-model/       # domain types
│   ├── avalanche-core/        # application orchestration
│   ├── avalanche-index/       # repository discovery + derived model
│   ├── avalanche-nix/         # Nix evaluation + process interaction
│   ├── avalanche-source/      # Nix parsing, AST, safe source editing
│   ├── avalanche-ownership/   # ownership rules and queries
│   ├── avalanche-graph/       # request/dependency graph
│   ├── avalanche-write/       # intent-based mutation + transactions
│   ├── avalanche-validate/    # validation tiers
│   ├── avalanche-git/         # git operations
│   ├── avalanche-doctor/      # repository diagnostics + repairs
│   └── avalanche-cli/         # CLI adapter around the same core
├── schemas/                   # contract.nix
├── fixtures/                  # real Nix fixture repositories
├── docs/                      # this documentation tree
└── flake.nix                  # development shell
```

---

## Crate responsibilities

### `avalanche-model`

Domain types only. No filesystem or UI behavior. Holds `Repository`, `Host`, `User`, `Profile`, `Module`, `Application`, `Capability`, `Request`, `OptionSchema`, `OptionDefinition`, `EffectiveValue`, `SourceLocation`, `Ownership`, `Provenance`, `Layer`, `Priority`, `Condition`, `Transaction`, `ValidationReport`, and the repository contract (`schemas/contract.nix` mirrored in Rust).

### `avalanche-core`

Application orchestration. Exposes `ConfigurationEngine`, which coordinates index, write, validation, Git, and Doctor, enforces high-level invariants, and provides the API consumed by both the CLI and Tauri.

### `avalanche-index`

Repository discovery and derived model construction. Builds a `StaticIndex` (from the source engine) and a `SemanticIndex` (from the Nix engine), merges them into a `RepositoryIndex`, computes a repository fingerprint, detects staleness, and builds the `Provenance` map.

### `avalanche-nix`

Nix evaluation and process interaction. A structured process abstraction (no shell interpolation), `nix eval`, `nix flake check`, `nix build`, JSON decoding, Nix type representation, option extraction, assertions, host and Home Manager evaluation, and external flake discovery. Answers *what does the configuration mean?* - not *where should I edit it?*

### `avalanche-source`

Nix parsing, AST/source locations, and safe source editing. A hand-written lexer + recursive-descent parser producing a typed AST with source spans, import detection with cycle detection, assignment extraction and classification, formatting, and source patch generation. Answers *where is this configuration written and how can it be safely changed?*

### `avalanche-ownership`

Ownership rules and queries. Builds an `OwnershipRegistry` from provenance, modules, and requests; classifies each option as Owned / Unowned / WrongOwner / Ambiguous / External; tracks requesters and implementers.

### `avalanche-graph`

Request/dependency graph. Nodes (Profile, Capability, Program, Module, Option, Host, User, Desktop, Aggregate, Core) and edges (Requests, Implements, Owns, Overrides, Imports). Provides `why`, `dependents`, `requesters`, `implementation`, `owners`.

### `avalanche-write`

Intent-based mutation and transactions. Resolves intents into concrete edits and file creations, plans diffs, applies atomically, and rolls back on failure.

### `avalanche-validate`

Validation tiers: structural (parse), Nix eval + assertions, `nix flake check`, and `nix build` dry-run.

### `avalanche-git`

Git operations: status, diff, history, branches, commit, pull, push, rollback, conflicts. Independent from write planning.

### `avalanche-doctor`

Repository diagnostics and proposed repairs. Consumes the index and reports schema, registration, ownership, path, quality, and state issues, producing repair proposals (never silent fixes).

### `avalanche-cli`

CLI adapter around the same core. No duplicated configuration logic.

---

## Data flow

```text
Nix repository
      |
      +--> static parser (avalanche-source)
      |
      +--> Nix evaluation (avalanche-nix)
      |
      v
RepositoryIndex (avalanche-index)
      |
      +--> Provenance (definitions, winning value, contributions)
      +--> Ownership (registry)
      +--> Request Graph (why/dependents)
      +--> Doctor (diagnostics)
      |
      v
Core queries + Write planning
      |
      v
Source patch generation -> Transaction -> Validation -> Diff -> Apply
      |
      v
Nix repository (updated)
```

The index is always rebuildable from the repository. Nothing in the index is authoritative.

---

## Static vs semantic information

Static analysis can discover: file, line, source span, imports, expression, registration, `mkDefault`, assignment.

Nix evaluation can discover: option schema, effective values, types, defaults, descriptions, examples, assertions, conditions.

Neither is sufficient alone; the index combines both.

---

## Assignment classification

Assignments are classified:

```text
DirectLiteral       -> automatic editing allowed
DirectExpression    -> allowed only when understood
GeneratedPattern    -> specialized safe editing
Computed            -> usually raw/specialized
Unknown             -> raw
```

Avalanche must never silently transform unfamiliar expressions. See [Source Engine](features/source-engine.md).

---

## Architectural invariants

1. Repository is source of truth.
2. Index is disposable.
3. GUI does not write Nix directly.
4. Every mutation is transactional.
5. Ownership is explicit.
6. Provenance is retained.
7. Effective value does not replace definitions.
8. Unknown expressions are never silently rewritten.
9. External dependencies are never edited.
10. CLI and GUI share one engine.
11. Doctor does not silently repair.
12. Specialized editors use the same write engine.

These are enforced in code and in review. See [Safety](concepts/safety.md).

---

## Architectural goal

The final system should make this question answerable for every important configuration value:

> **What is this, where did it come from, why is it enabled, who owns it, what overrides it, and what exact Nix change will happen if I modify it?**
