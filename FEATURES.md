# Avalanche — Features

This document defines the intended feature set of Avalanche.

A feature is not complete merely because a UI control exists. A production feature generally requires the domain model, indexing, ownership, source resolution, write operation, validation, GUI integration, CLI support where applicable, and tests.

---

# 1. Dashboard

The dashboard provides the global repository view.

## Repository context

Show:

- repository
- current Git branch
- working-tree state
- selected host
- selected user
- selected profile
- GB schema version
- index freshness

## Health

Show:

- last evaluation
- last `nix flake check`
- last build
- validation status
- Repo Doctor warnings
- pending transactions

## Summary

Show:

- active profiles
- enabled applications
- active capabilities
- host overrides
- recent changes

---

# 2. Hosts

Avalanche supports multiple hosts.

Host context affects:

- system configuration
- hardware facts
- profile activation
- conditional requests
- host overrides
- desktop configuration
- system packages

## Host manager

Supports:

- host list
- host identity
- class
- roles
- hardware facts
- profiles
- system capabilities
- host overrides
- raw host domains

Raw/advanced domains include:

- storage
- firewall
- Wake-on-LAN
- extra groups

## New host wizard

Generates the required host files and registration entries according to the repository contract.

---

# 3. Users

Avalanche supports multiple users.

User context affects:

- Home Manager
- user packages
- user programs
- desktop settings
- scripts
- user-specific overrides

## User manager

Supports:

- list users
- identity fields
- creation
- Home Manager configuration
- user packages
- per-user overrides

Per-user configuration must have an explicit source location.

---

# 4. Software Manager

The software manager is the main application installation interface.

## Search

Search:

- existing Avalanche modules
- package names
- relevant Nix packages
- installed applications

## Install

Installation is configuration, not an imperative package operation.

The install flow asks for:

- system or user scope
- host(s)
- profile default or host override
- existing module or new module

## Existing module

Show:

- typed `gb.*` settings
- upstream options
- requested capabilities
- provenance
- ownership

## New module

Generate:

- module
- package assignment
- options
- capability requests
- category registration
- aggregate registration
- ownership metadata

---

# 5. Application Pages

Each application gets a dedicated configuration page.

Sections:

- Avalanche settings
- upstream Nix options
- capabilities
- provenance
- ownership
- source location
- raw Nix escape hatch

---

# 6. Capability Manager

Capabilities are first-class configuration objects.

A capability contains:

- identity
- description
- owner
- implementation
- requesters
- host state

## Capability list

Show:

- enabled/disabled state
- requester count
- implementation
- ownership
- host activation

## Capability detail

Answer:

- Why is it enabled?
- Who requests it?
- What implements it?
- Which Nix options does it own?

## New capability wizard

Generates:

- request option
- capability implementation
- aggregate entry
- owner declaration
- metadata

---

# 7. Request Graph

Interactive graph showing relationships between:

- applications
- profiles
- hosts
- requesters
- capabilities
- implementations
- Nix options

Queries:

```text
why(X)
dependents(X)
requesters(X)
implementation(X)
owners(X)
```

Example:

```text
workstation profile
        │
        ▼
      desktop
        │
        ▼
     PipeWire
        ▲
        │
    noisetorch
```

---

# 8. Profiles

Profiles define reusable configuration roles.

A profile can contain:

- activation conditions
- requested capabilities
- default programs
- `mkDefault` values
- system defaults
- user defaults

The UI explains that a profile value is a default and can be overridden by a host or user definition.

---

# 9. Multi-Host Layering

Avalanche explicitly exposes:

```text
Profile default
Host override
Effective value
```

Example:

```text
Profile default:  ON
PC override:      ON
Laptop override:  OFF

PC effective:     ON
Laptop effective: OFF
```

"Reset to profile default" deletes the host definition.

It does not write `false`.

Internally, the engine retains the complete definition and priority model.

---

# 10. Hardware-Conditional Requests

Avalanche reads conditions from evaluated Nix configuration.

Example:

```text
hasBattery = true
        ↓
conditional request activates
```

The GUI shows:

- condition
- current value
- evaluation result
- activated request

Avalanche does not duplicate Nix condition logic in the frontend.

---

# 11. Nix Settings

Dedicated settings for:

- flakes
- nix-command
- allowUnfree
- optimizations
- autoclean
- locale
- timezone
- keyboard

Mutually exclusive groups include:

- kernel selection
- boot selection

Invalid combinations are rejected before writing.

---

# 12. Generic Upstream Option Editor

Avalanche recursively extracts upstream NixOS/Home Manager option schemas.

Each option may contain:

- path
- type
- default
- description
- example
- current value
- source

Renderers include:

- boolean
- string
- integer
- float
- enum
- package
- list
- attribute set
- `attrsOf`
- submodule
- freeform
- raw

## Fallback

Function-valued, computed, or unknown options are routed to raw editing unless a specialized renderer exists.

Avalanche never pretends that arbitrary Nix expressions are simple form values.

---

# 13. Specialized Desktop Editors

## MangoWC

Supports:

- monitors
- settings values
- environment
- keybinds
- autostart
- per-host settings

Structured strings are parsed into semantic objects before editing.

## Dank Material Shell

Supports:

- theme
- widget list
- widget ordering
- weather
- system monitoring
- VPN
- control-center features
- aliased GB settings

## Foot

Supports:

- font
- font size
- terminal options
- per-host overrides

## Greeter

Supports:

- tuigreet/DMS selection
- autologin
- greeter settings

---

# 14. Provenance

Every meaningful configuration definition can expose:

- source file
- source span
- priority
- layer
- host
- user
- profile
- condition
- definition type
- effective status

Example:

```text
programs.git.enable

Defined by:
    profiles/workstation.nix

Priority:
    mkDefault

Overridden by:
    hosts/pc/programs.nix

Effective:
    false
```

---

# 15. Ownership

Ownership answers:

> Which capability is responsible for this upstream configuration?

Ownership states:

- Owned
- Unowned
- Wrong owner
- Ambiguous
- External

Ambiguous ownership blocks automatic edits.

---

# 16. Raw Nix Editor

Raw Nix is an intentional escape hatch.

Features:

- syntax highlighting
- Nix LSP
- formatting
- source location
- ownership warnings
- diff
- validation

Raw changes cause the repository index to be rebuilt.

Avalanche warns about ownership violations but does not silently undo manual edits.

---

# 17. Repository Doctor

Repo Doctor checks:

## Schema

- missing schema version
- invalid GB namespaces

## Registration

- missing imports
- missing aggregate entries
- orphan modules

## Ownership

- duplicate ownership
- missing ownership
- wrong ownership
- ambiguous ownership

## Paths

- option path/file path drift
- documented path drift

## Quality

- copy-paste descriptions
- empty profiles
- no-op modules
- suspicious declarations

## State

- untracked configuration files
- broken requests
- missing implementations

Known repository issues should become regression tests.

---

# 18. Repo Doctor Repair

Repairs always follow:

```text
Detect
  ↓
Explain
  ↓
Proposed diff
  ↓
Validate
  ↓
Apply
```

Doctor never silently repairs configuration.

---

# 19. Transactional Writes

Every mutation becomes a transaction.

Operations include:

- set option
- set profile default
- set host override
- reset host override
- enable capability
- disable capability
- add request
- remove request
- create module
- create host
- create user

Pipeline:

```text
User intent
    ↓
Re-read repository fingerprint
    ↓
Re-index if stale
    ↓
Resolve ownership
    ↓
Resolve scope
    ↓
Resolve layer
    ↓
Choose edit strategy
    ↓
AST/source edit
    ↓
Validation
    ↓
Diff
    ↓
Confirmation
    ↓
Apply
```

---

# 20. Safe Source Editing

Avalanche classifies source assignments as:

- DirectLiteral
- DirectExpression
- GeneratedPattern
- Computed
- Unknown

Automatic edits are restricted to safe cases.

Unknown/computed structures use raw editing or specialized editors.

---

# 21. Validation

Validation tiers:

### Tier 1

- AST validity
- schema
- ownership
- registration
- source structure

### Tier 2

- affected `nix eval`
- assertions

### Tier 3

- `nix flake check`

### Tier 4

- expensive `nix build`
- full host validation

---

# 22. Diff Preview

Before applying a transaction, Avalanche shows:

- files changed
- additions
- removals
- exact source diff
- validation state

No hidden file modifications.

---

# 23. Rollback

Failed transactions must not partially modify the repository.

Transactions are applied atomically from the application's perspective.

Git additionally provides:

- restore
- reset
- branch
- rollback

---

# 24. Git

Git integration provides:

- status
- diff
- history
- branch
- commit
- pull
- push
- rollback
- conflict view

The write engine and Git remain separate concerns.

Multiple GUI changes can be reviewed and committed together.

---

# 25. Import / Export / Share

## Host bundle

Export:

- host files
- selected profiles
- selected capabilities
- required metadata

## Profile share

Export a profile plus its transitive capability closure.

## Import

Before modifying the target repository:

- validate schema
- validate namespaces
- validate registration
- validate ownership
- validate dependencies
- show merge preview
- apply as a transaction

---

# 26. External Flakes

Avalanche can inspect external module options from inputs such as:

- Nixvim
- Dank Material Shell
- Dank Greeter
- Mango modules

External source is explicitly marked as external.

Avalanche configures the local layer and never edits external dependencies.

---

# 27. Search

Search understands:

- application names
- package names
- GB option paths
- capabilities
- profiles
- hosts
- users
- upstream Nix options

Results indicate the object type.

---

# 28. CLI

The CLI uses exactly the same Rust engine.

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

---

# 29. Notifications

Notifications cover:

- stale index
- failed evaluation
- failed flake check
- failed build
- external repository changes
- Git conflicts
- ownership ambiguity
- broken repository contract

---

# 30. Safety Rules

Avalanche fails closed for dangerous operations.

```text
Unknown owner       → no automatic write
Unknown expression  → raw editor
External source     → local configuration only
Validation failure  → transaction blocked
Stale index         → re-index before write
Ambiguous scope     → require explicit choice
```

---

# 31. Repository Contract

The repository contract makes a managed Nix repository explicit and predictable. It is defined in `schemas/contract.nix` and declared inside each repository via `gb.schemaVersion`.

## Namespaces

The contract registers every `gb.*` subtree with its scope, kind, and semantics:

- `gb.schemaVersion` — conformance version
- `gb.host.*` — typed host identity and hardware facts
- `gb.user.*` — active user identity
- `gb.requires.system.*` / `gb.requires.home.*` — capability request lists
- `gb.programs.system.*` / `gb.home.programs.*` — typed application settings
- `gb.home.desktop.*` — structured desktop configuration
- `gb.debug.*` — diagnostic switches

## Requester Naming

Requester strings follow a canonical plural grammar:

```text
<kind>.<scope>[.<domain>].<name>
```

Kinds: `programs`, `profiles`, `capabilities`, `hosts`, `users`. Singular forms (`profile.`, `program.`, `host.`) are contract violations rejected by the doctor.

## Ownership

Ownership is an explicit declaration, never inferred from the nearest filename. One upstream option has exactly one owning module. Statuses: owned, unowned, wrongOwner, ambiguous, external.

## Registration

- Capabilities live in `modules/capabilities/<scope>/<domain>/<name>.nix` and activate on a non-empty request list or explicit enable.
- Every module must be imported by an aggregate (`modules/aggregate/nixos.nix` or `home.nix`). No recursive auto-import.

## Profile Semantics

Profiles express intent. They request capabilities and set `mkDefault` values. They never own low-level implementation.

## Host Override Semantics

A host override is a definition at host priority. Reset to profile default deletes the host definition; it never writes `false`.

## User Scoping

Per-user configuration has an explicit source location: identity in `users/<username>.nix`, per-host overrides in `hosts/<host>/home/<username>.nix`. The shared `hosts/<host>/home.nix` is deprecated.

## External Module Boundaries

Options from external flake inputs are tagged `external`. Avalanche configures the local layer and never edits the dependency.

---

# 32. Repository Initialization

`avalanche init` creates a conforming Avalanche-managed Nix repository from scratch.

## What it generates

- `flake.nix` with the standard output structure
- `hosts/` skeleton with at least one host
- `users/` skeleton with at least one user
- `modules/aggregate/nixos.nix` and `home.nix`
- `modules/core/` option declarations (host, user, requests, debug)
- `gb.schemaVersion = 1` declaration
- Git initialization with an initial commit

## Idempotent re-init

Running `avalanche init` on an existing conforming repository adds a new host or user without destroying existing configuration.

## Validation

The generated skeleton is validated against `schemas/contract.nix` before the command exits. A repository that fails validation is rolled back.

## Exit criteria

```bash
avalanche init ~/my-config --host pc --user alice
avalanche doctor  # zero errors
```

---

# 33. Nix Engine

The Nix engine provides reliable communication with the Nix package manager.

## Process Abstraction

All Nix interactions go through a structured process API with:

- configurable binary path
- configurable timeout
- experimental feature flags (`nix-command flakes`)
- structured error reporting

## Evaluation

Supports:

- arbitrary expression evaluation
- flake attribute evaluation
- JSON decoding of results
- Nix type inference

## Option Extraction

Extracts option schemas including:

- type
- default
- description
- example
- internal flag
- readOnly flag

## Assertions

Retrieves and evaluates host assertions, identifying failures before writes.

## Host Evaluation

- lists available NixOS configurations
- evaluates host system information
- evaluates per-host option values

## Home Manager Evaluation

- evaluates Home Manager configurations per user/host
- extracts user-level option values

## Flake Discovery

- discovers flake inputs from lock file
- identifies external module sources
- excludes base dependencies from external module list

## Validation

- `nix flake check` for structural validation
- `nix build` with dry-run support for build validation

---

# 34. Source Engine

The source engine understands Nix source code structure.

## Lexer

Tokenizes Nix source into a stream with:

- identifiers, keywords, numbers, strings, paths
- operators and delimiters
- search paths (`<nixpkgs>`)
- source span tracking for every token

## Parser

Recursive descent parser producing a typed AST:

- attribute sets with bindings
- lists, lambdas, let/in, if/then/else
- binary operators with correct precedence
- function application
- with/assert expressions
- inherit statements
- dynamic attribute paths

## Source Spans

Every AST node carries position information:

- line and column of start
- line and column of end
- enables precise error reporting and targeted edits

## Import Detection

- extracts `import` expressions from parsed files
- resolves relative and absolute paths
- resolves directory imports (default.nix)
- builds import graphs with cycle detection
- classifies import context (top-level, list, let, attr value)

## Assignment Extraction

Walks the AST to extract all assignments with their:

- dotted attribute path
- value expression
- source span
- classification

## Assignment Classification

Assignments are classified as:

- `DirectLiteral` — bool, int, float, string, path, list of literals
- `DirectExpression` — identifiers, binary ops, known function calls (mkDefault, mkForce, import)
- `GeneratedPattern` — recognized generated patterns
- `Computed` — conditionals, lambdas, unknown function applications
- `Unknown` — inherit, unrecognized structures

Write policy:

- DirectLiteral and GeneratedPattern allow automatic editing
- DirectExpression requires understanding
- Computed and Unknown use raw editing only

## Formatting

Pretty-prints AST back to Nix source:

- consistent 2-space indentation
- proper string escaping
- correct operator spacing
- multi-line attr sets and lists

## Source Patch Generation

Generates targeted source modifications:

- value replacement at exact span
- binding removal
- binding insertion with correct indentation
- multi-patch application with reverse-order safety

## Exit Criteria

Avalanche can answer "where is this value actually written?" without modifying the repository.

---

# 35. Future Features

Potential future work:

- remote repositories
- repository profiles
- visual host comparison
- configuration snapshots
- dependency impact analysis
- migration assistant
- schema migrations
- plugin-defined specialized editors
- remote build integration

These should wait until the core model and transaction system are stable.
