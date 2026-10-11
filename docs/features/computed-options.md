# Computed Options

**Status: Partial** (Phase 23 - classification implemented, editing pending)

---

## Implemented

| Capability | Status |
|---|---|
| Function detection (lambda, unknown apply) | Implemented (source engine classifier) |
| Computed detection (if/then/else, let, with, assert) | Implemented |
| Unknown detection (inherit, unrecognized) | Implemented |
| Write policy enforcement (Computed/Unknown → refuse auto-edit) | Implemented (write engine: `Unsupported` error) |
| Raw fallback routing | Designed, not yet surfaced in UI |

---

## Not yet implemented

| Capability | Status |
|---|---|
| Specialized editor API | Not yet |
| Raw Nix editor UI | Not yet |
| Function-value inspection (show the lambda, allow parameter editing) | Not yet |

---

## Classification rules

```text
Expr::If { .. }       → Computed
Expr::Lambda { .. }   → Computed
Expr::Let { .. }      → Computed
Expr::With { .. }     → Computed
Expr::Assert { .. }   → Computed
Expr::Apply { func: unknown } → Computed
Expr::Inherit { .. }  → Unknown
Expr::Attrs { non-literal values } → Computed
```

---

## Write policy

```text
DirectLiteral       → automatic editing allowed
DirectExpression    → allowed only when understood
GeneratedPattern    → specialized safe editing
Computed            → usually raw/specialized
Unknown             → raw
```

The write engine calls `edit_kind_for` which checks `is_auto_editable()`. Non-editable assignments produce:

```text
WriteError::Unsupported("current definition of 'X' in file is Computed; refusing to silently transform")
```

---

## Exit criteria (when complete)

Avalanche never corrupts computed options by treating them as simple values.

---

## See also

- [Source Engine](source-engine.md)
- [Write Engine](write-engine.md)
- [Raw Nix Editor](raw-nix-editor.md)
