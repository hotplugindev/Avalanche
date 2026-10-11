# Profiles

**Status: Not yet implemented** (Phase 19)

---

## Designed capabilities

| Feature | Status |
|---|---|
| Profile list | Not yet |
| Activation conditions view | Not yet |
| Requested capabilities view | Not yet |
| Default programs view | Not yet |
| `mkDefault` editing | Implemented (write engine: `SetProfileDefault`) |
| Profile editor (create/edit/delete) | Not yet |
| Host override editor | Not yet |
| Reset semantics (delete host definition) | Implemented (write engine: `ResetHostOverride`) |

---

## What already exists

- The domain model represents `Profile` with scope and source file.
- The index discovers profiles from `modules/profiles/`.
- The write engine can set profile defaults (`lib.mkDefault`) and reset host overrides.
- The Doctor flags empty profiles.
- Provenance retains profile-layer definitions.

What is missing is the **interactive editing surface** for managing profile contents.

---

## Exit criteria (when complete)

Profile/default/override/effective semantics are completely supported.

---

## See also

- [Concepts: Layering & Priority](../concepts/layering-and-priority.md)
- [Write Engine](write-engine.md)
- [Provenance](provenance.md)
