# Requester Grammar

Canonical form for every entry in a `gb.requires.*` list.

---

## Form

```text
<kind>.<scope>[.<domain>].<name>
```

---

## Kinds (always plural)

| Kind | Scoped? | Example |
|---|---|---|
| `programs` | Yes | `programs.home.ai.codex` |
| `profiles` | Yes | `profiles.system.gaming` |
| `capabilities` | Yes | `capabilities.system.audio.pipewire` |
| `hosts` | No | `hosts.pc` |
| `users` | No | `users.hotplugin` |

---

## Scopes

| Scope | Meaning |
|---|---|
| `system` | NixOS system configuration |
| `home` | Home Manager user configuration |

---

## Rules

1. Kind prefixes are **always plural**. Singular forms (`profile.`, `program.`, `host.`, `user.`, `capability.`) are contract violations.
2. Scoped kinds (`programs`, `profiles`, `capabilities`) require a `system` or `home` segment as the second component.
3. Unscoped kinds (`hosts`, `users`) have exactly two components: `<kind>.<name>`.
4. The `<domain>` segment is optional and only appears for scoped kinds with hierarchical naming.
5. The `<name>` segment is always present and is the final component.

---

## Valid examples

```text
programs.home.ai.codex
programs.system.gaming.steam
profiles.system.workstation
profiles.home.desktop
capabilities.system.audio.pipewire
hosts.pc
hosts.laptop
users.hotplugin
users.alice
```

---

## Invalid examples (rejected by Doctor)

```text
profile.system.development    ← singular "profile"
program.home.spotify          ← singular "program"
host.pc                       ← singular "host"
programs.system               ← missing name
garbage                       ← unknown kind
```

---

## Parsing

```rust
pub fn parse_requester(requester: &str) -> Result<RequesterKind, ModelError>
```

Splits on `.`, validates the first segment as a known plural kind, checks scope for scoped kinds, and verifies minimum component count.

---

## Usage in requests

```nix
# modules/profiles/system/workstation.nix
gb.requires.system.audio.pipewire = [ "profiles.system.workstation" ];

# modules/programs/system/steam.nix
gb.requires.system.audio.pipewire = [ "programs.system.steam" ];
```

The request graph resolves these strings to nodes and builds `Requests` edges.

---

## See also

- [Repository Contract](../features/repository-contract.md)
- [`gb.*` Namespaces](gb-namespaces.md)
- [Request Graph](../features/request-graph.md)
