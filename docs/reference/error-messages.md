# Error Messages

Avalanche errors preserve context. Every error explains: what failed, why, what was not changed, and what the user can do next.

---

## Error taxonomy

### `WriteError` (write engine)

| Variant | When | Message pattern |
|---|---|---|
| `OwnershipDenied(reason)` | Option not owned by a single module | "ownership check failed: {reason}" |
| `StaleIndex` | Repository changed after indexing | "stale index: repository changed after indexing; nothing was modified" |
| `ValidationFailed(reason)` | Structural/eval validation failed | "validation failed: {reason}; repository was not modified" |
| `EditFailed(reason)` | Could not resolve target, invalid name, missing definition | "edit failed: {reason}" |
| `Io { path, reason }` | File read/write error | "io error while reading {path}: {reason}" |
| `UnsupportedValue { option_path, reason }` | Value cannot be represented safely | "unsupported value for {option_path}: {reason}; refusing to silently transform" |
| `Unsupported(reason)` | Computed/Unknown expression | "unsupported operation: {reason}; repository was not modified" |

### `NixError` (Nix engine)

| Variant | When |
|---|---|
| `NixNotFound` | Nix binary not available |
| `ProcessFailed { args, stderr }` | Nix process exited non-zero |
| `JsonDecodeFailed { reason }` | Could not parse Nix JSON output |
| `OptionExtractionFailed { path, reason }` | Option eval failed |
| `HostEvalFailed { host, reason }` | Host eval failed |
| `HomeEvalFailed { host, user, reason }` | Home Manager eval failed |
| `Timeout { args }` | Process exceeded timeout |

### `IndexError` (index)

| Variant | When |
|---|---|
| `RepoNotFound { path }` | Repository directory does not exist |
| `WalkFailed { path, reason }` | Directory traversal error |
| `EvalFailed { reason }` | Semantic index build error |

### `ParseError` (source engine)

| Variant | When |
|---|---|
| `UnexpectedToken { found, expected, line, col }` | Parser encountered wrong token |
| `UnexpectedEof { expected }` | Input ended prematurely |
| `Lex(reason)` | Tokenizer error |

### `GitError` (git engine)

| Variant | When |
|---|---|
| `CommandFailed { command, stderr }` | Git command exited non-zero |
| `NotARepo { path }` | Path is not a git repository |
| `Io(reason)` | I/O error |
| `Parse(reason)` | Could not parse git output |

### `ModelError` (domain model)

| Variant | When |
|---|---|
| `Invalid(reason)` | Contract violation (bad requester, unknown kind, etc.) |

---

## Good vs bad error messages

Bad:

```text
failed
```

Good:

```text
Failed to update programs.git.enable:
the current definition in hosts/pc/programs.nix
changed after the index was created.
Repository was not modified.
```

---

## Error context requirements

Every error should explain:

1. **What failed** - the operation and target
2. **Why** - the specific reason
3. **What was not changed** - reassurance that the repository is intact
4. **What the user can do next** - actionable guidance

---

## Exit codes (CLI)

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Runtime error |
| 2 | Usage error (bad flags, missing args) |

---

## See also

- [Concepts: Safety](../concepts/safety.md)
- [Write Engine](../features/write-engine.md)
- [CLI](../features/cli.md)
