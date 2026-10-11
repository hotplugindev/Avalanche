# Validation

**Status: Implemented** (Phase 14)

Validation is tiered and integrated into every transaction.

---

## Tiers

| Tier | Name | What it checks | Cost |
|---|---|---|---|
| 1 | Structural | AST validity (parse), schema, ownership, registration, source structure | Cheap (no Nix) |
| 2 | NixEval | Affected `nix eval` + assertions | Moderate |
| 3 | FlakeCheck | `nix flake check` | Expensive |
| 4 | Build | `nix build --dry-run` for affected hosts | Very expensive |

---

## Implementation

```rust
pub struct ValidationService;

impl ValidationService {
    pub fn validate_structural(&self, files: &[(&str, &str)]) -> ValidationReport;
    pub fn validate_nix_eval(&self, nix: &NixService, hosts: &[String]) -> ValidationReport;
    pub fn validate_assertions(&self, nix: &NixService, host: &str) -> ValidationReport;
    pub fn validate_flake_check(&self, nix: &NixService) -> ValidationReport;
    pub fn validate_build(&self, nix: &NixService, hosts: &[String]) -> ValidationReport;
}
```

---

## Structural validation

Parses every changed file with the source engine. A parse error is an `Error` finding. This runs **before** and **after** every write.

---

## Nix eval validation

Runs `nix eval` on affected hosts. If the Nix binary is unavailable, emits a `Warning` (not an error) and skips. Assertion failures are `Error` findings.

---

## Flake check validation

Runs `nix flake check`. Failure is an `Error`. Missing Nix binary is a `Warning`.

---

## Build validation

Runs `nix build --dry-run` on `.#nixosConfigurations.<host>.config.system.build.toplevel` for each affected host. Failure is an `Error`. Missing Nix binary is a `Warning`.

---

## Report structure

```rust
pub struct ValidationReport {
    pub tier: ValidationTier,
    pub findings: Vec<ValidationFinding>,
    pub passed: bool,
}

pub struct ValidationFinding {
    pub severity: ValidationSeverity,  // Error | Warning | Info
    pub message: String,
    pub path: Option<String>,
    pub file: Option<String>,
}
```

`passed` is false if any `Error`-severity finding exists.

---

## Integration with transactions

`execute_mutation_with_tier(index, intents, tier)`:

1. Plan (includes structural pre-flight).
2. Apply.
3. Structural post-flight.
4. If tier >= NixEval: eval + assertions on affected hosts.
5. If tier >= FlakeCheck: `nix flake check`.
6. If tier >= Build: `nix build --dry-run`.
7. Attach combined report to transaction.

---

## Graceful degradation

When Nix is not available (CI, minimal install), Tiers 2-4 emit warnings and skip rather than failing. Tier 1 (structural) always runs and is the minimum gate.

---

## See also

- [Concepts: Transactions](../concepts/transactions.md)
- [Write Engine](write-engine.md)
- [Nix Engine](nix-engine.md)
