# Repository Initialization

**Status: Implemented** (Phase 17)

`avalanche init` creates a conforming Avalanche-managed Nix repository from scratch.

---

## What it generates

```text
<target>/
├── flake.nix
├── hosts/
│   └── <host>/
│       └── default.nix
├── users/
│   └── <user>.nix
├── modules/
│   ├── aggregate/
│   │   ├── nixos.nix
│   │   └── home.nix
│   └── core/
│       ├── schema.nix
│       ├── host/options.nix
│       ├── user/options.nix
│       ├── requests/system.nix
│       └── requests/home.nix
└── .git/
```

---

## Contract bootstrap

The generated repository declares `gb.schemaVersion = 1` and includes:

- Core option modules (host, user, requests, debug)
- Aggregate registration scaffolding
- Initial host and user files

---

## Git initialization

After generating files, `avalanche init`:

1. Runs `git init` in the target directory.
2. Stages all files (`git add -A`).
3. Creates an initial commit.

---

## Validation

The generated skeleton is validated against `schemas/contract.nix` before the command exits. A repository that fails validation is rolled back (directory removed).

---

## Idempotent re-init

Running `avalanche init` on an existing conforming repository adds a new host or user without destroying existing configuration.

---

## CLI

```bash
avalanche init ~/my-config --host pc --user alice
avalanche init ~/my-config --host pc --user alice --full-name "Alice Smith" --email alice@example.com --system x86_64-linux
```

Flags:

| Flag | Required | Default |
|---|---|---|
| `<path>` (positional) | Yes | - |
| `--host <name>` | Yes | - |
| `--user <name>` | Yes | - |
| `--full-name <name>` | No | empty |
| `--email <email>` | No | empty |
| `--system <triple>` | No | `x86_64-linux` |

---

## Exit criteria

```bash
avalanche init ~/my-config --host pc --user alice
avalanche doctor  # zero errors
```

Produces a repository that passes `avalanche doctor` with zero errors and can be immediately indexed.

---

## See also

- [Repository Contract](repository-contract.md)
- [Write Engine](write-engine.md)
- [CLI](cli.md)
