# Import / Export / Share

**Status: Not yet implemented** (Phase 27)

---

## Designed capabilities

| Feature | Status |
|---|---|
| Host export (files + profiles + capabilities + metadata) | Not yet |
| Profile export (profile + transitive capability closure) | Not yet |
| Capability closure computation | Not yet |
| Import validation (schema, namespaces, registration, ownership, dependencies) | Not yet |
| Merge preview | Not yet |
| Transactional import | Not yet |
| Schema compatibility check | Not yet |

---

## Safety rules (designed)

- Never trust paths from imported bundles.
- Validate against the repository contract before applying.
- Import is a transaction: partial failure rolls back completely.
- External dependencies are never edited during import.
- Ownership conflicts are detected and require explicit resolution.

---

## What already exists

- The contract validation infrastructure (Doctor checks schema, namespaces, registration, ownership).
- The transaction engine (plan → validate → apply → rollback).
- The request graph (can compute capability closures).
- Fingerprint/staleness detection.

---

## Exit criteria (when complete)

Configuration can be shared without bypassing repository invariants.

---

## See also

- [Repository Contract](repository-contract.md)
- [Write Engine](write-engine.md)
- [Request Graph](request-graph.md)
