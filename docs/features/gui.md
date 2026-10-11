# GUI

**Status: Partial** (Phase 11 - frontend scaffolded, desktop shell incomplete)

The GUI is a React + TypeScript desktop client hosted by Tauri 2. It is a **thin client**: it consumes structured data and submits typed commands. It does not parse Nix, determine ownership, select files, perform Git mutations, or implement validation semantics.

---

## What exists

### Frontend (`apps/desktop/frontend`)

Fully scaffolded with:

- React 18 + TypeScript
- Tailwind CSS + shadcn/ui components
- TanStack Query for server state
- Zustand for local UI state
- React Router for navigation
- IPC bridge (`src/ipc/client.ts`) with Tauri/native fallback

#### Pages

| Page | Route | Status |
|---|---|---|
| Dashboard | `/` | Scaffolded |
| Hosts | `/hosts` | Scaffolded |
| Users | `/users` | Scaffolded |
| Profiles | `/profiles` | Scaffolded |
| Applications | `/applications` | Scaffolded |
| Capabilities | `/capabilities` | Scaffolded |
| Graph | `/graph` | Scaffolded |
| Provenance | `/provenance` | Scaffolded |
| Doctor | `/doctor` | Scaffolded |

#### Infrastructure

| Component | File | Purpose |
|---|---|---|
| IPC client | `src/ipc/client.ts` | Tauri invoke or HTTP fallback |
| Queries | `src/queries/index.ts` | TanStack Query hooks |
| Types | `src/types/index.ts` | TypeScript mirrors of Rust model |
| Store | `src/stores/appStore.ts` | Zustand global state |
| Layout | `src/components/layout/Layout.tsx` | Shell with sidebar navigation |
| UI primitives | `src/components/ui/index.tsx` | shadcn-style components |

### Tauri shell (`apps/desktop/src-tauri`)

Configured but **not yet buildable**:

- `tauri.conf.json` exists (dev URL `http://localhost:5173`, frontend build commands wired)
- `src/main.rs` uses `tauri::generate_context!()`
- `src/commands.rs` implements 13 thin command handlers delegating to `ConfigurationEngine`
- **Missing**: `build.rs` (required by `generate_context!`)
- **Missing**: `icons/` directory (referenced by `tauri.conf.json` bundle config)
- **Missing**: native WebKit/GTK libraries on non-Nix hosts (provided inside `nix develop`)

---

## Tauri commands

| Command | Engine method |
|---|---|
| `get_repository` | `repository_view` |
| `get_hosts` | `hosts_view` |
| `get_users` | `users_view` |
| `get_profiles` | `profiles_view` |
| `get_applications` | `applications_view` |
| `get_capabilities` | `capabilities_view` |
| `get_graph` | `graph_view` |
| `get_doctor` | `doctor_view` |
| `get_provenance` | `provenance_view` |
| `search` | `search` |
| `get_why` | `why` |
| `get_owners` | `owners` |
| `get_dependents` | `dependents` |

All handlers are thin: parse request, call engine, return structured JSON.

---

## What is NOT yet implemented

- Desktop window launch (`tauri dev` / `tauri build`)
- Release bundling (icons, `build.rs`)
- Write/mutation UI (no "apply" buttons yet)
- Real-time index refresh
- Notification system
- Accessibility audit

---

## Running the frontend standalone

```bash
cd apps/desktop/frontend
npm install
npm run dev   # http://localhost:5173
```

This renders the UI shell. Data calls fall back to `POST /api/<command>` which will 404 without a mounted API shim. The UI is visually functional but data-less outside the desktop shell.

---

## See also

- [Getting Started](../getting-started.md)
- [Architecture](../architecture.md)
- [CLI](cli.md) (fully working alternative)
