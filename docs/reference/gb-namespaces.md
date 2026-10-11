# `gb.*` Namespaces

Every registered namespace and its semantics. Defined in `schemas/contract.nix`, mirrored in `avalanche-model::contract`.

---

## Summary table

| Namespace | Scope | Kind | Rust type | Description |
|---|---|---|---|---|
| `gb.schemaVersion` | both | meta | `int` | Contract version the repository conforms to (currently `1`) |
| `gb.host` | system | facts | `submodule` | Typed host identity and hardware facts |
| `gb.user` | both | identity | `submodule` | Active user identity exposed to modules |
| `gb.requires.system` | system | request | `attrsOf (listOf str)` | System capability requests |
| `gb.requires.home` | home | request | `attrsOf (listOf str)` | Home capability requests |
| `gb.programs.system` | system | application | `attrsOf submodule` | Typed system application settings |
| `gb.home.programs` | home | application | `attrsOf submodule` | Typed home application settings |
| `gb.home.desktop` | home | desktop | `attrsOf submodule` | Structured desktop configuration |
| `gb.debug` | system | debug | `attrsOf bool` | Diagnostic switches |

---

## `gb.schemaVersion`

```nix
gb.schemaVersion = 1;
```

- **Required** in every managed repository.
- Declares conformance to the Avalanche repository contract.
- Avalanche validates against this version before indexing.
- Missing → Doctor error + repair proposal (`AddSchemaVersion`).

---

## `gb.host`

Typed host identity and hardware facts. Hosts describe the machine, never implement services.

### Fields

| Field | Type | Description |
|---|---|---|
| `gb.host.name` | str | Hostname |
| `gb.host.system` | str | Nix system triple (e.g. `x86_64-linux`) |
| `gb.host.class` | enum | `desktop` / `laptop` / `server` / `vm` |
| `gb.host.roles` | listOf enum | `workstation` / `gaming` / `development` / `server` / `hybrid` |
| `gb.host.desktop` | enum | `mango` / `dms` / `gnome` / `plasma` / `none` |
| `gb.host.shell` | enum | `zsh` / `fish` / `bash` / `nu` |
| `gb.host.stateVersion` | str | NixOS state version |
| `gb.host.hardware.cpu` | enum | `amd` / `intel` / `arm` |
| `gb.host.hardware.gpu` | enum | `amd` / `intel` / `nvidia` / `none` |
| `gb.host.hardware.hasBattery` | bool | Battery present |
| `gb.host.hardware.hasBluetooth` | bool | Bluetooth present |
| `gb.host.hardware.hasTouchpad` | bool | Touchpad present |
| `gb.host.hardware.hasPrinter` | bool | Printer present |

---

## `gb.user`

Active user identity exposed to modules.

| Field | Type |
|---|---|
| `gb.user.username` | str |
| `gb.user.fullName` | str |
| `gb.user.email` | str |

---

## `gb.requires.system` / `gb.requires.home`

Capability request lists. The value is a list of canonical requester strings.

```nix
gb.requires.system.audio.pipewire = [ "programs.system.steam" "profiles.system.workstation" ];
```

- Type: `attrsOf (listOf str)` (nested by domain.name)
- Activation: non-empty list → capability module activates
- Requester grammar: see [Requester Grammar](requester-grammar.md)

---

## `gb.programs.system` / `gb.home.programs`

Typed application settings. Each application is a submodule with at minimum an `enable` option.

```nix
gb.programs.system.gaming.steam.enable = true;
gb.home.programs.ai.opencode.enable = true;
```

---

## `gb.home.desktop`

Structured desktop configuration (Mango, DMS, etc.).

```nix
gb.home.desktop.mango.monitors = [ ... ];
gb.home.desktop.dms.theme = "catppuccin";
```

---

## `gb.debug`

Diagnostic switches. Never affects production configuration semantics.

```nix
gb.debug.traceEvaluation = true;
```

---

## Undeclared namespaces

Any `gb.*` option outside the registered namespaces is flagged as a warning by the Doctor:

```text
option 'gb.unknown.thing' is in an undeclared gb.* namespace
```

---

## See also

- [Repository Contract](../features/repository-contract.md)
- [Requester Grammar](requester-grammar.md)
- [Domain Model](../features/domain-model.md)
