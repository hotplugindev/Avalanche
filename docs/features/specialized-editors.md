# Specialized Desktop Editors

**Status: Not yet implemented** (Phase 24)

---

## Designed editors (in order)

| # | Editor | Scope | Status |
|---|---|---|---|
| 1 | Foot | Font, font size, terminal options, per-host overrides | Not yet |
| 2 | Greeter | Tuigreet/DMS selection, autologin, greeter settings | Not yet |
| 3 | Dank Material Shell | Theme, widget list/ordering, weather, monitoring, VPN, control-center, aliased GB settings | Not yet |
| 4 | MangoWC | Monitors, settings values, environment, keybinds, autostart, per-host settings | Not yet |

---

## Design principle

A specialized editor is used when a value has domain semantics that generic Nix forms cannot represent well. Examples:

- MangoWC monitor strings (structured semicolons)
- MangoWC binds (key + action pairs)
- Widget ordering (positional list with constraints)
- Complex desktop settings (nested structured strings)

The specialized editor **parses into a semantic representation** and then produces a **normal Avalanche mutation** through the write engine. It never bypasses the transaction pipeline.

---

## What already exists

- The source engine can classify structured strings as `GeneratedPattern` (safe editing) vs `Computed` (raw only).
- The write engine accepts any `MutationIntent` regardless of which UI produced it.
- The domain model has `Desktop` layer and `gb.home.desktop.*` namespace.

---

## Exit criteria (when complete)

Complex desktop configuration can be managed semantically rather than as raw strings.

---

## See also

- [Generic Option Editor](option-editor.md)
- [Write Engine](write-engine.md)
- [Source Engine](source-engine.md)
