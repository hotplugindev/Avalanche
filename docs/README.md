# Avalanche Documentation

Avalanche is a native desktop configuration manager for structured **NixOS + Home Manager** repositories. It provides a graphical and command-line interface for managing a typed `gb.*` configuration layer while keeping the underlying Nix repository as the single source of truth.

> **Nix is the database. The index is a cache. Rust is the configuration engine. The GUI is a client.**

Avalanche does not replace Nix. It makes a structured Nix repository easier, safer, and more understandable to operate. It models more than configuration values: hosts, users, profiles, applications, capabilities, capability requests, ownership, implementation, provenance, priorities, `mkDefault` layering, host overrides, hardware conditions, upstream NixOS/Home Manager options, source locations, imports and registration, Git state, and validation state.

---

## Reading order

1. [Getting Started](getting-started.md) - install, run, first commands
2. [Architecture](architecture.md) - how the system is put together
3. [Concepts](concepts/) - the mental model (layering, provenance, ownership, transactions, safety)
4. [Features](features/) - every capability, with implementation status
5. [Reference](reference/) - namespaces, grammar, priorities, errors
6. [Contributing](contributing.md) - development workflow

---

## Status badges

Every feature page carries an explicit implementation status:

| Badge | Meaning |
|---|---|
| **Implemented** | Shipped, tested, usable today |
| **Partial** | Some sub-capabilities exist, others pending |
| **Not yet implemented** | Designed and documented, awaiting a phase |

---

## Documentation map

### Getting started / architecture / contributing

| Document | Contents |
|---|---|
| [Getting Started](getting-started.md) | Install from source and from a flake, first CLI run, GUI status |
| [Architecture](architecture.md) | Golden architecture, crate responsibilities, data flow, invariants |
| [Contributing](contributing.md) | Development setup, testing, code style, review process |

### Concepts

| Document | Contents |
|---|---|
| [Layering & Priority](concepts/layering-and-priority.md) | Layer model, priority arithmetic, `mkDefault`, reset semantics |
| [Provenance](concepts/provenance.md) | Definitions, winning value, contributions, effective derivation |
| [Ownership](concepts/ownership.md) | Owner vs requester vs implementer, ownership states |
| [Transactions](concepts/transactions.md) | Mutation lifecycle, planning, apply, rollback |
| [Safety](concepts/safety.md) | Architectural invariants, fail-closed behavior |

### Features - engines & foundations

| Document | Status |
|---|---|
| [Repository Contract](features/repository-contract.md) | Implemented |
| [Domain Model](features/domain-model.md) | Implemented |
| [Nix Engine](features/nix-engine.md) | Implemented |
| [Source Engine](features/source-engine.md) | Implemented |
| [Repository Index](features/repository-index.md) | Implemented |

### Features - analysis

| Document | Status |
|---|---|
| [Provenance](features/provenance.md) | Implemented |
| [Ownership](features/ownership.md) | Implemented |
| [Request Graph](features/request-graph.md) | Implemented |
| [Repo Doctor](features/repo-doctor.md) | Implemented |

### Features - interfaces

| Document | Status |
|---|---|
| [CLI](features/cli.md) | Implemented |
| [GUI](features/gui.md) | Partial |

### Features - mutation

| Document | Status |
|---|---|
| [Write Engine](features/write-engine.md) | Partial |
| [Validation](features/validation.md) | Implemented |
| [Module Generation](features/module-generation.md) | Implemented |
| [Repository Initialization](features/repository-init.md) | Implemented |

### Features - management (planned / partial)

| Document | Status |
|---|---|
| [Application Manager](features/application-manager.md) | Partial |
| [Capability Manager](features/capability-manager.md) | Not yet implemented |
| [Profiles](features/profiles.md) | Not yet implemented |
| [Hosts](features/hosts.md) | Not yet implemented |
| [Users](features/users.md) | Not yet implemented |
| [Generic Option Editor](features/option-editor.md) | Not yet implemented |
| [Computed Options](features/computed-options.md) | Partial |
| [Specialized Editors](features/specialized-editors.md) | Not yet implemented |
| [Raw Nix Editor](features/raw-nix-editor.md) | Not yet implemented |
| [Git Integration](features/git.md) | Partial |
| [Import / Export / Share](features/import-export.md) | Not yet implemented |

### Reference

| Document | Contents |
|---|---|
| [`gb.*` Namespaces](reference/gb-namespaces.md) | Every registered namespace and its semantics |
| [Requester Grammar](reference/requester-grammar.md) | Canonical plural naming rules |
| [Priority Values](reference/priority-values.md) | Priority constants and comparison |
| [Error Messages](reference/error-messages.md) | Error taxonomy and context rules |

---

## Implementation status at a glance

| Phase | Name | Status |
|---|---|---|
| 0 | Repository Contract | Implemented |
| 1 | Rust Workspace | Implemented |
| 2 | Domain Model | Implemented |
| 3 | Nix Engine | Implemented |
| 4 | Source Engine | Implemented |
| 5 | Repository Index | Implemented |
| 6 | Provenance | Implemented |
| 7 | Ownership | Implemented |
| 8 | Request Graph | Implemented |
| 9 | Repo Doctor | Implemented |
| 10 | Read-Only CLI | Implemented |
| 11 | Read-Only GUI | Partial |
| 12 | Mutation Planning | Implemented |
| 13 | Transactional Write Engine | Partial |
| 14 | Validation | Implemented |
| 15 | Application Manager | Partial |
| 16 | Module Generation | Implemented |
| 17 | Repository Initialization | Implemented |
| 18 | Capability Manager | Not yet implemented |
| 19 | Profiles | Not yet implemented |
| 20 | Hosts | Not yet implemented |
| 21 | Users | Not yet implemented |
| 22 | Generic Option Editor | Not yet implemented |
| 23 | Computed Options | Partial |
| 24 | Specialized Desktop Editors | Not yet implemented |
| 25 | Raw Nix Editor | Not yet implemented |
| 26 | Git | Partial |
| 27 | Import / Export / Share | Not yet implemented |
| 28 | Hardening | Not yet implemented |
| 29 | Release | Not yet implemented |

The full ordered plan with exit criteria lives in [`ROADMAP.md`](../ROADMAP.md).

---

## License

Avalanche is licensed under the **Apache License 2.0**. See [`LICENSE`](../LICENSE).
