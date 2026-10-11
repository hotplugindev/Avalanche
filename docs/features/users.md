# Users

**Status: Not yet implemented** (Phase 21)

---

## Designed capabilities

| Feature | Status |
|---|---|
| User creation | Implemented (write engine: `CreateUser`, CLI: `user create`) |
| Identity view | Implemented (read-only: `inspect user`) |
| Home Manager configuration view | Not yet |
| User packages editing | Not yet |
| User programs editing | Not yet |
| User overrides editing | Not yet |
| Per-host user configuration | Not yet |

---

## What already exists

- User creation via CLI and write engine.
- User inspection (read-only).
- The contract defines explicit user source locations: `users/<username>.nix` (identity), `hosts/<host>/home/<username>.nix` (per-host).
- Provenance retains user-layer definitions.

What is missing is the **full interactive user management surface**.

---

## Exit criteria (when complete)

Multiple users can be independently managed.

---

## See also

- [Repository Contract](repository-contract.md)
- [Write Engine](write-engine.md)
