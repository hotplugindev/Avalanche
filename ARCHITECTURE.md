# Avalanche — Architecture

## 1. Purpose

Avalanche is a configuration engine with a desktop client.

The desktop GUI is not the core system.

The core system is a Rust engine that understands a structured Nix repository and provides:

- semantic inspection
- source provenance
- ownership
- dependency/request graphs
- safe mutation
- validation
- Git integration

The repository remains the source of truth.

---

# 2. Golden Architecture

```text
                         USER
                          │
                          ▼
                  ┌───────────────┐
                  │ Avalanche GUI │
                  └───────┬───────┘
                          │
                    typed API only
                          │
                          ▼
                 ┌──────────────────┐
                 │ Avalanche Core   │
                 └────────┬─────────┘
                          │
       ┌──────────────────┼──────────────────┐
       ▼                  ▼                  ▼
     Index             Ownership           Graph
       │                  │                  │
       └──────────────────┼──────────────────┘
                          ▼
                    Write Planner
                          │
                          ▼
                    Source Engine
                          │
                          ▼
                      Transaction
                          │
                          ▼
                      Validation
                          │
                          ▼
                         Diff
                          │
                          ▼
                        Apply
                          │
                          ▼
                  Nix Repository
                  SOURCE OF TRUTH
```

---

# 3. Core Principles

## 3.1 Repository is the database

Avalanche never becomes authoritative over the Nix repository.

The index can be deleted and rebuilt.

## 3.2 GUI is a client

The frontend submits typed intent.

It does not:

- parse Nix
- determine ownership
- select files
- execute arbitrary Nix mutations
- implement validation rules

## 3.3 Index is derived

The index combines static source analysis and Nix evaluation.

## 3.4 Provenance is first-class

Effective values are insufficient.

The system must retain definitions and their origin.

## 3.5 Ownership is explicit

Owner, requester, and implementer are separate concepts.

## 3.6 Mutations are transactional

No operation should partially modify the repository.

## 3.7 The repository contract is explicit

A managed repository declares conformance to `schemas/contract.nix` via `gb.schemaVersion`. The contract registers every `gb.*` namespace, fixes the canonical requester grammar, and makes ownership, registration, profile, host-override, user-scoping, and external-module semantics machine-readable. Avalanche validates a repository against this contract before indexing; a repository that violates it is rejected, not guessed at.

---

# 4. Workspace

```text
avalanche/
├── apps/
│   └── desktop/
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
├── schemas/
├── fixtures/
├── tests/
└── docs/
```

---

# 5. `avalanche-model`

The domain model.

Core entities:

```text
Repository
Host
User
Profile
Module
Application
Capability
Request
OptionSchema
OptionDefinition
EffectiveValue
SourceLocation
Ownership
Provenance
Layer
Priority
Condition
Transaction
ValidationReport
```

The model should remain independent from Tauri and UI concerns.

---

# 6. `avalanche-core`

Orchestration layer.

Responsibilities:

- expose application-level operations
- coordinate index, write, validation, Git, and Doctor
- enforce high-level invariants
- provide the API consumed by CLI and Tauri

Conceptually:

```rust
pub struct ConfigurationEngine {
    model: ModelService,
    index: IndexService,
    nix: NixService,
    source: SourceService,
    ownership: OwnershipService,
    graph: GraphService,
    writer: WriteService,
    validator: ValidationService,
    git: GitService,
    doctor: DoctorService,
}
```

---

# 7. `avalanche-nix`

Nix semantics and process execution.

Responsibilities:

- Nix process execution
- `nix eval`
- `nix flake check`
- `nix build`
- option extraction
- Nix values
- Nix types
- assertions
- configuration evaluation
- flake discovery
- external input discovery

This layer answers:

> What does the configuration mean?

It does not answer:

> Where should I edit it?

---

# 8. `avalanche-source`

Nix source analysis and editing.

Responsibilities:

- parsing
- AST
- imports
- source spans
- assignments
- expression classification
- AST mutation
- source patch generation
- formatting

This layer answers:

> Where is this configuration written and how can it be safely changed?

---

# 9. `avalanche-index`

Builds the repository index.

Inputs:

```text
Nix repository
      │
      ├── static parser
      │
      └── Nix evaluation
```

Output:

```text
RepositoryIndex
```

The index contains:

```text
options
assignments
requests
ownership
modules
hosts
users
profiles
provenance
```

---

# 10. Static vs Semantic Information

Static analysis can discover:

- file
- line
- source span
- imports
- expression
- registration
- `mkDefault`
- assignment

Nix evaluation can discover:

- option schema
- effective values
- types
- defaults
- descriptions
- examples
- assertions
- conditions

Neither is sufficient alone.

---

# 11. Assignment Classification

Assignments are classified:

```text
DirectLiteral
DirectExpression
GeneratedPattern
Computed
Unknown
```

Write policy:

```text
DirectLiteral       → automatic editing allowed
DirectExpression    → allowed only when understood
GeneratedPattern    → specialized safe editing
Computed            → usually raw/specialized
Unknown             → raw
```

Avalanche must never silently transform unfamiliar expressions.

---

# 12. Provenance Model

An option can have multiple definitions.

```text
OptionDefinition {
    path
    value
    file
    sourceSpan
    priority
    layer
    host
    user
    profile
    condition
    definitionType
}
```

The effective value is derived from definitions.

For merged lists/attribute sets, provenance is retained per contribution where possible.

---

# 13. Ownership Model

Ownership distinguishes:

```text
Owner
Requester
Implementer
```

Example:

```text
Application
    │
    │ requests
    ▼
Capability
    │
    │ implemented by
    ▼
Capability module
    │
    │ owns
    ▼
Upstream options
```

Ownership status:

```text
Owned
Unowned
WrongOwner
Ambiguous
External
```

---

# 14. Request Graph

Graph edges include:

```text
requester → request
request → capability
capability → implementation
implementation → option
```

Queries:

```text
why(node)
dependents(node)
requesters(node)
implementation(node)
owners(option)
```

---

# 15. Write Architecture

The write API is intent-based.

Examples:

```text
setOption
addRequest
removeRequest
enableCapability
disableCapability
setProfileDefault
setHostOverride
resetHostOverride
createModule
createHost
createUser
```

Never expose a generic frontend operation such as:

```text
writeFile(path, contents)
```

to the normal UI.

---

# 16. Write Pipeline

```text
User intent
     │
     ▼
Check repository fingerprint
     │
     ▼
Re-index if stale
     │
     ▼
Resolve ownership
     │
     ▼
Resolve scope/layer
     │
     ▼
Resolve definition
     │
     ▼
Choose edit strategy
     │
     ▼
Create AST/source patch
     │
     ▼
Temporary transaction state
     │
     ▼
Format
     │
     ▼
Validate
     │
     ▼
Generate diff
     │
     ▼
User confirmation
     │
     ▼
Apply
```

---

# 17. Transaction Model

Transactions contain:

```text
id
operations
file changes
validation report
diff
status
```

Possible states:

```text
Planned
Validating
Ready
Applied
Rejected
Failed
RolledBack
```

A transaction must not leave partial configuration behind on failure.

---

# 18. Layer Model

The UI simplifies layering into:

```text
Profile default
Host override
Effective
```

The engine stores a more general definition/priority model.

`mkDefault` is represented as priority information.

Reset semantics:

```text
resetHostOverride
    ↓
delete host definition
    ↓
profile/default definition becomes effective
```

---

# 19. Generic Option System

Pipeline:

```text
Nix option tree
       ↓
OptionSchema
       ↓
RendererRegistry
       ↓
Renderer
       ↓
UI
```

Renderers:

```text
BoolRenderer
StringRenderer
IntRenderer
FloatRenderer
EnumRenderer
PackageRenderer
ListRenderer
AttrsRenderer
AttrsOfRenderer
SubmoduleRenderer
FreeformRenderer
RawRenderer
```

Specialized renderers can override generic ones.

---

# 20. External Modules

External options are tagged:

```text
origin = External
```

Avalanche may configure them through local modules but never edits the dependency itself.

---

# 21. Validation Architecture

Validation is tiered.

```text
Tier 1:
    source/schema/ownership

Tier 2:
    affected nix eval + assertions

Tier 3:
    nix flake check

Tier 4:
    nix build
```

Validation results are attached to transactions.

---

# 22. Git Architecture

Git is independent from write planning.

```text
Write engine
      ↓
working tree
      ↓
Git service
```

The user can make multiple changes before committing.

Git provides:

- diff
- history
- branches
- commits
- pull
- push
- rollback
- conflicts

---

# 23. Repo Doctor

Doctor consumes the repository index and reports:

- schema issues
- registration issues
- ownership problems
- path drift
- orphan modules
- empty profiles
- broken requests
- quality problems

Doctor produces proposed changes rather than silently applying fixes.

---

# 24. CLI and GUI

Both consume the same core engine.

```text
CLI ──────┐
          ├── Avalanche Core
Tauri ────┘
```

No duplicated configuration logic.

---

# 25. Frontend Architecture

Recommended:

```text
React
TypeScript
Tailwind
shadcn/ui
TanStack Query
Zustand
```

Frontend structure:

```text
src/
├── components/
├── features/
├── pages/
├── queries/
├── commands/
├── stores/
└── types/
```

The frontend receives structured data and submits typed commands.

---

# 26. Tauri Boundary

Tauri commands should be thin adapters.

```text
Tauri command
    ↓
parse/validate request
    ↓
Avalanche Core
    ↓
return structured result
```

Business logic should not live inside Tauri command handlers.

---

# 27. Repository Fingerprint

Index freshness is determined using:

```text
git HEAD
working tree hash
flake.lock hash
GB schema version
index schema version
```

A write must verify that its source model is still current.

---

# 28. Testing Architecture

Use real fixture repositories.

Fixtures should cover:

- simple enable flags
- typed options
- function options
- plain options
- capabilities
- mutually exclusive groups
- hardware conditions
- `mkDefault`
- semi-structured strings
- `attrsOf`
- aliased options
- deep module trees
- per-host home overrides
- scripts
- raw host domains
- two-level registration
- option/file path mismatch
- external flakes
- pinned overrides
- multiple users

Write tests should compare:

```text
input repository
+
intent
=
expected diff/final repository
```

---

# 29. Architectural Invariants

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

---

# 30. Architectural Goal

The final system should make this question answerable for every important configuration value:

> **What is this, where did it come from, why is it enabled, who owns it, what overrides it, and what exact Nix change will happen if I modify it?**
