# Getting Started

This guide covers installing and running Avalanche from source and from a Nix flake, then taking a first tour of the CLI.

Avalanche ships today as a **Rust engine + CLI**, with a **React/Tauri desktop GUI that is partially scaffolded**. The CLI is fully usable. The desktop shell is not yet packaged for release. Plan your expectations accordingly:

| Surface | Status |
|---|---|
| `avalanche` CLI | Usable today |
| Rust engine (`cargo test --workspace`) | Usable today |
| React frontend (`npm run dev`) | Renders, but no data without the desktop shell |
| Tauri desktop window | Scaffolded, not yet buildable (missing `build.rs` + icons) |

---

## Prerequisites

To build from source you need:

- **Rust** (stable, with `rustfmt` and `clippy`)
- **Cargo**
- **Node.js** (22 or newer) and **npm** - only for the frontend
- **Git**
- **Nix** (with `nix-command` and `flakes` enabled) - only for full evaluation-backed indexing and validation; the static engine and CLI work without it

The easiest way to get a reproducible toolchain is the bundled dev flake.

---

## Run from a flake (recommended)

The repository exposes a Nix **development shell** that pins the Rust toolchain, Node.js, and the native libraries the Tauri shell needs.

> Note: the flake currently provides **only `devShells.default`**. It does not yet expose a `packages` or `apps` output, so `nix run .#avalanche` is not available yet. Enter the shell and build inside it.

```bash
cd Avalanche
nix develop

# inside the shell:
cargo build --workspace
cargo test  --workspace
```

If your Nix does not have flakes enabled by default, either add them to `~/.config/nix/nix.conf`:

```text
experimental-features = nix-command flakes
```

or invoke explicitly:

```bash
nix --extra-experimental-features 'nix-command flakes' develop
```

The dev shell prints a reminder of the common commands on entry:

```text
Avalanche development environment
  cargo build --workspace
  cargo test --workspace
  cargo clippy --workspace
  cargo fmt --all -- --check
```

---

## Run from source (no Nix)

### 1. Build the Rust engine and CLI

```bash
git clone <repository-url> Avalanche
cd Avalanche
cargo build --workspace
```

The CLI binary is `avalanche` (crate `avalanche-cli`). Run it directly with cargo:

```bash
cargo run -p avalanche-cli -- --help
```

or install it:

```bash
cargo install --path crates/avalanche-cli
```

### 2. Build and run the frontend (optional)

```bash
cd apps/desktop/frontend
npm install
npm run build        # tsc && vite build -> dist/
```

For live development:

```bash
npm run dev          # Vite dev server on http://localhost:5173
```

> The frontend is a thin client. Outside the desktop shell it falls back to `POST /api/<command>`, so `npm run dev` on its own renders the UI but returns no data until it is hosted by the desktop binary or a compatible API shim is mounted.

### 3. The desktop shell (Tauri) - status

The Tauri 2 shell lives in `apps/desktop/src-tauri`. It is **configured** (`tauri.conf.json` points its dev URL at `http://localhost:5173` and runs the frontend build first), and its command handlers (`src/commands.rs`) are wired to the same `ConfigurationEngine` the CLI uses.

However, the shell is **not yet buildable for release**:

- `apps/desktop/src-tauri/build.rs` is missing (required by `tauri::generate_context!`).
- the `icons/` bundle assets referenced by `tauri.conf.json` are missing.
- the native WebKit/GTK libraries must be present on the host.

Inside `nix develop` the native libraries are provided, but the missing `build.rs` and icons still prevent `tauri dev` / `tauri build` from completing. Treat the desktop window as **pending**; use the CLI for a fully working experience today.

---

## First CLI run

Point the CLI at any Avalanche-managed repository (a directory with a `gb.schemaVersion` declaration and the standard layout). The bundled fixture is a good smoke test:

```bash
# inspect the fixture repository statically (no Nix required)
cargo run -p avalanche-cli -- -r fixtures/simple-eval inspect host testhost

# why is a capability enabled?
cargo run -p avalanche-cli -- -r fixtures/simple-eval why audio.pipewire

# who owns an upstream option?
cargo run -p avalanche-cli -- -r fixtures/simple-eval owners services.pipewire.enable

# repository diagnostics
cargo run -p avalanche-cli -- -r fixtures/simple-eval doctor
```

Global flags:

| Flag | Meaning |
|---|---|
| `-r`, `--repo <PATH>` | repository path (default `.`) |
| `--json` | emit machine-readable JSON instead of text |
| `--nix` / `--eval` | use the full Nix-evaluation index instead of the static index |
| `-h`, `--help` | print help |

By default the CLI uses the **static index** (pure source analysis, no Nix needed). Pass `--nix` to additionally run Nix evaluation for richer semantic data (option schemas, effective values, assertions). See [Nix Engine](features/nix-engine.md) and [Repository Index](features/repository-index.md).

---

## Verify your build

The standard gates, all of which must pass:

```bash
cargo fmt  --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

For the frontend:

```bash
cd apps/desktop/frontend
npm run typecheck
npm run lint
npm run build
```

---

## Next steps

- Read [Architecture](architecture.md) to understand the layers.
- Read the [Concepts](concepts/) pages for the mental model.
- Browse [Features](features/) for what is and is not implemented.
- Use [Reference](reference/gb-namespaces.md) for the namespace and grammar details.
