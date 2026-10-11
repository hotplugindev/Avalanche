# Source Engine

**Status: Implemented** (Phase 4)

The source engine lives in `avalanche-source`. It understands Nix source code structure and enables safe editing without modifying the repository during analysis.

---

## Lexer

Tokenizes Nix source into a stream with:

- identifiers, keywords, numbers, strings, paths
- operators and delimiters
- search paths (`<nixpkgs>`)
- source span tracking for every token

---

## Parser

Recursive descent parser producing a typed AST:

- attribute sets with bindings
- lists, lambdas, let/in, if/then/else
- binary operators with correct precedence
- function application
- with/assert expressions
- inherit statements
- dynamic attribute paths

Every AST node carries position information (line/column of start and end).

---

## Import detection

- extracts `import` expressions from parsed files
- resolves relative and absolute paths
- resolves directory imports (default.nix)
- builds import graphs with cycle detection
- classifies import context (top-level, list, let, attr value)

---

## Assignment extraction

Walks the AST to extract all assignments with their:

- dotted attribute path
- value expression
- source span
- classification

Nested attribute sets are flattened into dotted paths (e.g. `services.pipewire.enable`).

---

## Assignment classification

| Kind | Examples | Auto-editable? |
|---|---|---|
| `DirectLiteral` | `true`, `42`, `"hello"`, `./path`, `[ "a" "b" ]` | Yes |
| `DirectExpression` | identifiers, binary ops, `mkDefault`, `mkForce`, `import` | Only when understood |
| `GeneratedPattern` | recognized generated patterns | Yes (specialized) |
| `Computed` | conditionals, lambdas, unknown function applications | No - raw |
| `Unknown` | inherit, unrecognized structures | No - raw |

---

## Definition analysis

`analyze_definition(expr)` classifies an expression into:

- `DefinitionType` (Literal / MkDefault / MkForce / MkIf / MkMerge / MkOverride / Expression / Unknown)
- `Priority` (extracted from mkDefault=1500, mkForce=50, mkOverride=N)
- `Condition` (for mkIf: preserves the expression string)
- `value` (for literals/lists/attrsets: converted to JSON)

This feeds the provenance index.

---

## Formatting

Pretty-prints AST back to Nix source:

- consistent 2-space indentation
- proper string escaping
- correct operator spacing
- multi-line attr sets and lists

---

## Source patch generation

Generates targeted source modifications:

- value replacement at exact span
- binding removal
- binding insertion with correct indentation
- multi-patch application with reverse-order safety

Patches are applied to in-memory strings, never directly to disk. The write engine controls when patches are committed.

---

## Exit criteria

Avalanche can answer:

> Where is this value actually written?

without modifying the repository.

---

## See also

- [Nix Engine](nix-engine.md)
- [Provenance](provenance.md)
- [Write Engine](write-engine.md)
