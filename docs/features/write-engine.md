# Write Engine

**Status: Partial** (Phases 12, 13 - core mutations implemented, broader editing pending)

The write engine implements intent-based mutation with full transaction semantics. It lives in `avalanche-write` (planning/apply/rollback) and `avalanche-core` (intent resolution).

---

## Implemented operations

| Intent | Status | Description |
|---|---|---|
| `SetOption` | Implemented | Set an upstream option (ownership-gated) |
| `SetProfileDefault` | Implemented | Write a `lib.mkDefault` value in a profile module |
| `SetHostOverride` | Implemented | Write a host-priority override |
| `ResetHostOverride` | Implemented | Delete the host definition (never writes `false`) |
| `EnableCapability` | Implemented | Add requester to a capability's request list |
| `DisableCapability` | Implemented | Remove requester from a capability's request list |
| `AddRequest` | Implemented | Add a requester string to `gb.requires.*` |
| `RemoveRequest` | Implemented | Remove a requester string from `gb.requires.*` |
| `CreateModule` | Implemented | Generate a new program/capability/profile module |
| `CreateHost` | Implemented | Scaffold a new host directory + registration |
| `CreateUser` | Implemented | Scaffold a new user file + registration |

---

## Pipeline

```text
intent
  → resolve (ownership gate, scope, layer, definition, edit kind)
  → edit (AST/source patch)
  → format
  → validate (structural pre-flight)
  → diff
  → apply (atomic write)
  → validate (structural post-flight + optional eval/flake/build)
  → rollback on failure
```

---

## Resolution

`resolve_intent` in `avalanche-core` maps each `MutationIntent` to one or more `ResolvedEdit`s:

1. **Staleness gate**: recompute fingerprint, reject if stale.
2. **Ownership gate**: for upstream options, require `Owned` status.
3. **Scope/layer resolution**: determine target file from provenance + intent parameters.
4. **Edit kind selection**:
   - Existing `DirectLiteral`/`GeneratedPattern` → `ReplaceValue`
   - Existing `Computed`/`Unknown` → refuse (`Unsupported`)
   - No existing definition → `InsertBinding`
5. **Render hint**: `Literal`, `MkDefault`, or `MkForce` (controls how the value is written in Nix syntax).

---

## Planning

`WriteService::plan_edits`:

1. Read current file contents.
2. Generate source patches (AST-based, never string replacement).
3. Apply patches in-memory.
4. Produce `FileChange { path, before, after }`.
5. Compute unified diff.
6. Return `MutationPlan` with status `Ready`.

---

## Applying

`WriteService::apply_plan`:

1. Write all modified files.
2. Create all new files.
3. Re-parse every written file; if unparseable → **rollback** (restore originals, remove created files).
4. Return `Transaction` with status `Applied`.

---

## Rollback

On any failure during apply:

- Created files are removed.
- Modified files are restored to their `before` content.
- The repository is left exactly as found.

Tested in `generation_rolls_back_created_files_on_parse_failure` and `module_creation_without_aggregate_fails_closed_and_leaves_repo_unchanged`.

---

## Validation integration

`execute_mutation_with_tier` runs validation at the requested tier:

| Tier | What runs |
|---|---|
| Structural (default) | Parse all changed files before and after |
| NixEval | + `nix eval` affected hosts + assertions |
| FlakeCheck | + `nix flake check` |
| Build | + `nix build --dry-run` |

If structural validation fails pre-flight, nothing is written. If post-flight fails, rollback occurs.

---

## What is NOT yet implemented

- Generic option editing UI (typed forms for bool/string/int/list/attrs/submodule/freeform)
- Computed/unknown expression editors (raw fallback)
- Specialized desktop editors (Foot, Greeter, DMS, MangoWC)
- Multi-file atomic transactions (currently single-intent-per-plan)
- Conflict detection with concurrent external edits

---

## See also

- [Concepts: Transactions](../concepts/transactions.md)
- [Source Engine](source-engine.md)
- [Validation](validation.md)
- [Ownership](ownership.md)
