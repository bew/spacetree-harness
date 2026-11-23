# [READY] Cross-spacetree Task Routing

## Introduction

Beads (`bd`) tracks work in a single workspace, which it locates from `BEADS_DIR` when the wrapper sets it; every query inside that workspace is scoped to its own `issue_prefix`.
That model is exactly right inside one project, which is why per-project isolation needs no extra tooling.
The workspace itself is no longer assumed to live in the repository: see `<spec:beads-out-of-tree-storage>`.

It has no notion, however, of the many independent projects a developer keeps on one machine, nor of the spacetrees that group them (see `README.md` for the spacetree definition).
There is no way to address a different project, or a different spacetree, from a shell without changing directory first, and no coherent scheme for naming such a target.

`wkg` and `wkt` are the addressing layer that fills that gap.
They take a user-typed target, resolve it to exactly one workspace, and route the invocation there: move to that directory, then exec `wk`.
They are pure routing — one command, one project — and deliberately do not reimplement `wk`.

The user-facing surface is the `wk`/`wkg`/`wkt` frontend; see `<spec:wk-beads-wrappers>`.

The layer serves the human first, per the harness constraint in `README.md`: short commands, from anywhere, with agent support as a layer on top.
Two entry points cover the two situations a person works from.

`wkg` addresses any known spacetree from anywhere: it resolves a bare name in the enclosing spacetree when there is one, and otherwise across all known spacetrees.
`wkt` is the in-spacetree form: it resolves the enclosing spacetree from the current directory and only ever reads that spacetree's project registry.

The design stays within the precedent set by Gas City's `gc bd --rig <name>`, which routes by changing directory and exec'ing `bd` unmodified.
Routing therefore stays a matter of choosing a directory, not of reimplementing the query layer.


## Terminology & Key Concepts

**`wkg`** (new!):
The cross-spacetree routing command.
It resolves a target, then routes the invocation (via `wk`) there.

**`wkt`** (new!):
The in-spacetree routing command.
It resolves the enclosing spacetree from the current directory and only ever reads that spacetree's project registry.

**qualified form** (new!):
A target written as `<spacetree>:<project>`, naming both axes explicitly.

**bare form** (new!):
A target written as `<project>` or `<spacetree>`, with no spacetree qualifier.

**resolver** (new!):
The logic that maps a target to exactly one beads workspace, or rejects it as ambiguous.

**spacetree-aware tooling** (new!):
Tooling that resolves the enclosing spacetree and follows harness conventions, such as calling `spacetree registry lazy-register` on interaction.

**spacetree root** (new!):
The directory carrying the `.spacetree-repo` marker, a symlink whose target is the spacetree repo; the enclosing spacetree is found by walking up to it (see `<spec:spacetree-repo>`).

**spacetree project registry** (new!):
A spacetree's own lazy index of its projects, stored under `<spacetree root>/.spacetree/machine-state/`.
A project appears once spacetree-aware tooling has interacted with it inside that spacetree, and is identified by its project key.

**known-spacetrees list** (new!):
The global list of known spacetrees, held as name and path under `$XDG_STATE_HOME/spacetree-global/`.

**lazy registration** (new!):
The idempotent upsert, exposed as `spacetree registry lazy-register`, that spacetree-aware tooling calls on interaction to add the current project to its spacetree's project registry and the current spacetree to the known-spacetrees list.

**cwd-scoped resolution** (new!):
Resolving a target within the enclosing spacetree of the current directory only, as `wkt` and in-spacetree `wkg` do.

**registry-scoped resolution** (new!):
Resolving a bare project across every known spacetree's project registry, used by `wkg` outside any spacetree.


## Naming & IDs

**Commands**:
`wkg` (cross-spacetree) and `wkt` (in-spacetree).

**Target forms**:

```pseudocode
target    := address [ ":" scope ]
address   := qualified | bare-project | bare-spacetree
qualified := <spacetree-name> ":" <project-key>
```

- `address` — the routing target proper: a spacetree and/or a project.
- `qualified` — `work:cmdo-nix`.
- `bare-project` — `cmdo-nix`.
- `bare-spacetree` — `work`.
- `project-key` — the project's declared key (see `<spec:project-info>`), not its beads prefix.
- `scope` — optional third segment (`work:cmdo-nix:auth`), rewritten into `bd` label arguments; see *Routing semantics* and the task-scopes spec.

**Command shape**:

```sh
wkg <target> [bd-args…]
wkt <project-key> [bd-args…]     # spacetree taken from cwd
```

The first positional argument is always the target; everything after it is forwarded verbatim to `wk`.
A missing target is an error: `wkt` does not fall back to the current project, and plain `wk` is used for that.

## Interface / How to use

Both commands take a target as their first positional argument and forward everything after it to `wk` unchanged.

### `wkg` — cross-spacetree routing

```sh
# Qualified: spacetree and project named explicitly. Resolves from anywhere.
wkg self:ghh list --status=open

# Bare project.
# Inside a spacetree, resolved in that spacetree only.
# Outside a spacetree, resolved across all known spacetrees.
wkg cmdo-nix ready

# Bare spacetree: targets the spacetree root's own scope.
wkg work list

# Arguments are forwarded verbatim, including cross-project dependency targets.
# The external: argument is stored by bd as an edge it never resolves; see Global Open Questions.
wkg work:cmdo-nix dep add commando-nix-a1b2 external:ghh:build
```

### `wkt` — in-spacetree routing

```sh
# Spacetree taken from the current directory; only the project is named.
wkt ghh list
wkt cmdo-nix ready

# Same, from anywhere under the spacetree.
cd ~/self/my-projects/ghh/subdir && wkt cmdo-nix list
```

### Ambiguity and errors

```console
# Outside a spacetree, a colliding bare project is rejected, never guessed:
$ wkg api list
!! ERROR: 'api' is ambiguous across spacetrees: self, work
   qualify as 'wkg self:api' or 'wkg work:api'

# A project unknown to the lazy registry has nowhere to route:
$ wkg ghh list
!! ERROR: 'ghh' project is unknown in spacetree 'self'
```

### Routing semantics

`wkg`/`wkt` resolve the target to a workspace directory, change into it, and exec `wk` from `PATH` with the forwarded arguments.
Routing adds no rewriting of its own, save for one: a target's `:scope` segment is turned into `bd` label arguments by the `wk` wrapper before it invokes `bd` (see the task-scopes spec). There is no other output rewriting or query interception.

`wkg`/`wkt` stream `wk`'s output and propagate its exit status, so they are transparent for piping and scripting.
Exec'ing `wk` from `PATH` is deliberate: it lets the target project's `wk` wrapper resolve the project info and set up its beads workspace (`BEADS_DIR`), then reach the bundled `bd` (see `<spec:wk-beads-wrappers>` and *Placement / Scope*).

## Resolution rules

Resolution depends on two inputs: the target form, and whether the current directory is inside a spacetree.
The enclosing spacetree is found by walking up for the `.spacetree-repo` marker.

### Spacetree names are global; project names are scoped

A bare name is matched against spacetree names first.
On a match, the target is the spacetree root's own scope.

Spacetree names are global and win over any project key of the same spelling, so `wkg work` always means the spacetree `work`, never a project named `work`.

When the bare name is not a spacetree name, it is treated as a project and resolved by scope:
- Current directory inside a spacetree: resolved in that spacetree's project registry only.
  Other spacetrees are never consulted, so a project key that also exists elsewhere is not a collision.
- Current directory outside any spacetree: resolved across every known spacetree's project registry.

NOTE: The spacetree project registry is lazy-only, so an in-spacetree bare project resolves only once something has registered it — typically a prior interaction inside it.
A never-touched project stays unknown until then.

### Qualified targets

`<spacetree>:<project>` resolves the spacetree axis globally and the project axis within that spacetree.
The qualified form is always available and is the way to reach a project in another spacetree.

### Collisions

- Outside a spacetree, a bare project name that matches more than one spacetree is rejected, never tie-broken: the error lists the candidate spacetrees and asks for the qualified form.
- Project keys are not required to be globally unique.
  Collisions are handled here, at resolution time, not enforced at creation.

### Misses

- An unknown spacetree name is an error.
- A project absent from the resolved spacetree is an error; resolution never falls back to another spacetree.

### `wkt`

`wkt` resolves the spacetree from the current directory and the project within that spacetree's project registry only.
It never reads the known-spacetrees list.

## Registry

The registry is a harness-wide concept, not a `wkg` detail.
It has two tiers, both populated lazily by spacetree-aware tooling and each stored as a single JSON index.

This spec defines the contract `wkg`/`wkt` rely on; the registry's own commands live in the `spacetree` CLI.

### Spacetree project registry

Each spacetree keeps its own index of its projects under `<spacetree root>/.spacetree/machine-state/`.
A project enters the index once spacetree-aware tooling interacts with it inside that spacetree; its identity is its project key, and the entry records the project key and repository path.

The index is lazy-only: no scan, no TTL, and a project never interacted with stays absent.
`spacetree registry refresh` is the explicit escape hatch that rebuilds a spacetree's index.

### Known-spacetrees list

The global list is held under `$XDG_STATE_HOME/spacetree-global/` and stores each spacetree's name and path only — no project data.
A spacetree enters the list when any spacetree interaction happens inside it.

Resolving across spacetrees reads each entry's project registry from its recorded path, on demand.

### Lazy registration

`spacetree registry lazy-register` is the single upsert all spacetree-aware tooling calls on interaction.
It adds the current project to its spacetree's project registry, and the current spacetree to the known-spacetrees list, each only if not already present.
It is idempotent.

`wkg`/`wkt` are ordinary consumers of this contract, no different from any other spacetree-aware tool.

### Freshness

Lazy-only; there is no TTL and no daemon.
The known-spacetrees list prunes an entry whose spacetree directory no longer exists when the list is read.
A spacetree project registry only changes through interaction or an explicit `refresh`.

## Placement / Scope

### State placement

- Spacetree project registries live under `<spacetree root>/.spacetree/machine-state/`, the state root holding what is true of this machine alone (see `<spec:spacetree-repo>`); they are never committed to a project repo.
- The known-spacetrees list lives under `$XDG_STATE_HOME/spacetree-global/`.
- `wkg`/`wkt` write no state inside a project repo; a project carries only its project info file (tracked, or a linked symlink), and its beads workspace lives under the spacetree's `.spacetree/shared-state/` (see `<spec:beads-out-of-tree-storage>`).

### In scope

- Resolving a target to exactly one beads workspace.
- Routing to it (change directory, then exec `wk`), preserving arguments, output, and exit status.
- Consuming the harness registry contract: reading registries for resolution, and calling `spacetree registry lazy-register` on interaction.

### Out of scope

- **Aggregate or cross-project reporting.** `wkg`/`wkt` route one command to one project; there is no `wkg all …`.
- **The `wk` frontend.** The `wk`/`wkg`/`wkt` binaries, the `wk` wrapper, its packaging, and its `wk init` are owned by `<spec:wk-beads-wrappers>` and `<spec:wk-init>`; this spec only relies on `wk` being on `PATH`.
- **Registry management commands.** `spacetree registry lazy-register` and `spacetree registry refresh` belong to the `spacetree` CLI; this spec fixes only their contract.
- **Scope semantics.** The `:scope` model and its translation into `bd` label arguments belong to the task-scopes spec; this spec only reserves the optional third target segment.
- **`bd` itself** and its issue model, which `wkg`/`wkt` do not reimplement.

## Alternatives & Tradeoffs

The chosen direction is a two-command routing layer (`wkg`/`wkt`) over a two-tier lazy registry.

### Option A — pure routing (chosen)

```sh
wkg work:cmdo-nix list --status=open
# -> chdir <spacetree root>/cmdo-nix, exec wk list --status=open
```

- Advantages: minimal surface; `bd` stays the single source of truth; arguments, output, and exit status pass through untouched, bar the `:scope` rewrite; matches Gas City's `gc bd --rig` ceiling.
- Costs: no cross-project view; every answer comes from a single project.

### Option B — routing plus aggregate reporting

```sh
wkg all list --status=open       # every project, every spacetree
```

- Advantages: one query spans projects, useful for a global work view.
- Costs: forces the registry to carry Dolt connection info and stay fresh enough to trust; duplicates `bd`'s query surface; grows scope past addressing.

### Option C — live scan, no registry

```pseudocode
# resolve a bare name by walking known spacetrees on every run
```

- Advantages: no state to keep fresh; always current.
- Costs: slow across many spacetrees; still needs a root-discovery source, which is the known-spacetrees list anyway.

### Decision criteria

- Option A when the goal is addressing: the layer routes, `bd` answers.
- Option B only if a cross-project reporting need appears that per-project `wkg` calls cannot meet.
- Option C only while spacetree count stays tiny; the known-spacetrees list is already a cheap cache.

## Related artifacts

- `<spec:wk-beads-wrappers>` — the `wk`/`wkg`/`wkt` task frontend this spec routes through; owns the `wk` wrapper and its packaging.
- `<spec:wk-init>` — the project bootstrap that assigns each project's beads prefix.
- `HANDOFF-20260920-beads-shared-server-devtree.md` — an earlier design note on running one Dolt server per spacetree; the workspace placement it discussed is now owned by `<spec:beads-out-of-tree-storage>`.
- `README.md` — the spacetree definition, the human-first constraint, and the tooling inventory.
- `docs/beads-refs/` — the `bd` data model, how-to, and tag vocabulary, including the `issue_prefix` and `external:<project>:<capability>` mechanics this spec leans on.
- `<spec:spacetree-repo>` — the state roots under `<spacetree root>/.spacetree/`, the branch each is checked out from, and the machine-state root the project registry lands in.
- `EXPLO-*.md` in this directory — the pre-spec design record (resolver-core, registry, routing-vs-aggregate, bd-wrapper-interaction).
- `<spec:task-scopes>` — the task-scopes spec: the `:scope` model and how the third target segment is translated into `bd` label arguments.

## Global Open Questions

**Registry entries versus project info** — A registry entry records the project key and repository path, while the beads prefix lives in the project info.
Whether the registry should also cache the prefix, or always re-read the project info at the destination, is open.
Non-blocking.

**Spacetree name uniqueness** — Spacetree names now come from `spacetree.toml` in the spacetree repo rather than a directory name.
How a duplicate name is detected across the known-spacetrees list, and by whom, is open.
Non-blocking.

**Cross-project dependency references** — `external:<project>:<capability>` is forwarded verbatim today, and `bd` stores it as an edge whose target has no row in the local database.
Such an edge never blocks its issue: `bd ready` still lists it, and `bd dep tree` omits the edge.
Three things are open: the grammar (project keys are unique per spacetree only, so the ref currently has no spacetree axis), whether anything ever resolves the target, and what a rename or workspace deletion does to stored refs.
The candidate directions are to document the ref as an inert annotation and warn when an unresolved edge exists, to resolve it in `wk` against the target workspace, or to adopt `bd repo` hydration or `bd federation` — both of which change the one-workspace-per-project model owned by `<spec:beads-out-of-tree-storage>`.
Blocking for any use that expects a cross-project dependency to actually hold a task back.
Non-blocking for forwarding, which already works as specified.

**Terminology & Key Concepts** (TKC) — Whether this section is needed for this spec.
Non-blocking. Refer to spec-writing skill for guidance.
