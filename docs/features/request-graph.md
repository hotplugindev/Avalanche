# Request Graph

**Status: Implemented** (Phase 8)

The request graph provides dependency attribution across profiles, capabilities, programs, hosts, users, and options.

---

## Model

```rust
pub struct RequestGraph {
    pub nodes: HashMap<NodeId, String>,
    pub edges: Vec<Edge>,
}

pub struct NodeId { pub kind: NodeKind, pub name: String }
pub struct Edge { pub from: NodeId, pub to: NodeId, pub kind: EdgeKind }
```

### Node kinds

```text
Profile | Capability | Program | Module | Option | Host | User | Desktop | Aggregate | Core
```

### Edge kinds

```text
Requests | Implements | Owns | Overrides | Imports
```

---

## Construction

`build_request_graph(provenance, requests, modules, imports)`:

1. Add nodes for every capability, profile, program, module, option, host, user.
2. Add `Requests` edges from requester nodes to capability nodes (from resolved requests).
3. Add `Implements` edges from capability/program modules to the options they define.
4. Add `Owns` edges from owner modules to their owned options.
5. Add `Overrides` edges from host/user definitions to the options they override.
6. Add `Imports` edges from aggregate modules to their imported children.

---

## Queries

| Method | Purpose |
|---|---|
| `why(node)` | All paths from requesters to the node (explains why it is enabled) |
| `dependents(node)` | What a requester pulls in (transitive closure) |
| `requesters(node)` | Direct requesters |
| `implementation(node)` | Modules that implement an option |
| `owners(node)` | Owner modules |

---

## Core API

```rust
engine.why(index, "audio.pipewire") -> Vec<WhyChain>
engine.dependents(index, "profiles.system.workstation") -> Vec<String>
engine.owners(index, "services.pipewire.enable") -> OwnerEntry
```

`WhyChain`:

```rust
pub struct WhyChain {
    pub via: String,
    pub steps: Vec<String>,
}
```

---

## CLI exposure

```bash
avalanche why audio.pipewire
avalanche dependents profiles.system.workstation
avalanche owners services.pipewire.enable
```

---

## Exit criteria

```bash
avalanche why audio.pipewire
```

works reliably.

---

## See also

- [Ownership](ownership.md)
- [Provenance](provenance.md)
- [CLI](cli.md)
