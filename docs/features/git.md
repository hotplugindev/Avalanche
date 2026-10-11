# Git Integration

**Status: Partial** (Phase 26 - engine implemented, UI integration pending)

---

## Implemented (engine)

| Operation | Method | Status |
|---|---|---|
| Status (branch, ahead/behind, staged, modified, untracked, deleted, conflicted) | `GitService::status()` | Implemented |
| Diff (working tree) | `GitService::diff()` | Implemented |
| History (N most recent commits) | `GitService::history(n)` | Implemented |
| Branches (local + remote) | `GitService::branches()` | Implemented |
| Current branch | `GitService::current_branch()` | Implemented |
| Create branch | `GitService::create_branch(name)` | Implemented |
| Checkout branch | `GitService::checkout_branch(name)` | Implemented |
| Commit (add -A + commit) | `GitService::commit(message)` | Implemented |
| Pull | `GitService::pull()` | Implemented |
| Push | `GitService::push()` | Implemented |
| Rollback paths | `GitService::rollback(paths)` | Implemented |
| Conflicts (with base/ours/theirs stages) | `GitService::conflicts()` | Implemented |
| Init (git init + add + commit) | `GitService::init(path)` | Implemented |
| HEAD hash | `GitService::head()` | Implemented |
| Dirty check | `GitService::is_dirty()` | Implemented |

---

## Not yet implemented (UI/CLI)

| Feature | Status |
|---|---|
| Git panel in GUI | Not yet |
| `avalanche git` CLI subcommand | Not yet |
| Conflict resolution UI | Not yet |
| Branch management UI | Not yet |
| History browser | Not yet |
| Integration with transaction review | Not yet |

---

## Architecture

Git is **independent** from write planning:

```text
Write engine
      ↓
working tree
      ↓
Git service
```

The user can make multiple Avalanche changes before committing. Git provides the review and lifecycle layer on top.

---

## Fingerprint integration

The repository fingerprint includes `git_head` and `working_tree_hash`. Any Git operation that changes HEAD or the working tree invalidates the index, forcing a re-index before the next mutation.

---

## Exit criteria (when complete)

Users can review and manage Avalanche changes through Git.

---

## See also

- [Repository Index](repository-index.md)
- [Write Engine](write-engine.md)
- [CLI](cli.md)
