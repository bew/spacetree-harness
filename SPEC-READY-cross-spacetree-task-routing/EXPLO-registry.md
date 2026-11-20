# registry — exploration

## Findings

- (updated) The registry is two-tier: a per-spacetree project registry, read both inside and outside a spacetree; and a global known-spacetrees list, read only from outside. The registry is a harness-wide concept, not a `bdg` detail.
- The bare name is the project `issue_prefix`; `bd` exposes `--json` on every command, so prefixes can be read via `bd info --json` rather than hand-parsing `config.yaml`.
- A spacetree holds an arbitrary tree of projects, so enumeration means walking for `.beads/` dirs.
- `spacetree root --json` yields `{name, path}`, where name is the directory basename.

## Design crux

Where the set of spacetrees comes from, and how the index stays fresh without a daemon,
so bare-name resolution outside a spacetree is trustworthy.

## Candidate directions

- (active) **Global state dir + materialized cache, auto-refreshed.**
- (rejected) **Live scan per run** — no cache; rejected in favor of a cached index to keep `bdg` fast.

## Decisions

- (superseded) **Registry state lives under `$XDG_STATE_HOME/spacetree-global/`.** Superseded by the two-tier model below: only the global known-spacetrees list lives there.
- (superseded) **Materialized cache with auto-refresh.** Superseded: the per-spacetree project registry is lazy-only (no scan, no auto-refresh); the known-spacetrees list prunes dead entries on read.
- (settled) **Project identity is `issue_prefix` only.** Only beads-initialized projects are addressable; no directory-name fallback.
- (settled) **Two-tier registry.** A spacetree keeps its own project registry under `<spacetree>/.state/`; a global known-spacetrees list (name + path only) lives under `$XDG_STATE_HOME/spacetree-global/`. Resolving across spacetrees reads each listed spacetree's registry on demand.
- (settled) **Lazy-only spacetree project registry.** No scan and no TTL; a project stays absent until spacetree-aware tooling interacts with it inside that spacetree. `spacetree registry refresh` rebuilds a spacetree's index explicitly.
- (settled) **Registration is a shared harness primitive.** `spacetree registry lazy-register` is the single idempotent upsert all spacetree-aware tooling calls on interaction; it adds the current project to its spacetree's registry and the current spacetree to the known-spacetrees list.

### Freshness model

Why: lazy registration resolves the spacetree edge exactly when it is natural — you are inside a spacetree when a project there is touched.
The known-spacetrees list is pruned on read, so deletion needs no daemon.
The per-spacetree registry never scans, so an untouched project is invisible until an explicit `refresh`.

## Open threads

- (resolved) Each tier is a single JSON index; registration is the shared `spacetree registry lazy-register` primitive, not private to `bdg`/`bdt`.
