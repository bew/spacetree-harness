# [READY] wk — Personal spacetree-aware work management frontend

## Introduction

This spec defines `wk`, the personal work frontend for the spacetree harness.
`wk` is the in-project form, `wkg` the cross-spacetree form, and `wkt` the in-spacetree form.

The trio renames the harness's in-project, cross-spacetree, and in-spacetree task commands to short work-named binaries.

The premise is that users should type task commands, not database commands.
Upstream beads `bd` is an implementation detail: it is not on `PATH` and is only ever reached through the trio.

`wk` runs the `wk` wrapper, which resolves the project info, sets `BEADS_DIR` to the project's external workspace, and applies the `:scope` rewrite before invoking the bundled `bd`.

This spec covers only what is specific to the frontend surface.

Routing target grammar and the `wkg`/`wkt` delegation mechanism are owned by `<spec:cross-spacetree-task-routing>`.
The `:scope` syntax and its translation table are owned by `<spec:task-scopes>`.
Neither is re-specified here.

## Naming & IDs

The frontend exposes three binaries:

- `wk` — work. The in-project form.
- `wkg` — work-global. The cross-spacetree form.
- `wkt` — work-spacetree. The in-spacetree form.

The names map one-to-one onto the harness task commands they replace: `wk` to the in-project command, `wkg` to the cross-spacetree command, and `wkt` to the in-spacetree command.
There are no aliases and no back-compatibility shims: no scripts or habits depend on the old names.

The `wk` wrapper is what `wk` runs.
It fronts the bundled upstream beads `bd`: it resolves the project info, points `BEADS_DIR` at the project's external workspace, and applies the `:scope` rewrite, then invokes `bd` with the resulting arguments.

Upstream `bd` is only ever reached through the `wk` wrapper.

The implementation directory is `wk-beads-wrappers/`.

## Interface / How to use

Run `wk` from inside a project for in-project tasks.
Run `wkt <target>` for tasks elsewhere in the same spacetree.
Run `wkg <target>` for tasks across spacetrees.

The `<target>` grammar and the `wkg`/`wkt` resolution-and-delegation mechanism are specified in `<spec:cross-spacetree-task-routing>`.
The `:scope` segment and its translation are specified in `<spec:task-scopes>`.

Both are referenced here, not restated.

Examples:

- `wk ready` — list ready tasks in the current project.
- `wk create --title="fix the thing" --type=bug` — create a task in the current project.
- `wkt ghh list` — list tasks in project `ghh` in the current spacetree.
- `wkg self:ghh ready` — list ready tasks in project `ghh` of spacetree `self`.
- `wk :auth list` — list tasks in scope `auth`.
- `wk init` — bootstrap the current project (see `<spec:wk-init>`).
- `wk commit` — snapshot the current project's task state and push it.

### Open Questions

**Other intercepted verbs** — Whether `wk` should intercept any verb besides `init` and `commit`.
Non-blocking. Every other verb is forwarded to the wrapped `bd` unchanged.

## Packaging & `bd` Bundling

The trio ships as a single Nix package that provides `wk`, `wkg`, and `wkt` on `PATH`.
Upstream beads `bd` is not exposed on `PATH`; it is bundled inside the same closure.

The user only ever types `wk`, `wkg`, or `wkt`.

The binaries are built from the Rust source in `wk-beads-wrappers/` and wired into `flake.nix`, alongside the bundled `bd`.
`wkg` and `wkt` resolve their target, change directory, and exec `wk` — a sibling binary from the same closure — forwarding the remaining arguments.

Exec'ing `wk` from the resolved directory is what lets it read the target project's project info and derive that project's workspace, so the same wrapper serves either spacetree.

## `wk init`

`wk init` intercepts the `init` verb to run the project bootstrap, replacing `bd init`.
It is one of two deliberate exceptions to passthrough: `init` and `commit` are not forwarded to the wrapped `bd`.

The workflow is owned by `<spec:wk-init>`: preconditions, prompts, flags, the project info step, workspace creation, failure behavior, and the manifest.
This spec owns only that the verb exists in the trio, that it intercepts `bd init` rather than forwarding it, and that it ships in the same closure as `bd`.

The old `beads-devtree-init` behavior — an in-repo `.beads/` workspace, force-adding its files, and undoing beads' auto-commit — is dropped.
The Rust rewrite replaces the tool before either reached a project; the superseded behaviors are recorded in the `wk init` spec.

## `wk commit` / `wkg commit`

`wk commit` snapshots the current project's task state onto the spacetree's state branch and pushes it.
`wkg commit` does the same for a project in another spacetree, resolved by the routing target grammar.

What is committed, when a push is refused, and the branch-protection check are owned by `<spec:beads-out-of-tree-storage>`.
This spec owns the verb names and their place in the trio, not the snapshot semantics.

### Open Questions

**Automated snapshots** — Whether a timer runs `wk commit` periodically, and at what cadence.
Non-blocking. Deferred; the manual verb is the whole surface for now.

## Spec Amendments

Linking this frontend into the existing specs required the following edits, now applied:

- `<spec:cross-spacetree-task-routing>`: renamed the user-facing entry points `bdg`/`bdt` to `wkg`/`wkt`, replaced the "exec `bd` from `PATH`" clause with exec'ing `wk`, addressed projects by their project key, and pointed its out-of-scope `bd`-wrapper item at this spec.
- `<spec:task-scopes>`: answered Global Open Question #1 as yes — the frontend supersedes the `:scope` wrapper syntax at the surface — and replaced the "a separate `wk` binary is out of scope" note, which was stale.
- `<spec:spacetree-daemons>`: the `.spacetree-root` marker is now a symlink to the config repo, and `.state/beads-by-project/` is the one versioned exception under `.state/`.
- `<spec:beads-out-of-tree-storage>`: workspaces live outside the repo on the config repo's state branch; this frontend no longer attaches a shared Dolt server, it sets `BEADS_DIR`.
- `<spec:project-info>`: the file the wrapper reads for the project key and beads prefix.
- `<spec:spacetree-config-repo>`: the marker symlink, config repo, and state branch the wrapper resolves through.
- `<spec:wk-init>`: extracted the `wk init` workflow; this spec keeps only the verb and its packaging.

The routing, scope, and storage semantics themselves are unchanged.

## Placement / Scope

The implementation lives in `wk-beads-wrappers/`, replacing the standalone `tooling/beads-devtree-init/` package.
It is written in Rust, departing from the Python used by the rest of the harness tooling.

In scope:

- The naming and surface of the `wk`/`wkg`/`wkt` trio.
- Bundling `bd` inside the trio's closure, off `PATH`.
- The `wk` wrapper: project-info resolution, `BEADS_DIR`, and the `:scope` rewrite.
- The intercepted verbs `init` and `commit`, and their packaging.

Out of scope:

- Routing target grammar and the `wkg`/`wkt` resolution-and-delegation mechanism — owned by `<spec:cross-spacetree-task-routing>`.
- `:scope` syntax and its translation table — owned by `<spec:task-scopes>`.
- The `wk init` workflow — owned by `<spec:wk-init>`.
- Workspace placement, the state branch, and commit semantics — owned by `<spec:beads-out-of-tree-storage>`.
- Upstream beads `bd` behavior, and cross-project aggregation.

## Alternatives & Tradeoffs

**Keep upstream `bd` on `PATH`, no trio.**
This is the simplest option: nothing to build, and users keep typing `bd` directly.
It gives up the `BEADS_DIR` wiring, the `:scope` rewrite, and hiding `bd` — exactly the wrapper value the trio exists to provide.
Rejected.

**Use stock `bd init`, no custom `init`.**
This defers initialization to beads' default.
`bd init` is not good enough by default: its defaults do not suit the harness, and it does not follow spacetree conventions.
That is exactly why `wk init` overrides it (see `<spec:wk-init>`).
Rejected.

**Rename only, keep the implementation in Python.**
This keeps the trio purely a rename of the existing Python tooling, with no rewrite.
It is less work than a Rust implementation and keeps the harness tooling in one language.
Rust is chosen deliberately for the frontend binaries; the tradeoff is a rewrite of `beads-devtree-init` and a second language in the tooling.
Accepted.

**Name the trio `wk`/`wkg`/`wkt`.**
`wk` abbreviates "work", a generic term with no overloaded `task`/`ticket` baggage in agent tooling or Jira.
Accepted; `t*` and `tk*` are the rejected alternatives below.

**Name the trio `t`/`tg`/`tt`.**
`t` is one letter, and `tg`/`tt` follow the old `bdg`/`bdt` suffix pattern.
`t` is overloaded: in agent tooling and trackers like Jira, `t`/`task`/`ticket` are ubiquitous, so the name reads ambiguously and invites collisions.
Rejected in favor of `wk`/`wkg`/`wkt`.

**Name the trio `tk`/`tkg`/`tkt`.**
`tk` reads as "task"/"ticket", making the family's intent explicit.
It carries the same overloaded `task`/`ticket` connotation, which is exactly what the naming is meant to avoid.
Rejected in favor of `wk`/`wkg`/`wkt`.

## Related artifacts

- `<spec:cross-spacetree-task-routing>` — target grammar and `wkg`/`wkt` delegation.
- `<spec:task-scopes>` — `:scope` syntax and translation.
- `<spec:wk-init>` — the `wk init` workflow this spec only intercepts.
- `<spec:beads-out-of-tree-storage>` — the external workspace `wk` points `bd` at, and the commit semantics `wk commit` exposes.
- `<spec:project-info>` — the project info the wrapper resolves on every invocation.
- `<spec:spacetree-config-repo>` — the marker symlink and config repo behind spacetree resolution.
- `SPEC-DRAFT-wk-beads-wrappers/EXPLO-frontend-cli.md` — the exploration that settled this design.
- `docs/beads-refs/` — the upstream beads reference.
- `README.md` — the harness overview and tooling conventions.
