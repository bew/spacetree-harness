# [DRAFT] Spacetree Config Repo

> IMPORTANT: Before any drafting/planning/editing of this spec,
> agents MUST load one of the spec-writing skills first.

## Introduction

A spacetree root is currently a non-git container whose only spacetree-wide artifact is an empty `.spacetree-root` marker file (see `README.md`).
Nothing at the spacetree level is versioned: hierarchy, naming, and per-project metadata have no home, which is the "spacetree config file as the source of truth" gap in the README.

This spec introduces a **config repo**: one git repository per spacetree that holds spacetree-wide configuration.
The marker becomes the pointer to it — `.spacetree-root` is a symlink whose target is the config repo directory — so the repo may live inside or outside the spacetree.

The config repo is the versioned source of truth for the spacetree name and for project infos whose canonical copy lives outside a project repo (see the project-info spec).
It is also the future home of other spacetree-wide settings, but those are out of scope here.

The design serves the human first: the config is an ordinary git repo the human edits, reviews, and commits.
The harness only reads it.

## Terminology & Key Concepts

**config repo** (new!):
The per-spacetree git repository holding spacetree-wide configuration.

**marker symlink** (new!):
The `.spacetree-root` symlink whose target is the config repo directory.

**spacetree name** (new!):
The name declared in the config repo's `spacetree.toml`; it is the name used on the `wkg` spacetree axis.

**project info store** (new!):
The `project-infos/` directory inside the config repo, holding one `<key>.toml` per project whose project info is not tracked in its own repo.

**state branch** (new!):
The fixed branch `beads-devtree-init` of the config repo, holding task state instead of configuration.

**state worktree** (new!):
The checkout of the state branch at `<spacetree>/.state/beads-by-project/`, created by `spacetree init` when it scaffolds the spacetree.

## Naming & IDs

- **Marker**: `<spacetree root>/.spacetree-root`, always a symlink to a directory.
  It is not a regular file and not a directory.
- **Config file**: `spacetree.toml` at the config repo root.
  Minimal shape:

  ```toml
  name = "self"
  ```

- **Project info store**: `<config repo>/project-infos/<key>.toml`, one file per key.
- **Spacetree name**: `[a-z0-9][a-z0-9-]*`, unique across the known spacetrees (see *Global Open Questions*).
- **Branches**: the config repo carries its own configuration branch under whatever name the human uses, plus the fixed state branch `beads-devtree-init`.

Root resolution, used by every spacetree-aware tool:

```pseudocode
root        := walk up from cwd until a `.spacetree-root` entry is found
config-repo := absolute target of that symlink
name        := spacetree.toml:name
```

Resolution fails explicitly, never by fallback, when:

- the entry is not a symlink;
- the symlink target does not exist;
- `spacetree.toml` is missing;
- `spacetree.toml` has no `name`.

The spacetree root path is the directory containing the marker, not the config repo path.

## Interface / How to use

`spacetree root --json` keeps reporting the root and now also reports the config repo and the declared name.

```json
{"name": "self", "path": "/Users/…/self", "config_repo": "/Users/…/self-config"}
```

The human workflow is plain git in the config repo:

```sh
cd ~/self-config
$EDITOR spacetree.toml
git add spacetree.toml && git commit -m "name the spacetree"
```

The harness never commits, stages, or creates the config repo's history (see *Lifecycle & State*).
Any tool that needs spacetree-wide data resolves the root, then reads the config repo from the marker.

## Placement / Scope

- The config repo may live anywhere.
  A sibling of the spacetree root is the expected case, e.g. `~/self-config` for `~/self`.
- Inside the spacetree root is also valid, provided it is a hidden or clearly non-project directory (e.g. `<spacetree>/.spacetree-config`), so it is never mistaken for a project.
- A config repo must never be nested inside a project repo.
- Machine-local runtime state stays under `<spacetree>/.state/` and is never committed; the one exception is the state worktree, which is a checkout of this repo (see *Branches & State Worktree*).
- Projects, their git repos, and their `.envrc`/flake setup are unaffected.

In scope:

- The marker symlink and the root/config-repo resolution contract.
- `spacetree.toml` and the `name` field.
- The location and ownership of `project-infos/`.
- The branch layout: one configuration branch plus the state branch, and the state worktree that checks it out.
- The `spacetree root --json` shape.

Out of scope:

- The project info schema and its resolution (owned by the project-info spec).
- What task state a workspace holds, and how it is committed (owned by the beads out-of-tree storage spec).
- Other spacetree-wide settings, including the process tree.
- The known-spacetrees list contract, except for the open question below.

## Branches & State Worktree

The config repo holds two kinds of content that must never mix in history: configuration, and task state.

```text
<config repo>/
├── spacetree.toml                 # configuration branch
├── project-infos/<key>.toml
└── .git/                        # worktree bookkeeping for the state branch

<spacetree>/.state/beads-by-project/          # state worktree: branch beads-devtree-init
└── <project key>/issues.jsonl   # task state, one directory per project
```

The configuration branch is whatever branch the human works on; this spec fixes no name for it.
The state branch name is fixed so a forge protection rule and the harness can both name it exactly, and it is protected against force-push and deletion by the operator.

`spacetree init` creates the state worktree, and the state branch when absent, as part of scaffolding the spacetree; no other command creates either.
When the state branch does not exist yet, init creates it from an empty root commit, because creating an absent branch is not a rewrite of existing history.
`wk init` never creates the checkout: a spacetree without one is a precondition failure that points at `spacetree init`.
The manual equivalent of what init runs:

```sh
git -C ~/self-config worktree add ~/self/.state/beads-by-project beads-devtree-init
```

Removing a worktree removes the checkout, never the branch.
Deleting `<spacetree>/.state/beads-by-project/` without `git worktree remove` leaves stale bookkeeping in the config repo, which `git worktree prune` clears.

## Lifecycle & State

The config repo is versioned and human-owned: changes to configuration are committed by the human, reviewed as ordinary diffs.
The harness reads it on every spacetree-aware invocation and writes nothing to the configuration branch, except where a command explicitly creates a project info in the store (see the project-info spec).

Task commits are the one recurring harness write, and they go to the state branch only, through `wk commit` (see the beads out-of-tree storage spec).
The same branch is touched once more when `spacetree init` creates it and checks out the state worktree; nothing else in the harness writes it.

A missing or dangling marker means the directory is not a spacetree, or the spacetree is broken.
Tools error in both cases and never fall back to the directory basename, so a renamed directory cannot silently rename a spacetree.

Moving the spacetree directory keeps the marker and therefore the config binding intact.
Moving or renaming the config repo breaks every marker that points at it; whether the symlink target is absolute or relative decides how much breaks (see *Global Open Questions*).

## Alternatives & Tradeoffs

**Chosen — symlink marker to a per-spacetree config repo.**
One hop from marker to repo, no parsing needed to locate the config, and the repo can live anywhere including outside the tree.

**TOML marker file with a `config_repo` path.**
Explicit and greppable, but every consumer must parse the marker instead of testing existence, and the marker and config are two separate things to keep in sync.

**Fixed directory at the spacetree root (`<spacetree>/.spacetree-config/`).**
Zero ambiguity and no symlink semantics, but the config cannot be relocated, and it lives inside a tree that is explicitly not a repo.

**The spacetree root itself becomes the git repo.**
No pointer at all, but it contradicts the README's "the spacetree root is not itself a repo", nests project repos inside a tracked worktree, and invites committing project contents by accident.

**A second repo for task state.**
One repo for configuration and another for task state gives clean separation of history without branch bookkeeping.
It also means a spacetree owns two repos to create, clone, and protect, and that its truth is split across two places; a second branch in the existing repo keeps one clone and one protection setup.

**Decision criteria**: use a second repo when task state must be readable by people who should not see configuration, or when its retention differs sharply; use the chosen single-repo, two-branch design when the same human owns both and one clone should carry everything.

## Related artifacts

- `README.md` — spacetree definition, the non-git root, the "spacetree config file as source of truth" gap.
- The project-info spec — `project-info.toml` and the project info store this spec locates.
- The beads out-of-tree storage spec — the state branch and state worktree this spec hosts, and the workspaces inside them.
- `<spec:wk-init>` — the command that creates a workspace inside the state worktree, and therefore requires `spacetree init` to have created the checkout.
- `<spec:spacetree-daemons>` — the `.state/` convention and the `spacetree init` command that now also creates the state worktree.
- `<spec:cross-spacetree-task-routing>` — spacetree root detection, spacetree names, the known-spacetrees list.
- `tooling/spacetree/spacetree` — the existing `spacetree root --json` implementation.
- `integrations/opencode-plugin/spacetree-context.ts` — the agent-side marker resolution.

## Global Open Questions

**Default location** — Whether the recommended config repo path is a sibling (`~/self-config`), a hidden dir inside the root (`~/self/.spacetree-config`), or chosen freely at init.
Non-blocking.

**Symlink target form** — Whether `.spacetree-root` stores an absolute or a relative target.
Absolute survives moving the spacetree but breaks when the config repo moves; relative is the reverse.
Non-blocking.

**Scaffolding ownership** — Whether `spacetree init` creates the config repo, runs `git init`, writes `spacetree.toml`, and makes the first commit, or whether the symlink and repo are created by hand.
Non-blocking.

**Known-spacetrees list** — The global list under `$XDG_STATE_HOME/spacetree-global/` still has to enumerate spacetrees from outside one.
Whether it should now store config repo paths instead of spacetree paths, and whether it can be derived from markers, is open.
Non-blocking.

**Process tree ownership** — Whether `process-compose.yaml` moves from the spacetree root into this config repo, which would make it versioned.
Non-blocking; the daemons spec keeps it at the root for now.

**Protection bootstrap** — Whether creating the state branch also enables forge protection, or whether the operator does it by hand from a documented recipe.
Non-blocking; the requirement is that protection exists before the first push, not who enables it.

**Variant field** — Whether `spacetree.toml` carries a `variant` (or `type`) key recording that a spacetree is a devtree, the code-oriented case.
As floated, it would be descriptive metadata only: no tooling branches on it and it enables nothing.
Whether the key should exist at all, and which values it accepts beyond `devtree`, stays open.
Non-blocking; the spacetree rename spec (`<spec:spacetree-rename>`) states the distinction is vocabulary-only until a consumer needs it.

**Terminology & Key Concepts** (TKC) — Whether this section is needed for this spec.
Non-blocking. Refer to spec-writing skill for guidance.
