# Module Generation

**Status: Implemented** (Phase 16)

Avalanche can create new application modules safely, without manual registration edits.

---

## What it generates

| Kind | Files created | Registration |
|---|---|---|
| `program` | `modules/programs/<scope>/<category>/<name>.nix` | Added to aggregate + category index |
| `capability` | `modules/capabilities/<scope>/<domain>/<name>.nix` | Added to aggregate + request option declared |
| `profile` | `modules/profiles/<scope>/<name>.nix` | Added to aggregate |

---

## Pipeline

```text
intent (CreateModule { name, kind })
  → validate name (Nix identifier: letters, digits, '-', '_')
  → determine scope, category/domain from name
  → generate module template
  → generate package placement (for programs)
  → generate option declarations (for capabilities)
  → generate capability requests (for programs)
  → generate category registration
  → generate aggregate registration
  → generate ownership metadata
  → write all files atomically
  → validate (structural)
  → rollback on failure
```

---

## Templates

### Program module

```nix
{ config, lib, pkgs, ... }:
{
  options.gb.programs.system.<category>.<name> = lib.mkOption {
    type = lib.types.submodule {
      options.enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };
    };
    default = { };
  };

  config = lib.mkIf config.gb.programs.system.<category>.<name>.enable {
    # package placement, capability requests
  };
}
```

### Capability module

```nix
{ config, lib, pkgs, ... }:
{
  options.gb.requires.system.<domain>.<name> = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    default = [ ];
  };

  config = lib.mkIf (config.gb.requires.system.<domain>.<name> != [ ]) {
    # implementation
  };
}
```

---

## Safety

- Module creation is transactional: if any generated file fails to parse, all created files are removed.
- Aggregate registration is validated: a module not reachable from an aggregate is inert and flagged by the Doctor.
- Names are validated as Nix identifiers.
- The generated module is immediately indexable.

---

## CLI

```bash
avalanche generate program spotify
avalanche generate capability audio.pipewire
avalanche generate profile workstation
```

---

## Exit criteria

A new application can be generated without manual registration edits.

---

## See also

- [Write Engine](write-engine.md)
- [Repository Contract](repository-contract.md)
- [Repo Doctor](repo-doctor.md)
