# Avalanche

Avalanche is a native desktop configuration manager for structured **NixOS + Home Manager** repositories.

It treats the Nix repository as the source of truth and derives a typed index for inspection, provenance, ownership, dependency analysis, transactional mutation, and validation.

> **Nix is the database. The index is a cache. Rust is the configuration engine. The GUI is a client.**

## Why Avalanche?

- Understand what a configuration value means, not just what it is set to.
- Trace every effective value back to its source file, span, layer, priority, host, user, profile, and condition.
- Separate owner, requester, and implementer so capability dependencies are explicit.
- Mutate configuration through transactional plans, diffs, validation, and rollback.
- Use the same engine from the CLI and the desktop GUI.

## Current status

| Surface | Status |
|---|---|
| Rust engine + CLI | Usable today |
| React frontend | Scaffolded; renders without data outside the desktop shell |
| Tauri desktop shell | Configured but not yet packaged (missing `build.rs` and icons) |
| Documentation | Extensive docs live in [`docs/`](docs/README.md) |

## Install and run

### From a flake

```bash
nix develop
cargo build --workspace
cargo test --workspace
```

The flake currently provides a development shell only. It does not yet expose `packages` or `apps` outputs.

### From source

```bash
git clone <repository-url> Avalanche
cd Avalanche
cargo build --workspace
cargo run -p avalanche-cli -- --help
```

### CLI

```bash
cargo run -p avalanche-cli -- -r fixtures/simple-eval inspect host testhost
cargo run -p avalanche-cli -- -r fixtures/simple-eval why audio.pipewire
cargo run -p avalanche-cli -- -r fixtures/simple-eval doctor
```

### Frontend

```bash
cd apps/desktop/frontend
npm install
npm run dev
```

The Vite dev server runs on `http://localhost:5173`. Outside the Tauri shell the frontend falls back to `POST /api/<command>` and will not receive data until the desktop binary or an API shim is mounted.

## Documentation

Start with [`docs/README.md`](docs/README.md). It contains getting-started guides, architecture, concepts, feature-by-feature status, and reference material.

## License

Avalanche is licensed under the Apache License 2.0. See [`LICENSE`](LICENSE).
