# Capability Manager

**Status: Not yet implemented** (Phase 18)

---

## Designed capabilities

| Feature | Status |
|---|---|
| Capability list (from index) | Not yet |
| Capability detail (identity, description, owner, implementation, requesters, host state) | Not yet |
| Requesters view | Not yet |
| Implementation view | Not yet |
| Per-host state view | Not yet |
| New capability wizard (generate module + request option + aggregate entry + owner declaration) | Not yet |
| Enable/disable capability (add/remove requester) | Implemented (write engine) |

---

## What already exists

The **write engine** already supports `EnableCapability`, `DisableCapability`, `AddRequest`, and `RemoveRequest` intents. The **request graph** already resolves capability edges. The **ownership registry** already tracks capability modules.

What is missing is the **dedicated UI/CLI surface** for browsing and editing capabilities as first-class objects.

---

## Exit criteria (when complete)

Capabilities can be created, enabled, disabled, inspected, and traced.

---

## See also

- [Request Graph](request-graph.md)
- [Ownership](ownership.md)
- [Write Engine](write-engine.md)
- [Module Generation](module-generation.md)
