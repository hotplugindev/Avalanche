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

- [x] source file
- [x] source span
- [x] layer
- [x] priority
- [x] host
- [x] user
- [x] profile
- [x] condition
- [x] definition type
- [x] merged-value contribution provenance

## Exit criteria

Avalanche can explain why an effective value exists.

---

# Phase 7 — Ownership

## Goal

Implement explicit ownership.

## Tasks

- [x] ownership registry
- [x] owner lookup
- [x] requester lookup
- [x] implementation lookup
- [x] duplicate ownership detection
- [x] ambiguous ownership
- [x] external ownership
- [x] ownership validation

## Exit criteria

Avalanche can answer:

> Who owns this upstream setting?

Ownership is derived from the provenance index (which capability/program module
actually writes the option), never from the nearest filename. A single registered
capability/program owner yields `Owned`; two or more yield `Ambiguous`; a
capability/program-layer value written from a file that is not the registered owner
yields `WrongOwner`; definitions sourced entirely from external modules yield
`External`; everything else yields `Unowned`. Only `Owned` permits automatic edits.

---

# Phase 8 — Request Graph

## Goal

Implement dependency attribution.

## Tasks

- [x] graph model
- [x] capability edges
- [x] requester edges
- [x] implementation edges
- [x] option edges
- [x] `why`
- [x] `dependents`
- [x] `requesters`
- [x] `implementation`

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

- [x] schema checks
- [x] import checks
- [x] aggregate checks
- [x] orphan modules
- [x] duplicate ownership
- [x] path/file drift
- [x] copy-paste descriptions
- [x] empty profiles
- [x] broken requests
- [x] untracked configuration
- [x] repair proposal system
- [x] regression fixtures

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

 without a GUI.

## Status

Implemented. `avalanche-cli` is a thin adapter over `ConfigurationEngine` with
six read-only subcommands (`inspect`, `why`, `dependents`, `owners`, `doctor`,
`diff`) and global flags `-r/--repo`, `--json`, `--nix`, `-h/--help`. The default
index is static (no Nix evaluation required); `--nix` opts into the full semantic
index. `doctor` exits non-zero when errors are present. All semantics live in
core/index/ownership/graph/doctor — the CLI never parses Nix, infers ownership,
or mutates Git.

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

- [x] MutationIntent
- [x] MutationPlan
- [x] FileChange
- [x] Transaction
- [x] diff generation
- [x] repository fingerprint checks
- [x] stale index rejection

## Exit criteria

Avalanche can show exactly what it intends to change without applying it.

Implemented. `ConfigurationEngine::plan_mutation` resolves typed
`MutationIntent`s into a `MutationPlan` (a list of `ResolvedEdit`s plus
`FileChange`s and a unified diff) without touching the repository. Planning
enforces the safety gates before producing any diff: the index fingerprint is
compared against a freshly computed one and a stale index is rejected (§13);
upstream options are gated by ownership so only `Owned` options are auto-edited
(§8); existing assignments that classify as computed/unknown are refused rather
than silently transformed (§7); `mkDefault` is preserved as a Nix priority via
the render hint rather than a boolean (§10); and "reset to profile default"
deletes the host override instead of writing `false` (§11). Capability, request
and create intents return an explicit `Unsupported` error so the caller knows
nothing changed (§22). Actual application of a plan remains in Phase 13.

---

# Phase 13 — Transactional Write Engine

## Goal

Implement safe configuration mutation.

## First operations

- [x] set boolean
- [x] set string
- [x] set integer
- [x] set simple list
- [x] set profile default
- [x] set host override
- [x] reset host override

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

Implemented. `WriteService::apply_plan` writes all planned
`FileChange`s through a `FileSource`, then reads each file back and
parses it (Tier 1 structural check). On any IO error or parse failure
the already-written files are rolled back to their `before` content
in reverse order, so a failed mutation leaves the repository unchanged.
`ConfigurationEngine::execute_mutation` composes `plan_mutation`
(staleness + ownership + resolution) with `apply_plan` (write + validate
+ rollback) and returns an `Applied` `Transaction` carrying the diff
and validation report. `nix eval` (Tier 2) is deferred to Phase 14.

---

# Phase 14 — Validation

## Goal

Integrate validation into transactions.

## Tasks

- [x] structural validation
- [ ] affected `nix eval`
- [ ] assertions
- [ ] `nix flake check`
- [ ] optional full build
- [ ] validation UI

## Exit criteria

Every transaction has an explicit validation state.

Implemented (Tier 1). `ValidationService::validate_structural` parses every
planned file and reports AST errors as `ValidationFinding`s. `execute_mutation`
runs a pre-flight check on the in-memory `after` content before any write
(a bad plan never touches disk, §14) and a post-write check on the content
actually read back from the repository (confirms the write landed and is
parseable). On post-write failure `apply_plan` restores every already-written
file to its pre-write content in reverse order before returning the error.
Every successful transaction carries
`validation: Some(ValidationReport { tier: Structural, .. })`. Tier 2–4
(`nix eval`, assertions, `flake check`, build) are implemented in
`ValidationService` but not yet wired into the default transaction path;
they require a real Nix binary and are gated behind explicit opt-in to
avoid slow tests (§15).

---

# Phase 15 — Application Manager

## Goal

Manage existing applications.

## Tasks

- [x] application search
- [ ] install
- [x] enable
- [x] disable
- [ ] package selection
- [x] typed GB options
- [ ] capability requests
- [ ] profile defaults
- [x] host overrides
- [x] provenance

## Exit criteria

Simple applications can be completely managed through Avalanche.

Implemented (discovery + search + enable/disable slice). Application
discovery (`build_application_index`) is driven by the declared
`options.gb.{programs.system,home.programs}.<...>.enable` assignments in the
static index — never inferred from file paths — so only options inside the two
contract `NamespaceKind::Application` namespaces become applications, and
out-of-namespace modules (e.g. `gb.home.dev.editors.zed`) are correctly
excluded. `ApplicationId::option_path` round-trips both the flat system shape
(`gb.programs.system.steam`) and the namespaced home shape
(`gb.home.programs.web.firefox`). `ConfigurationEngine::search_applications`
performs a case-insensitive substring match over name, category, requester
name, GB option path and module path, returning typed `Application` results.
Enable/disable reuse the Phase 13 transactional write engine: enabling writes a
host override (`<app>.enable = true`) via `SetHostOverride`; disabling deletes
that override via `ResetHostOverride` (delete, never write `false`, §11). All
edits preserve layer/scope/host provenance and pass the Tier 1 structural
gate. Deferred to keep the slice deterministic and nix-free (§15/§18): install
and package selection (require a live `nix` binary), capability-request wiring
(`EnableCapability`/`DisableCapability` remain `Unsupported` in the planner),
and a dedicated profile-default helper for apps (reachable today through the
generic `SetProfileDefault` intent but not yet exposed at the application
level).

---

# Phase 16 — Module Generation

## Goal

Create new application modules safely.

## Tasks

- [x] module templates
- [x] package placement
- [x] option declarations
- [x] capability requests
- [x] category registration
- [x] aggregate registration
- [x] ownership
- [x] re-index
- [x] validation

## Exit criteria

A new application can be generated without manual registration edits.

Implemented. `avalanche generate <program|capability|profile> <name>` and the
`CreateModule` intent build a module from `avalanche-write::templates`
(`program_module`/`capability_module`/`profile_module`, with `mkEnableOption`
declarations and the correct `gb.*` option prefix), place it at the contract path
(`modules/{programs,capabilities,profiles}/<scope>/…`), and append the matching
`../…` import to the scope aggregate (`modules/aggregate/nixos.nix` or
`home.nix`) in the same transaction. Home programs in a category also get a
category `default.nix` aggregation. Names are validated (`validate_identifier`/
`validate_dotted`) so path traversal and malformed identifiers are rejected before
any write. The plan flows through the Phase 13 pipeline (stale gate, structural
validation, rollback on failure), and the next CLI invocation re-indexes the new
module automatically.

---

# Phase 17 — Repository Initialization

## Goal

Create a conforming Avalanche-managed Nix repository from scratch.

## Tasks

- [x] `avalanche init` CLI command
- [x] repository skeleton templates (flake.nix, hosts/, users/, modules/)
- [x] contract bootstrap (gb.schemaVersion declaration)
- [x] aggregate registration scaffolding
- [x] core option modules (host, user, requests, debug)
- [x] initial host creation (interactive or flag-driven)
- [x] initial user creation
- [x] git initialization
- [x] validation of generated skeleton against contract
- [x] idempotent re-init (add host/user to existing repo)

## Exit criteria

```bash
avalanche init ~/my-config --host pc --user alice
```

produces a repository that passes `avalanche doctor` with zero errors and can be immediately indexed.

Implemented. `ConfigurationEngine::init_repository` validates the spec, writes the
skeleton from `templates::init_skeleton` (`flake.nix`, `hosts/default.nix` +
`hosts/<host>/default.nix`, `users/default.nix` + `users/<user>.nix`,
`modules/core/schema.nix` declaring `gb.schemaVersion = 1`, the core host/user/
request/debug option modules, and both aggregates), then runs `git init` with an
initial commit, re-indexes statically, and diagnoses against the contract. If the
skeleton reports any error the whole generation (including `.git`) is removed and
`init` fails closed. Re-running `init` where `flake.nix` already exists takes the
idempotent `add_to_existing` path and appends a new host or user without touching
existing configuration. Covered by `crates/avalanche-core/tests/repository_init.rs`
(valid repo + doctor-zero-errors, idempotent second host, bad-name rejection,
schema version in fingerprint).

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
5. [ ] Niri
6. [ ] Hyprland
7. [ ] Noctalia

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
