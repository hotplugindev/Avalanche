# Application Manager

**Status: Partial** (Phase 15 - search and enable/disable implemented; full editing pending)

---

## Implemented

| Capability | Status |
|---|---|
| Application search (name, category, scope, module path, requester, option path) | Implemented |
| Enable application (host override) | Implemented |
| Disable application (reset host override) | Implemented |
| Application listing (from index) | Implemented |
| Typed `gb.*` option display | Implemented (read-only) |
| Capability request display | Implemented (read-only) |
| Provenance display | Implemented (read-only) |
| Ownership display | Implemented (read-only) |

---

## Not yet implemented

| Capability | Status |
|---|---|
| Install (create new module) | Pending (module generation exists but not wired to install flow) |
| Package selection | Not yet |
| Typed GB option editing (forms) | Not yet |
| Capability request editing | Not yet |
| Profile default setting | Not yet |
| Host override setting | Not yet |
| Full application detail page with edit controls | Not yet |

---

## Architecture

The application manager is a **view layer** over the engine. It does not implement configuration semantics. All mutation goes through `MutationIntent` → write engine.

```text
UI intent
  ↓
Avalanche command (SetHostOverride / ResetHostOverride)
  ↓
write service
  ↓
transaction
  ↓
validation
  ↓
diff
  ↓
apply
```

---

## CLI

```bash
avalanche inspect app firefox
avalanche search spotify
```

---

## Exit criteria (when complete)

Simple applications can be completely managed through Avalanche.

---

## See also

- [Write Engine](write-engine.md)
- [Provenance](provenance.md)
- [Ownership](ownership.md)
