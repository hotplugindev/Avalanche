# Hosts

**Status: Not yet implemented** (Phase 20)

---

## Designed capabilities

| Feature | Status |
|---|---|
| Host wizard (create) | Implemented (write engine: `CreateHost`, CLI: `host create`) |
| Host facts view | Implemented (read-only: `inspect host`) |
| Roles view | Implemented (read-only) |
| Hardware facts view | Implemented (read-only) |
| Profiles view | Not yet |
| System capabilities view | Not yet |
| Host overrides editing | Implemented (write engine: `SetHostOverride`, `ResetHostOverride`) |
| Raw host domains (storage, firewall, WoL, groups) | Not yet |

---

## What already exists

- Host creation via CLI and write engine.
- Host inspection (read-only) via CLI and GUI page.
- Host override mutation (set/reset) via write engine.
- Hardware facts in the domain model (`Cpu`, `Gpu`, `hasBattery`, etc.).
- Provenance retains host-layer definitions with `host` field.

What is missing is the **full interactive host management surface** (wizard for editing facts, roles, profiles, raw domains).

---

## Exit criteria (when complete)

A host can be created and configured end-to-end.

---

## See also

- [Write Engine](write-engine.md)
- [Provenance](provenance.md)
- [CLI](cli.md)
