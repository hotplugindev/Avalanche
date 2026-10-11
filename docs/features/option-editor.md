# Generic Option Editor

**Status: Not yet implemented** (Phase 22)

---

## Designed renderer order

| # | Type | Renderer | Status |
|---|---|---|---|
| 1 | bool | BoolRenderer | Not yet |
| 2 | string | StringRenderer | Not yet |
| 3 | int | IntRenderer | Not yet |
| 4 | float | FloatRenderer | Not yet |
| 5 | enum | EnumRenderer | Not yet |
| 6 | package | PackageRenderer | Not yet |
| 7 | list | ListRenderer | Not yet |
| 8 | attrs | AttrsRenderer | Not yet |
| 9 | attrsOf | AttrsOfRenderer | Not yet |
| 10 | submodule | SubmoduleRenderer | Not yet |
| 11 | freeform | FreeformRenderer | Not yet |
| 12 | raw | RawRenderer (fallback) | Not yet |

---

## Pipeline (designed)

```text
Nix option tree
       ↓
OptionSchema
       ↓
RendererRegistry
       ↓
Renderer
       ↓
UI
```

Specialized renderers can override generic ones.

---

## What already exists

- `OptionSchema` with full type information (from Nix engine).
- `NixType` enum covering all 14 type variants.
- The write engine can `SetOption` for any owned option.
- Provenance shows where the option is defined.

What is missing is the **typed form rendering** and the **renderer registry** that maps types to UI widgets.

---

## Fallback rule

Function-valued, computed, or unknown options are routed to raw editing unless a specialized renderer exists. Avalanche never pretends that arbitrary Nix expressions are simple form values.

---

## Exit criteria (when complete)

Most ordinary NixOS/Home Manager module options can be edited without writing Nix manually.

---

## See also

- [Computed Options](computed-options.md)
- [Raw Nix Editor](raw-nix-editor.md)
- [Write Engine](write-engine.md)
