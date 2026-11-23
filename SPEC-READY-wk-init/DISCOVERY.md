# wk init

## Name

`wk init` — the harness bootstrap command run from inside a project.

## Problem

`wk init` is currently described in pieces across four specs and one legacy Python tool.
No single spec owns the command: the bootstrap sequence lives in the `wk` frontend spec,
the project-info creation workflow in the project-info spec, the beads workspace creation in the
beads out-of-tree storage spec, and the legacy in-repo behavior in `tooling/beads-devtree-init/`.
The pieces also disagree: the legacy tool force-tracks `.beads/` files in the repo and undoes
beads' auto-commit, while the out-of-tree spec settles on an external `BEADS_DIR` workspace with
no tracked beads files and no commit to undo.

For: a human running one command to make a project addressable in the spacetree, and agents told to
bootstrap a project without reading four specs to learn the order of steps.

## Inspirations

- `<spec:wk-beads-wrappers>` — owns `wk`, the intercepted `init` verb, and the Rust
  home of the implementation.
- `<spec:project-info>` — project info file, key grammar, ownership detection, the
  `wk init` ownership prompt.
- `<spec:beads-out-of-tree-storage>` and `DISCOVERY.md` — external `BEADS_DIR`
  workspace under `<spacetree root>/.spacetree/shared-state/beads-by-project/<key>/`, verified against bd 1.3.0.
- `<spec:spacetree-repo>` — the spacetree repo holding the project info store.
- `docs/beads-init-complaints.md` — why stock `bd init` is overridden at all.
- `tooling/beads-devtree-init/beads-devtree-init` — the legacy wrapper whose behavior is being
  reimplemented and partly superseded.

## Mention inventory

Every current mention of `wk init` or `beads-devtree-init`:

- `<spec:wk-beads-wrappers>`: `## wk init` section (lines ~77-93), the interface
  example, scope list, and an alternative.
- `<spec:project-info>`: `wk init` project-info creation workflow, ownership
  detection, `--owner` flags, uniqueness check, scope list.
- `SPEC-DRAFT-project-info/DISCOVERY.md`: the tracked-vs-linked prompt.
- `<spec:beads-out-of-tree-storage>`: scope line and an interface TODO.
- `SPEC-DRAFT-beads-out-of-tree-storage/DISCOVERY.md`: `wk init` runs `bd init` with `BEADS_DIR`.
- `<spec:cross-spacetree-task-routing>`: delegates `wk init` ownership to the `wk` spec.
- `SPEC-READY-cross-spacetree-task-routing/EXPLO-resolver-core.md`: says `beads-devtree-init`
  assigns the issue prefix.
- `README.md`: harness package list and the `repo-init` gap.
- `docs/beads-init-complaints.md`: the whole doc is framed around `beads-devtree-init`.
- `tooling/beads-devtree-init/`: script, Nix package, bats suite, fixtures (real files, not prose).

## Constraints stated by the user

- The new spec owns the whole orchestration: project-info creation, beads workspace creation, and
  skills install; the concern specs point at it.
- Beads storage follows the settled out-of-tree `BEADS_DIR` model, not the legacy in-repo tracking.
- Every prose mention of `beads-devtree-init` across specs, README, and docs redirects to the new
  spec. The tooling directory keeps its name for now; renaming it is planned but deferred.
- Spec references use the `<spec:$slug>` syntax, which is not yet documented in the write-spec
  skill; a task capture records that gap.
- The new spec references the `wk` frontend spec (`wk-beads-wrappers`) as a dependency.

## Settled decisions

- Slug: `wk-init`.
- The new spec is the owner of the `wk init` command sequence; the `wk` frontend spec keeps owning
  the binary family and the interception of the `init` verb, and points here for the command itself.
- The project-info spec keeps owning the file format, key grammar, and resolution; its `wk init`
  creation workflow moves here.
- The beads out-of-tree storage spec keeps owning workspace placement and contents; its `wk init`
  creation step moves here.
- `beads-devtree-init` is legacy: its behavior is rewritten as Rust inside `wk`, and the parts that
  contradict out-of-tree storage are superseded rather than carried forward.

## Open threads

- The exact step order and failure/rollback behavior of a partially completed `wk init`.
- Whether `wk init` is idempotent, resumable, or refuses once any of its outputs exist.
- How `wk init` behaves without a spacetree marker, without a spacetree repo, or outside a git repo.
- Which steps stay interactive and which require flags in a non-interactive run.
- Whether the tooling package rename happens in the same milestone as the Rust rewrite.

## Project-global docs

Candidate set: `README.md`, `docs/beads-init-complaints.md`, `docs/beads-refs/`.
Terms already defined there: spacetree, `.spacetree-repo` marker, human-first constraint, stock
`bd init` complaints. The spec references these rather than restating them.
