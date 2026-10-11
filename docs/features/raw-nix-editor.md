# Raw Nix Editor

**Status: Not yet implemented** (Phase 25)

---

## Purpose

Raw Nix is the **intentional escape hatch**. When Avalanche cannot safely edit an expression (Computed, Unknown, or beyond the generic renderer's capability), the user can edit the source directly.

---

## Designed features

| Feature | Status |
|---|---|
| Syntax highlighting | Not yet |
| Nix LSP integration | Not yet |
| Formatting (via source engine formatter) | Engine implemented, UI not yet |
| Source location display | Implemented (provenance) |
| Ownership warnings | Implemented (ownership registry gates) |
| Diff preview | Implemented (write engine) |
| Validation | Implemented (Tier 1 structural always runs) |
| Automatic re-index after edit | Not yet |

---

## Safety rules

- Raw changes cause the repository index to be rebuilt.
- Avalanche warns about ownership violations but does **not** silently undo manual edits.
- The write engine's structural validation (parse check) always runs on raw-edited files.
- If the raw edit produces unparseable Nix, the transaction is rolled back.

---

## What already exists

- The source engine formats AST back to Nix (round-trip capable).
- The validation service parses every changed file (Tier 1).
- The write engine applies and rolls back atomically.
- Provenance tracks the source file and span of every definition.

---

## Exit criteria (when complete)

No supported configuration shape is completely inaccessible.

---

## See also

- [Computed Options](computed-options.md)
- [Source Engine](source-engine.md)
- [Validation](validation.md)
