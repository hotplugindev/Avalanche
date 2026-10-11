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

## Install and run

### Prerequisites

| Tool | Why | Minimum |
|---|---|---|
| Rust (stable) + `cargo` | build the engine, CLI and tests | 1.80+ |
| Node.js + npm | build/lint/typecheck the desktop frontend | 22+ |
| Git | repository fingerprints and `avalanche init` | any recent |
| Nix (optional) | `--nix` full evaluation index and Tier 2–4 validation | 2.18+ with flakes |

The engine, CLI and tests run without Nix: the default index is built from static
source analysis only. Nix is needed only for `avalanche --nix …` (live evaluation)
and for the higher validation tiers.

### Development shell (recommended)

A reproducible shell is provided by `flake.nix` (Rust toolchain with `rustfmt`
and `clippy`, Node 22, and the GTK3/WebKitGTK native libraries the desktop shell
needs):

```bash
nix develop        # or: nix shell .#
cargo build --workspace
```

Without Nix, install the equivalents manually: a stable Rust toolchain, Node 22,
and — only if you intend to build the desktop app — the Tauri 2 system
libraries (on Debian/Ubuntu: `libwebkit2gtk-4.1-dev build-essential libssl-dev
libayatana-appindicator3-dev librsvg2-dev`).

### Build and test the engine

```bash
cargo build --workspace
cargo test  --workspace                 # unit + integration tests
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt   --all -- --check
```

### Run the CLI

The CLI is the fully supported command-line client and shares one engine with the
GUI. It needs no Nix for the default (static) index.

```bash
# point it at any conforming repository (defaults to the current directory)
cargo run -p avalanche-cli -- -r /path/to/nix-repo doctor
cargo run -p avalanche-cli -- -r /path/to/nix-repo inspect host pc
cargo run -p avalanche-cli -- -r /path/to/nix-repo why audio.pipewire
cargo run -p avalanche-cli -- -r /path/to/nix-repo owners programs.git.enable
```

Create a brand-new, contract-conforming repository and confirm it is clean:

```bash
cargo run -p avalanche-cli -- init ~/my-config --host pc --user alice
cargo run -p avalanche-cli -- -r ~/my-config doctor      # exits 0, zero errors
```

Useful flags: `-r/--repo <path>` (repository root), `--json` (machine output),
`--nix` (build the index through live Nix evaluation instead of static parsing).
Run `avalanche --help` for the full command list.

To install the binary on your PATH:

```bash
cargo install --path crates/avalanche-cli      # provides the `avalanche` command
```

### Run the desktop frontend

The React/TypeScript client lives in `apps/desktop/frontend`.

```bash
cd apps/desktop/frontend
npm install
npm run typecheck        # tsc --noEmit
npm run lint             # eslint
npm run build            # tsc && vite build  →  dist/
npm run dev              # Vite dev server on http://localhost:1420
```

The frontend is a thin client: every screen calls a typed command through the IPC
bridge (`src/ipc/client.ts`). Outside the desktop shell it falls back to
`POST /api/<command>`, so `npm run dev` on its own renders the UI but returns no
data until it is hosted by the desktop binary (below) or a compatible API shim is
mounted.

### Desktop app (Tauri)

The Tauri 2 shell (`apps/desktop/src-tauri`) wires the frontend to the engine
through thin command handlers (`src/commands.rs`) that call the same
`ConfigurationEngine` view methods the CLI uses. It is configured in
`tauri.conf.json` (dev URL `:1420`, build runs the frontend first).

```bash
cd apps/desktop/src-tauri
cargo add tauri-cli   # or: cargo install tauri-cli --locked
tauri dev                                     # hot-reload desktop window
```

> **Status:** the desktop shell is scaffolded and type-checked, but is **not yet
> packaged for release** — `build.rs` and the `icons/` bundle assets are still
> missing, so `tauri build`/bundling is pending. Use the CLI for a fully working
> experience today; the GUI screens are exercised through `tauri dev` during
> development.

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

Repository creation:

```bash
avalanche init ~/my-config --host pc --user alice
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

Phase 0 (Repository Contract) is complete. The contract is defined in `schemas/contract.nix`, mirrored in `avalanche-model`, and declared in managed repositories via `gb.schemaVersion`. It registers every `gb.*` namespace, fixes canonical plural requester naming, and makes ownership, registration, profile, host-override, user-scoping, and external-module boundaries explicit.

Phase 4 (Source Engine) is complete. Avalanche can parse Nix source files, extract assignments with source spans, classify expressions, detect imports, generate source patches, and answer "where is this value actually written?" without modifying the repository.

Phase 5 (Repository Index) is complete. Avalanche builds a derived repository index from static source analysis and Nix evaluation, with fingerprint-based stale detection.

Phase 6 (Provenance) is complete. Definitions are first-class: `avalanche-model::Provenance` retains every definition's source file, span, layer, priority, host, user, profile, condition, and definition type, identifies the winning definition, lists overridden definitions, and projects merged list/attribute values into per-contribution provenance. The index exposes this through `RepositoryIndex::provenance_for`, so Avalanche can explain why an effective value exists.

Phases 7–15 are complete: explicit ownership (derived from provenance, never the nearest filename), the request/dependency graph with `why`/`dependents`/`owners`, Repo Doctor with repair proposals, the read-only CLI, the read-only desktop GUI client, transactional mutation planning and the write engine (stale + ownership + structural gates, rollback on failure), tiered validation (Tier 1 wired into transactions; Tier 2–4 implemented and opt-in), and application discovery/search/enable/disable.

Phase 16 (Module Generation) is complete. `avalanche generate <program|capability|profile> <name>` and the `CreateModule` intent emit templated modules and register them in the correct aggregate in one transaction — a new application needs no manual registration edits.

Phase 17 (Repository Initialization) is complete. `avalanche init <path> --host <h> --user <u>` writes a contract-conforming skeleton (`gb.schemaVersion`, core host/user/request/debug options, aggregates, host/user enumeration files), initializes Git, re-indexes, and validates against the contract — rolling the whole skeleton back if validation fails. Re-running `init` on an existing repository idempotently adds a host or user. The generated repository passes `avalanche doctor` with zero errors.

The next milestone is Phase 18 (Capability Manager) — making capabilities first-class editable objects (list/detail, requesters, implementation, per-host state, new-capability wizard).
