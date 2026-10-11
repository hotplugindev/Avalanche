# Priority Values

Numeric priorities used by the Nix module system and retained by Avalanche's provenance model.

---

## Constants

| Constant | Value | Nix equivalent | Meaning |
|---|---|---|---|
| `PRIORITY_VM_OVERRIDE` | 10 | `mkVMOverride` | VM-specific override |
| `PRIORITY_OVERRIDE` | 50 | `mkOverride 50` | Explicit override |
| `PRIORITY_FORCE` | 50 | `mkForce` | Force (same numeric value) |
| `PRIORITY_NORMAL` | 1000 | (default) | Ordinary assignment |
| `PRIORITY_DEFAULT` | 1500 | `mkDefault` | Profile/module default |

---

## Comparison rule

**Lower wins.**

```rust
pub fn wins_over(&self, other: &Priority) -> bool {
    self.0 < other.0
}
```

---

## Ordering examples

```text
mkForce (50) wins over normal (1000) wins over mkDefault (1500)
mkOverride 10 wins over mkForce (50)? No: 10 < 50, so 10 wins.
```

Wait: `PRIORITY_VM_OVERRIDE = 10` is lower than `PRIORITY_FORCE = 50`, so VM override wins over force. This matches Nix semantics.

---

## Tie-breaking (same priority)

When two definitions share the same priority number, the engine breaks ties by **layer rank** (higher rank wins):

| Layer | Rank |
|---|---|
| External | 0 |
| Profile | 1 |
| Capability | 2 |
| Program | 3 |
| Desktop | 4 |
| User | 5 |
| Host | 6 |

If layer rank also ties, the definition from the lexicographically later file path wins (deterministic but arbitrary).

---

## `mkDefault` is NOT a boolean

```text
WRONG:  isProfileDefault = true
RIGHT:  priority = Priority(1500), definition_type = MkDefault
```

The model retains the actual priority value. The UI projects it into "Profile default" for display.

---

## Custom priorities

`lib.mkOverride N` produces `Priority(N)`. The source engine's `analyze_definition` extracts the integer argument and stores it. This allows arbitrary priority values beyond the five constants above.

---

## See also

- [Concepts: Layering & Priority](../concepts/layering-and-priority.md)
- [Provenance](../features/provenance.md)
