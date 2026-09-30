# Avalanche — Roadmap

This roadmap defines the recommended implementation order for Avalanche.

The ordering is intentional. Later phases depend on earlier architectural foundations.

---

# Phase 0 — Repository Contract

## Goal

Make the managed Nix repository explicit and predictable.

## Tasks

- [x] Add `gb.schemaVersion`
- [x] Document `gb.*` namespaces
- [x] Define ownership conventions
- [x] Define requester naming
- [x] Define capability registration
- [x] Define aggregate registration
- [x] Define profile semantics
- [x] Define host override semantics
- [x] Define user scoping
- [x] Define external module boundaries
- [x] Refactor multi-user Home Manager layout
- [x] Resolve known path/description inconsistencies

## Exit criteria

A new application, capability, profile, host, and user can each be described unambiguously.

---

# Phase 1 — Rust Workspace

## Goal

Establish the project structure.

## Tasks

- [x] Create Cargo workspace
- [x] Create all core crates
- [x] Create Tauri desktop shell
- [x] Create frontend application
- [x] Establish development flake
- [x] Establish CI
- [x] Establish formatting/linting/test commands

## Exit criteria

```bash
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
```

work.

---

# Phase 2 — Domain Model

## Goal

Implement the semantic model.

## Tasks

- [x] Repository
- [x] Host
- [x] User
- [x] Profile
- [x] Module
- [x] Application
- [x] Capability
- [x] Request
- [x] OptionSchema
- [x] OptionDefinition
- [x] Provenance
- [x] Ownership
- [x] Layer
- [x] Priority
- [x] Condition
- [x] Transaction
- [x] ValidationReport

## Exit criteria

The domain model can represent every configuration shape identified by the repository audit.

---

# Phase 3 — Nix Engine

## Goal

Reliably communicate with Nix.

## Tasks

- [x] Nix process abstraction
- [x] `nix eval`
- [x] `nix flake check`
- [x] `nix build`
- [x] JSON decoding
- [x] Nix type representation
- [x] option extraction
- [x] assertions
- [x] host evaluation
- [x] Home Manager evaluation
- [x] external flake discovery

## Exit criteria

Avalanche can obtain structured semantic information from every managed host.

---

# Phase 4 — Source Engine

## Goal

Understand Nix source code.

## Tasks

- [x] Parser integration
- [x] AST model
- [x] imports
- [x] assignments
- [x] source spans
- [x] expressions
- [x] assignment classification
- [x] formatting
- [x] source patch generation

## Exit criteria

Avalanche can answer:

> Where is this value actually written?

without modifying the repository.

---

# Phase 5 — Repository Index

## Goal

Build the complete derived repository model.

## Tasks

- [x] Static index
- [x] Semantic index
- [x] module index
- [x] option index
- [x] assignment index
- [x] request index
- [x] host index
- [x] user index
- [x] profile index
- [x] source location index
- [x] repository fingerprint
- [x] stale detection

## Exit criteria

The index can answer:

- what exists
- where it is
- what it means
- current value
- source location

---

# Phase 6 — Provenance

## Goal

Make definitions first-class.

## Tasks

- [ ] source file
- [ ] source span
- [ ] layer
- [ ] priority
- [ ] host
- [ ] user
- [ ] profile
- [ ] condition
- [ ] definition type
- [ ] merged-value contribution provenance

## Exit criteria

Avalanche can explain why an effective value exists.

---

# Phase 7 — Ownership

## Goal

Implement explicit ownership.

## Tasks

- [ ] ownership registry
- [ ] owner lookup
- [ ] requester lookup
- [ ] implementation lookup
- [ ] duplicate ownership detection
- [ ] ambiguous ownership
- [ ] external ownership
- [ ] ownership validation

## Exit criteria

Avalanche can answer:

> Who owns this upstream setting?

---

# Phase 8 — Request Graph

## Goal

Implement dependency attribution.

## Tasks

- [ ] graph model
- [ ] capability edges
- [ ] requester edges
- [ ] implementation edges
- [ ] option edges
- [ ] `why`
- [ ] `dependents`
- [ ] `requesters`
- [ ] `implementation`

## Exit criteria

```bash
avalanche why audio.pipewire
```

works reliably.

---

# Phase 9 — Repo Doctor

## Goal

Detect repository inconsistencies.

## Tasks

- [ ] schema checks
- [ ] import checks
- [ ] aggregate checks
- [ ] orphan modules
- [ ] duplicate ownership
- [ ] path/file drift
- [ ] copy-paste descriptions
- [ ] empty profiles
- [ ] broken requests
- [ ] untracked configuration
- [ ] repair proposal system
- [ ] regression fixtures

## Exit criteria

Known repository problems are automatically detected and explained.

---

# Phase 10 — Read-Only CLI

## Goal

Make the engine independently useful.

## Commands

```text
avalanche inspect
avalanche why
avalanche dependents
avalanche owners
avalanche doctor
avalanche diff
```

## Exit criteria

The CLI is useful without a GUI.

---

# Phase 11 — Read-Only GUI

## Goal

Build the first desktop client.

## Screens

- Dashboard
- Hosts
- Users
- Profiles
- Applications
- Capabilities
- Graph
- Provenance
- Repo Doctor

## Exit criteria

A user can explore the entire repository without changing it.

---

# Phase 12 — Mutation Planning

## Goal

Represent changes before applying them.

## Tasks

- [ ] MutationIntent
- [ ] MutationPlan
- [ ] FileChange
- [ ] Transaction
- [ ] diff generation
- [ ] repository fingerprint checks
- [ ] stale index rejection

## Exit criteria

Avalanche can show exactly what it intends to change without applying it.

---

# Phase 13 — Transactional Write Engine

## Goal

Implement safe configuration mutation.

## First operations

- [ ] set boolean
- [ ] set string
- [ ] set integer
- [ ] set simple list
- [ ] set profile default
- [ ] set host override
- [ ] reset host override

## Pipeline

```text
intent
→ resolve
→ edit
→ format
→ evaluate
→ validate
→ diff
→ apply
```

## Exit criteria

A failed mutation leaves the repository unchanged.

---

# Phase 14 — Validation

## Goal

Integrate validation into transactions.

## Tasks

- [ ] structural validation
- [ ] affected `nix eval`
- [ ] assertions
- [ ] `nix flake check`
- [ ] optional full build
- [ ] validation UI

## Exit criteria

Every transaction has an explicit validation state.

---

# Phase 15 — Application Manager

## Goal

Manage existing applications.

## Tasks

- [ ] application search
- [ ] install
- [ ] enable
- [ ] disable
- [ ] package selection
- [ ] typed GB options
- [ ] capability requests
- [ ] profile defaults
- [ ] host overrides
- [ ] provenance

## Exit criteria

Simple applications can be completely managed through Avalanche.

---

# Phase 16 — Module Generation

## Goal

Create new application modules safely.

## Tasks

- [ ] module templates
- [ ] package placement
- [ ] option declarations
- [ ] capability requests
- [ ] category registration
- [ ] aggregate registration
- [ ] ownership
- [ ] re-index
- [ ] validation

## Exit criteria

A new application can be generated without manual registration edits.

---

# Phase 17 — Repository Initialization

## Goal

Create a conforming Avalanche-managed Nix repository from scratch.

## Tasks

- [ ] `avalanche init` CLI command
- [ ] repository skeleton templates (flake.nix, hosts/, users/, modules/)
- [ ] contract bootstrap (gb.schemaVersion declaration)
- [ ] aggregate registration scaffolding
- [ ] core option modules (host, user, requests, debug)
- [ ] initial host creation (interactive or flag-driven)
- [ ] initial user creation
- [ ] git initialization
- [ ] validation of generated skeleton against contract
- [ ] idempotent re-init (add host/user to existing repo)

## Exit criteria

```bash
avalanche init ~/my-config --host pc --user alice
```

produces a repository that passes `avalanche doctor` with zero errors and can be immediately indexed.

---

# Phase 18 — Capability Manager

## Goal

Make capabilities first-class editable objects.

## Tasks

- [ ] capability list
- [ ] capability detail
- [ ] requesters
- [ ] implementation
- [ ] per-host state
- [ ] new capability wizard

## Exit criteria

Capabilities can be created, enabled, disabled, inspected, and traced.

---

# Phase 19 — Profiles

## Goal

Manage reusable roles.

## Tasks

- [ ] profile list
- [ ] activation conditions
- [ ] requested capabilities
- [ ] default programs
- [ ] `mkDefault`
- [ ] profile editor
- [ ] host override editor
- [ ] reset semantics

## Exit criteria

Profile/default/override/effective semantics are completely supported.

---

# Phase 20 — Hosts

## Goal

Complete host management.

## Tasks

- [ ] host wizard
- [ ] host facts
- [ ] roles
- [ ] hardware facts
- [ ] profiles
- [ ] system capabilities
- [ ] host overrides
- [ ] raw host domains

## Exit criteria

A host can be created and configured end-to-end.

---

# Phase 21 — Users

## Goal

Complete multi-user management.

## Tasks

- [ ] user creation
- [ ] identity
- [ ] Home Manager
- [ ] user packages
- [ ] user programs
- [ ] user overrides
- [ ] per-host user configuration

## Exit criteria

Multiple users can be independently managed.

---

# Phase 22 — Generic Option Editor

## Goal

Edit upstream Nix module options.

## Renderer order

1. [ ] bool
2. [ ] string
3. [ ] int
4. [ ] float
5. [ ] enum
6. [ ] package
7. [ ] list
8. [ ] attrs
9. [ ] attrsOf
10. [ ] submodule
11. [ ] freeform
12. [ ] raw fallback

## Exit criteria

Most ordinary NixOS/Home Manager module options can be edited without writing Nix manually.

---

# Phase 23 — Computed Options

## Goal

Handle difficult Nix expressions safely.

## Tasks

- [ ] function detection
- [ ] computed detection
- [ ] unknown detection
- [ ] raw fallback
- [ ] specialized editor API

## Exit criteria

Avalanche never corrupts computed options by treating them as simple values.

---

# Phase 24 — Specialized Desktop Editors

## Order

1. [ ] Foot
2. [ ] Greeter
3. [ ] Dank Material Shell
4. [ ] MangoWC

## Exit criteria

Complex desktop configuration can be managed semantically rather than as raw strings.

---

# Phase 25 — Raw Nix Editor

## Goal

Provide the universal escape hatch.

## Tasks

- [ ] syntax highlighting
- [ ] Nix LSP
- [ ] formatting
- [ ] source location
- [ ] ownership warnings
- [ ] diff
- [ ] validation
- [ ] automatic re-index

## Exit criteria

No supported configuration shape is completely inaccessible.

---

# Phase 26 — Git

## Goal

Integrate repository lifecycle management.

## Tasks

- [ ] status
- [ ] diff
- [ ] history
- [ ] branch
- [ ] commit
- [ ] pull
- [ ] push
- [ ] rollback
- [ ] conflict view

## Exit criteria

Users can review and manage Avalanche changes through Git.

---

# Phase 27 — Import / Export / Share

## Goal

Make configuration portable.

## Tasks

- [ ] host export
- [ ] profile export
- [ ] capability closure
- [ ] import validation
- [ ] merge preview
- [ ] transactional import
- [ ] schema compatibility

## Exit criteria

Configuration can be shared without bypassing repository invariants.

---

# Phase 28 — Hardening

## Goal

Prepare for real-world use.

## Tasks

- [ ] external file modifications
- [ ] stale index recovery
- [ ] Git conflicts
- [ ] interrupted transactions
- [ ] failed Nix evaluation
- [ ] failed builds
- [ ] large repository performance
- [ ] cache invalidation
- [ ] error messages
- [ ] accessibility
- [ ] security review

---

# Phase 29 — Release

## Release checklist

- [ ] documentation complete
- [ ] fixture coverage complete
- [ ] core invariants tested
- [ ] transaction safety tested
- [ ] Git behavior tested
- [ ] import/export tested
- [ ] multi-host tested
- [ ] multi-user tested
- [ ] external flake tested
- [ ] unknown/computed options tested
- [ ] clean installation tested
- [ ] upgrade path documented

---

# Milestones

| Milestone | Result |
|---|---|
| M0 | Repository contract |
| M1 | Rust workspace |
| M2 | Domain + Nix/source engines |
| M3 | Complete repository index |
| M4 | Provenance + ownership + graph |
| M5 | Repo Doctor + read-only CLI |
| M6 | Read-only desktop |
| M7 | Transaction/write engine |
| M8 | Applications + modules + repository init |
| M9 | Capabilities + profiles |
| M10 | Hosts + users |
| M11 | Generic option editor |
| M12 | Specialized editors |
| M13 | Raw editor + Git |
| M14 | Import/export/share |
| M15 | Production hardening |

---

# Critical Gates

## Gate 1 — Model

Before serious UI work:

```text
inspect
why
owners
doctor
```

must work.

## Gate 2 — Safe Writes

Before broad configuration editing:

```text
ownership
provenance
transactions
AST editing
rollback
validation
```

must work.

## Gate 3 — Release

The application must survive:

- manual Nix edits
- Git changes
- external flake changes
- stale index
- failed evaluation
- failed build
- merge conflicts
- unknown expressions
- multiple hosts
- multiple users

without corrupting the repository.

---

# Long-Term Goal

Avalanche should make every important configuration question answerable:

> What is this?

> Where is it defined?

> Why is it enabled?

> Who requested it?

> Who owns it?

> What overrides it?

> What will change if I edit it?

> Will the resulting configuration validate?

That is the standard the project should be built around.
