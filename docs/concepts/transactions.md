# Transactions

**Status: Implemented** (Phases 12, 13, 14)

Every mutation in Avalanche is a transaction. A transaction is planned, validated,
diffed, and applied atomically. A failed validation must never leave a
half-written repository.

---

## The pipeline

```text
intent
  ↓
resolve (ownership, scope, layer, definition)
  ↓
edit (AST/source patch)
  ↓
format
  ↓
validate
  ↓
diff
  ↓
apply
```

---

## Transaction states

```text
Planned → Validating → Ready → Applied
                              → Rejected
                              → Failed
                              → RolledBack
```

| State | Meaning |
|---|---|
| `Planned` | intents resolved into concrete edits; no files touched |
| `Validating` | structural/eval validation running |
| `Ready` | validation passed; awaiting confirmation |
| `Applied` | changes written to disk |
| `Rejected` | user/ownership rejected the plan |
| `Failed` | an error occurred during apply |
| `RolledBack` | partial writes undone; repository restored |

---

## Mutation intents

```rust
pub enum MutationIntent {
    SetOption { path, value, scope },
    SetProfileDefault { path, value, profile },
    SetHostOverride { path, value, host },
    ResetHostOverride { path, host },
    EnableCapability { capability, requester },
    DisableCapability { capability, requester },
    AddRequest { capability, requester },
    RemoveRequest { capability, requester },
    CreateModule { name, kind },
    CreateHost { name },
    CreateUser { name },
}
```

Each intent is resolved into one or more `ResolvedEdit`s (and optionally
`NewFile` creations) by the core's `resolve_intent`.

---

## Resolved edits

```rust
pub struct ResolvedEdit {
    pub file: String,
    pub option_path: String,
    pub kind: EditKind,          // ReplaceValue | RemoveBinding | InsertBinding
    pub value: Option<serde_json::Value>,
    pub render: RenderHint,      // Literal | MkDefault | MkForce
    pub layer: Layer,
    pub scope: Scope,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub reason: String,
}
```

The edit kind is chosen based on the existing assignment's classification:

- `DirectLiteral` / `GeneratedPattern` → `ReplaceValue` (auto-editable)
- `Computed` / `Unknown` → refused with `Unsupported` (raw editor only)
- no existing definition → `InsertBinding`

---

## Planning

`plan_mutation`:

1. Recomputes the repository fingerprint and rejects if stale (`StaleIndex`).
2. Builds the ownership registry.
3. Resolves every intent into edits/creations.
4. Reads current file contents.
5. Generates source patches (AST-based, never string replacement).
6. Applies patches in-memory to produce `FileChange { before, after }`.
7. Computes a unified diff.
8. Returns a `MutationPlan` with status `Ready`.

No files are written during planning.

---

## Applying

`execute_mutation`:

1. Plans (above).
2. Runs structural validation on the planned output.
3. If validation fails → error, nothing written.
4. Writes files to disk via `DiskFileSource`.
5. Re-parses every written file; if any is unparseable → **rollback** all
   written files and created files, return `ValidationFailed`.
6. Runs post-flight structural validation.
7. Returns an `Applied` transaction with the diff and validation report.

---

## Rollback

On failure during apply, the write engine:

- removes any files it created,
- restores the original contents of any files it modified.

The repository is left exactly as it was found. This is tested in
`generation_rolls_back_created_files_on_parse_failure` and
`generation_plan_failure_writes_nothing`.

---

## Fingerprint / staleness

Before planning, the engine compares the stored index fingerprint against a
freshly computed one:

```text
git HEAD
working tree hash
flake.lock hash
GB schema version
index schema version
```

If any differ, the index is stale and the mutation is rejected with
`StaleIndex`. The caller must re-index before retrying.

---

## See also

- [Write Engine](../features/write-engine.md)
- [Validation](../features/validation.md)
- [Safety](safety.md)
