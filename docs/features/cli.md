# CLI

**Status: Implemented** (Phase 10)

The CLI (`avalanche-cli`) uses exactly the same Rust engine as the GUI. No duplicated configuration logic.

---

## Usage

```text
avalanche [OPTIONS] <COMMAND>
```

### Global options

| Flag | Meaning |
|---|---|
| `-r`, `--repo <PATH>` | Repository path (default `.`) |
| `--json` | Emit JSON instead of human-readable text |
| `--nix` / `--eval` | Use full Nix-evaluation index (default: static only) |
| `-h`, `--help` | Print help |

---

## Commands

### Read-only inspection

| Command | Purpose |
|---|---|
| `inspect <kind> <name>` | Inspect a host, user, profile, option, module, capability, program, desktop, aggregate, or core |
| `why <capability\|option>` | Explain why an effective value exists (requester chains) |
| `dependents <node>` | What a requester pulls in (transitive dependents) |
| `owners <option-path>` | Who owns an upstream option |
| `doctor` | Repository diagnostics + repair proposals |
| `diff` | Working-tree diff |

### Mutation

| Command | Purpose |
|---|---|
| `host create <name>` | Create a new host |
| `user create <name>` | Create a new user |
| `generate <kind> <name>` | Generate a module (program, capability, profile) |
| `init <path> --host <h> --user <u>` | Initialize a new repository from scratch |

---

## Examples

```bash
# inspect a host
avalanche -r ~/my-config inspect host pc

# inspect an option with full provenance
avalanche -r ~/my-config inspect option services.pipewire.enable

# why is pipewire enabled?
avalanche -r ~/my-config why audio.pipewire

# what does the workstation profile pull in?
avalanche -r ~/my-config dependents profiles.system.workstation

# who owns an upstream option?
avalanche -r ~/my-config owners services.pipewire.enable

# run diagnostics
avalanche -r ~/my-config doctor

# JSON output for scripting
avalanche -r ~/my-config --json doctor

# use Nix evaluation for richer data
avalanche -r ~/my-config --nix inspect option services.pipewire.enable

# create a new host
avalanche -r ~/my-config host create laptop

# generate a new program module
avalanche -r ~/my-config generate program spotify

# initialize a brand-new repository
avalanche init ~/my-new-config --host pc --user alice --full-name "Alice" --email alice@example.com
```

---

## Index modes

| Mode | Flag | Requires Nix? | Data available |
|---|---|---|---|
| Static (default) | none | No | Assignments, source locations, provenance (from source), modules, imports, fingerprints |
| Full | `--nix` | Yes | + option schemas, effective values, assertions, resolved requests, flake inputs |

Most inspection commands work in static mode. `why` and `dependents` benefit from semantic data but degrade gracefully.

---

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Runtime error (index failure, unknown entity, doctor errors) |
| 2 | Usage error (bad flags, missing arguments) |

---

## See also

- [Getting Started](../getting-started.md)
- [Architecture](../architecture.md)
- [Repository Index](repository-index.md)
