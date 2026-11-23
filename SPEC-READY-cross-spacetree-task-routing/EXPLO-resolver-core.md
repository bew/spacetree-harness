# resolver-core — exploration

## Findings

- `bd` resolves a workspace by walking up for `.beads/`; it has no concept of multiple spacetrees.
- Spacetree root detection is already solved: `.spacetree-repo` marker plus `spacetree root --json`, which prints the root path, the repo path, the shared root, the machine-state label, and the configured name.
- `issue_prefix` is assigned per project at init time by `wk init` (`--prefix`, interactive prompt); it is the identity `bd` filters every query on.
- Addressing candidate grammar from the originating handoff: `bdg <spacetree>:<project>`, `bdg <project>`, `bdg <spacetree>`, and `bdt` (spacetree pre-resolved from cwd).
- Gas City's `gc bd --rig <name>` establishes the ceiling for cross-spacetree routing: chdir + exec, no cross-rig joining.

## Design crux

Resolve a user-typed target into exactly one beads workspace to route into,
across an open-ended set of spacetrees that `bd` itself knows nothing about,
deterministically and never silently ambiguous.

## Candidate directions

- (rejected) **Qualified-only** — drop bare-name resolution entirely, require `spacetree:project`. Simplest and collision-free, but loses the ergonomics the layer exists to provide.
- (rejected) **Enforced disjoint namespaces** — guarantee globally-unique project prefixes (and keep the spacetree-name and prefix axes disjoint) at creation time. Superseded: forces a global naming constraint and still needs a resolver for unmanaged projects.

## Decisions

- (settled) **Two-mode resolution: cwd-scoped vs registry-scoped.**
  Inside a spacetree, a bare project name resolves within the cwd spacetree only; other spacetrees are never consulted and no collision check runs.
  Outside any spacetree, a bare project name resolves across every known spacetree's project registry: a unique match resolves, a collision is rejected and the qualified `<spacetree>:<project>` form is required.

### Two-mode resolution

Why: inside a spacetree the cwd is an unambiguous scope, so global collision checking buys nothing and only adds latency and failure modes.
Outside, there is no scope, so the registry is the only source and ambiguity must be surfaced, never guessed.

### Bare spacetree name

Confirmed behavior: `bdg <spacetree>` targets that spacetree's root scope.

### Name/prefix clash

- (settled) **Spacetree name wins.** When a name `X` is both a spacetree name and a project prefix somewhere, `bdg X` resolves the spacetree; the project is reached via `<spacetree>:X`.
  Keeps the coarse address stable; the qualified form is the project escape hatch.

### No global prefix-uniqueness enforcement

- (settled) Project prefixes may collide across spacetrees; this is not checked at creation time.
  Collisions are handled entirely at resolution time, and only from outside a spacetree.

### `bdt` is strictly cwd-scoped

- (settled) `bdt` resolves the spacetree from cwd and reads only that spacetree's project registry; it never touches the known-spacetrees list.
  It is the cwd-scoped form of `bdg`.

## Open threads

- (resolved) Bare spacetree-name resolution is global: `bdg <spacetree>` works from inside another spacetree too.
- Registry format, refresh, and discovery mechanism — deferred to the `registry` topic.
- Pure routing vs. an aggregate/reporting path — deferred to the `routing-vs-aggregate` topic.
- Coexistence with a Nix-wrapped `bd` that owns the shared-server socket — deferred to the `bd-wrapper-interaction` topic.
