# Avalanche

Avalanche is a native desktop configuration manager for structured NixOS + Home Manager repositories.

It provides a graphical and command-line interface for managing a typed `gb.*` configuration layer while keeping the underlying Nix repository as the single source of truth.

Avalanche does not replace Nix. It makes a structured Nix repository easier, safer, and more understandable to operate.

## Core philosophy

> **Nix is the database. The index is a cache. Rust is the configuration engine. The GUI is a client.**

The repository remains authoritative. Avalanche derives an index from it and uses that model for inspection, provenance, ownership, dependency analysis, safe mutation, and validation.

## What Avalanche understands

Avalanche models more than configuration values:

- hosts
- users
- profiles
- applications
- capabilities
- capability requests
- ownership
- implementation
- provenance
- priorities
- `mkDefault` layering
- host overrides
- hardware conditions
- upstream NixOS/Home Manager options
- source locations
- imports and registration
- Git state
- validation state

## Architecture

```text
                    ┌─────────────────────────┐
                    │      Avalanche GUI      │
                    │  React + TypeScript     │
                    └────────────┬────────────┘
                                 │
                            Tauri IPC
                                 │
                                 ▼
                    ┌─────────────────────────┐
                    │    Avalanche Rust Core  │
                    │                         │
                    │ Model / Index / Write   │
                    │ Ownership / Graph       │
                    │ Validation / Git        │
                    └────────────┬────────────┘
                                 │
                    ┌────────────┴────────────┐
                    ▼                         ▼
                 Nix Engine              Source Engine
                    │                         │
                    └────────────┬────────────┘
                                 ▼
                       Nix configuration repo
                       (source of truth)
```

The frontend does not edit Nix files directly.

Every mutation goes through the Rust configuration engine.

## Technology

| Area | Choice |
|---|---|
| Core | Rust |
| Desktop | Tauri 2 |
| Frontend | TypeScript + React |
| UI | Tailwind + shadcn/ui |
| Server state | TanStack Query |
| Local UI state | Zustand |
| Serialization | serde / JSON |
| Nix integration | Nix CLI |
| Source analysis | tree-sitter-Nix / compatible AST tooling |
| Version control | Git |
| Testing | Rust unit/integration tests + real Nix fixtures |

## Repository contract

A managed repository follows the Avalanche configuration contract.

The existing GB namespaces are retained:

```text
gb.schemaVersion
gb.programs.*
gb.requires.*
gb.capabilities.*
gb.profiles.*
gb.host.*
gb.user.*
gb.nix.*
```

The contract defines:

- ownership
- request naming
- capability registration
- module registration
- aggregate imports
- profile semantics
- host overrides
- user scoping
- external module boundaries

## Project structure

```text
avalanche/
├── Cargo.toml
├── package.json
├── flake.nix
├── README.md
├── FEATURES.md
├── ARCHITECTURE.md
├── CONTRIBUTING.md
├── AGENTS.md
├── ROADMAP.md
│
├── apps/
│   └── desktop/
│       ├── src/
│       │   ├── components/
│       │   ├── features/
│       │   ├── pages/
│       │   ├── queries/
│       │   ├── commands/
│       │   ├── stores/
│       │   └── types/
│       └── src-tauri/
│           └── src/
│
├── crates/
│   ├── avalanche-model/
│   ├── avalanche-core/
│   ├── avalanche-index/
│   ├── avalanche-nix/
│   ├── avalanche-source/
│   ├── avalanche-ownership/
│   ├── avalanche-graph/
│   ├── avalanche-write/
│   ├── avalanche-validate/
│   ├── avalanche-git/
│   ├── avalanche-doctor/
│   └── avalanche-cli/
│
├── schemas/
├── fixtures/
├── tests/
└── docs/
```

## Development order

Avalanche should not begin with the GUI.

Build in this order:

1. Repository contract
2. Domain model
3. Nix execution layer
4. Nix source parser
5. Repository index
6. Provenance
7. Ownership
8. Request graph
9. Repo Doctor
10. Read-only CLI
11. Read-only GUI
12. Transaction/write engine
13. Validation
14. Applications
15. Capabilities
16. Profiles
17. Hosts
18. Users
19. Generic Nix option editor
20. Specialized desktop editors
21. Git integration
22. Import/export/share
23. Production hardening

## Important repository preparation

Before write support, per-user Home Manager configuration must be made unambiguous.

Preferred structure:

```text
hosts/<host>/home/<username>.nix
```

instead of a single:

```text
hosts/<host>/home.nix
```

The repository audit also identified known inconsistencies such as Spotify path/file drift, a Boxflat description mismatch, a Moonlight typo, and an empty gaming profile. These should either be fixed or captured as Repo Doctor regression fixtures.

## CLI

The CLI uses the same Rust engine as the GUI.

Examples:

```bash
avalanche inspect host pc
avalanche inspect app firefox
avalanche inspect capability audio.pipewire

avalanche why audio.pipewire
avalanche dependents audio.pipewire
avalanche owners programs.git

avalanche doctor
avalanche diff
```

Mutations eventually include:

```bash
avalanche set programs.git.enable true
avalanche capability enable audio.pipewire
```

## Safety principles

1. Nix repository is the source of truth.
2. The index is disposable.
3. The GUI never directly writes Nix.
4. Every mutation is transactional.
5. Ownership is explicit.
6. Provenance is retained.
7. Unknown expressions are never silently rewritten.
8. External dependencies are never edited.
9. CLI and GUI use the same engine.
10. Repo Doctor proposes repairs instead of silently applying them.

## Status

Phase 4 (Source Engine) is complete. Avalanche can parse Nix source files, extract assignments with source spans, classify expressions, detect imports, generate source patches, and answer "where is this value actually written?" without modifying the repository.

The next milestone is Phase 5 (Repository Index) — building the complete derived repository model.
