# Repo Doctor

**Status: Implemented** (Phase 9)

The Doctor detects repository inconsistencies and proposes repairs. It never silently fixes anything.

---

## Checks

| Check | Severity | What it detects |
|---|---|---|
| Schema | Error | Missing `gb.schemaVersion` |
| Namespace | Warning | `gb.*` option in an undeclared namespace |
| Imports | Warning | Module imports a file that is not a registered module |
| Aggregate registration | Error | Module not imported by any aggregate |
| Orphan modules | Warning | Registered module defines no configuration |
| Ownership | Error/Warning | Ambiguous, WrongOwner, Unowned-with-impl, External |
| Duplicate ownership | Warning | Same option declared multiple times by one module |
| Path drift | Info | Option defined in a file whose path does not match its option prefix |
| Copy-paste descriptions | Warning | Identical description shared by multiple options |
| Empty profiles | Warning | Profile declares no configuration |
| Broken requests | Error/Warning | Invalid requester grammar, request with no option, unknown requester |
| Untracked configuration | Warning | Capability/Program definition in an unregistered file |

---

## Repair proposals

Each finding may carry a `RepairProposal`:

```rust
pub struct RepairProposal {
    pub action: RepairAction,
    pub description: String,
    pub confidence: f64,  // 0.0..1.0
}
```

### Actions

| Action | Meaning |
|---|---|
| `AddAggregateImport { module_path }` | Add module to the appropriate aggregate |
| `RemoveOrphanModule { module_path }` | Remove or populate an orphan |
| `FixRequesterString { path, requester }` | Correct a malformed requester to canonical grammar |
| `AddSchemaVersion { version }` | Declare `gb.schemaVersion` |
| `ResolveAmbiguousOwnership { option_path }` | Assign option to exactly one owner |
| `DeleteStaleDefinition { option_path, file }` | Remove a definition that no longer applies |

---

## Diagnosis output

```rust
pub struct Diagnosis {
    pub findings: Vec<ValidationFinding>,
    pub repairs: Vec<RepairProposal>,
    pub tier: ValidationTier,
}
```

`has_errors()`, `error_count()`, `warning_count()` summarize severity.

---

## CLI exposure

```bash
avalanche doctor
```

Outputs: error/warning counts, each finding with severity + location + message, repair proposals with confidence percentages. Exit code 1 if errors exist.

---

## Repair workflow

```text
Detect
  ↓
Explain
  ↓
Proposed diff
  ↓
Validate
  ↓
Apply (user confirms)
```

Doctor never silently repairs configuration. Repairs go through the write engine as transactions.

---

## Exit criteria

Known repository problems are automatically detected and explained.

---

## See also

- [Repository Contract](repository-contract.md)
- [Ownership](ownership.md)
- [Write Engine](write-engine.md)
