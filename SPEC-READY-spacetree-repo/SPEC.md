# [READY] Spacetree Repo

## Introduction

A spacetree root is a non-git container: it gathers projects, docs, and tasks without versioning anything at the tree level.
Left at that, hierarchy, naming, per-project metadata, and the material every project under the tree shares have no home.
That is the "spacetree config file as the source of truth" gap in the README, and the reason spacetree-defined skills and tools for agents have nowhere to land.

The sharing problem is not hypothetical.
The opencode integration resolves spacetree-level agent instructions and skills for every project under the tree, so such material exists and matters, and it needs what any living artifact needs: a diff, a history, a review, and a clone on a new machine.
The same holds for tree-level docs, scripts, and templates a human wants every project to inherit.

This spec introduces a **spacetree repo**: one git repository per spacetree that holds everything the spacetree itself owns, configuration and inherited content alike.
The repo marker points at it — `.spacetree-repo` is a symlink whose target is the spacetree repo directory — so its location is a fact the registry records, never a name the harness assumes.

The repo splits along one line.
Configuration lives at its root: `spacetree.toml`, a `related-trees.toml`, and a `projects/` store.
Inherited content lives under a single `tree-shared/` namespace, reached from the tree through the shared marker, and laid out the way consumers already expect at the spacetree root.
Configuration is the versioned source of truth for the spacetree name, for which trees the spacetree relates to, and for project infos whose canonical copy lives outside a project repo (see `<spec:project-info>`).
`tree-shared/` is the versioned source of truth for everything a project inherits from its spacetree instead of declaring itself: agent instructions, skills, docs, scripts, and templates.

The registry holds only what is true of the spacetree on any machine: a key, a source, a role.
Where a tree actually lives here, whether it is checked out at all — those are machine state, and machine state gets a home of its own rather than a place in configuration.
The spacetree carries state roots at its top, told apart by what they are for: `.spacetree/shared-state/` for what every user of the tree must see, `.spacetree/machine-state/` for what is true of this machine alone, `.spacetree/runtime-state/` for scratch that is never worth a backup.
The shared and machine state roots are checkouts of their own branches, so each survives a clone on a new machine, and neither ever puts a machine path on a branch the human reviews.

The design serves the human first: the spacetree repo is an ordinary git repo the human edits, reviews, and commits.
The harness only reads it, and everything it enables stays usable without an agent.

## Terminology & Key Concepts

**spacetree repo** (new!):
The per-spacetree git repository holding everything the spacetree itself owns: configuration at its root, inherited content under `tree-shared/`.
It is the spacetree's meta repo — the one repo that describes and supplies the tree, while the trees it lists are the work done inside it.

**repo marker** (new!):
The `.spacetree-repo` symlink at the spacetree root whose target is the spacetree repo directory.
It is what makes a directory a spacetree root, and the only thing root detection walks up for.

**shared marker** (new!):
The `.spacetree-shared-root` symlink at the spacetree root whose target is the spacetree repo's `tree-shared/` directory.
It is a peer of the repo marker rather than a derivation each consumer re-checks: a consumer of inherited content resolves this path and stops.
Agreement between the markers is an invariant of the spacetree, verified by `spacetree status` rather than re-derived on every read.

**tree-shared** (new!):
The single directory inside the spacetree repo that holds inherited content, laid out the way consumers already expect at the spacetree root.

**inherited content** (new!):
Material a project under the spacetree receives from `tree-shared/` instead of declaring it itself: agent instructions, skills, docs, scripts, and templates.

**spacetree name** (new!):
The name declared in the spacetree repo's `spacetree.toml`; it is the name used on the `wkg` spacetree axis.

**related tree** (new!):
A directory the spacetree knows about and relates to: another project, a tooling checkout, or a docs tree.

**related-trees registry** (new!):
The versioned `related-trees.toml` at the spacetree repo root, listing the trees the spacetree relates to, with only the facts that hold on any machine.

**role** (new!):
How a related tree is consumed: `meta` for the spacetree's own repo, `project` for routing and workspaces, `tooling` for tool activation, `docs` for indexing.
The set is closed, and a value joins it only together with the consumer that reads it.

**project store** (new!):
The `projects/` directory at the root of the spacetree repo, holding one `<key>.toml` of long-form project info per project whose project info is not tracked in its own repo.

**shared state** (new!):
The `.spacetree/shared-state/` checkout at the spacetree root, holding the state every user of the spacetree shares: task state above all.

**machine state** (new!):
The `.spacetree/machine-state/` checkout at the spacetree root, holding what is true of this machine alone: where the related trees live.
It is versioned and backed up, on a branch of its own.

**machine state name** (new!):
The short label naming one machine's state, recorded in `.spacetree/machine-state-name` at the spacetree root and used to name the branch the machine state checkout comes from.

**runtime state** (new!):
The `.spacetree/runtime-state/` directory at the spacetree root, holding scratch that is never versioned, never backed up, and safe to delete.

## Naming & IDs

- **Repo marker**: `<spacetree root>/.spacetree-repo`, always a symlink to a directory, with a target stored relative to the directory holding it.
  It is neither a regular file nor a directory, and it is the only entry root detection walks up for.
- **Shared marker**: `<spacetree root>/.spacetree-shared-root`, always a symlink to a directory, sitting in the same directory as the repo marker, its target relative in the same way.
- **Config file**: `spacetree.toml` at the spacetree repo root.

  Minimal shape:
  ```toml
  [spacetree]
  name = "work"
  description = "Work trees for the greenhouse and platform projects"
  variant = "devtree"
  ```

  `description` sits right under `name` and is free prose: what this spacetree is for, short enough to read in one glance.
  It is for humans — a listing prints it beside the name — and nothing resolves differently because of it; `spacetree.toml` still owes a consumer only `name`.

  `variant` records the kind of spacetree and defaults to an unset value.
  It is descriptive metadata: no resolution changes because of it, and a consumer that starts branching on it is adding a consumer, which is a spec change.

- **Spacetree repo directory**: `<spacetree root>/spacetree-home/` by convention, taken from the `meta` entry's `default-path` of `$root/spacetree-home`.
  The repo itself is named after the spacetree, `spacetree-for-<domain>`; the local checkout name and the repo name are independent, and only the registry says where the checkout goes.
  The name is a human convention and never a resolution rule: the marker is what points at the repo, so a repo named anything else keeps working.
- **Shared namespace**: `<spacetree repo>/tree-shared/`, mirroring the tree-root layout consumers expect:
  `AGENTS.md`, `.agents/skills/`, `docs/`, `scripts/`, `templates/`.
  Those names are fixed; adding a content class means adding a name here first.
- **Related-trees registry**: `<spacetree repo>/related-trees.toml`, a single file listing every related tree.
  An entry carries a key (`[a-z0-9][a-z0-9-]*`), a role, an optional source in the `github:org/name` format (inspired from flake refs), an optional root-relative default path, and whether the tree is required.
  A tree that is not a repo has no source, `default-path` is optional on every entry, and a tree that is not required may also have no location.
- **Projects store**: `<spacetree repo>/projects/<key>.toml`, one file per project whose project info is not tracked in its own repo.
- **Shared state**: `<spacetree root>/.spacetree/shared-state/`, a checkout of `PROTECTED/spacetree-shared-state`.
  Task state lives below it, one directory per consumer, each owning its own layout.
- **Machine state**: `<spacetree root>/.spacetree/machine-state/`, a checkout of `PROTECTED/spacetree-state-for-<name>`.
- **Machine state name**: `<spacetree root>/.spacetree/machine-state-name`, a regular file holding one short label (`[a-z0-9][a-z0-9-]*`), unversioned.
  `spacetree init` picks it from `--machine-state <name>`, falling back to the OS login; later invocations read it rather than re-derive it, so a different login cannot silently map to a different branch.
- **Related-tree locations**: `<spacetree root>/.spacetree/machine-state/related-trees/<key>.toml`, one file per related tree, holding the facts the registry deliberately omits.
  A tree that is not required may have no file at all.
- **Runtime state**: `<spacetree root>/.spacetree/runtime-state/`, unversioned and never backed up.
- **Spacetree name**: `[a-z0-9][a-z0-9-]*`, unique across the known spacetrees.
- **Known spacetrees**: a global list recording spacetree root paths.
  The repo behind each root is reached through its marker, so the list never tracks a repo that moved on its own, and enumerating spacetrees from outside one stays a read of that list.
- **Branches**: the branches of the spacetree repo belong to the human, under whatever names the human uses; the harness owns only `PROTECTED/spacetree-shared-state` for shared state, and one `PROTECTED/spacetree-state-for-<name>` per machine state.
  The `PROTECTED/` prefix is a harness-wide convention rather than a local one: a `PROTECTED/` branch is written by the harness and must never be force-pushed, rewound, or deleted by tooling or by hand.
  It is a branch prefix and not a ref namespace, because forge protection covers branches and tags only (see *Branches & State Roots*).

Resolution is split by the question being asked, and a consumer takes the branch matching it.
Every path below is rooted at the spacetree root — the directory the walk-up found — and never at the spacetree repo, which lives wherever the marker points:

```pseudocode
# tree identity: which spacetree, which repo, what name
<spacetree root> := walk up from cwd until a `.spacetree-repo` entry is found
<spacetree repo> := target of <spacetree root>/.spacetree-repo
name             := <spacetree repo>/spacetree.toml:name

# inherited content: where the shared material is reachable
shared := target of <spacetree root>/.spacetree-shared-root

# machine state: which branch this machine backs up onto
label  := contents of <spacetree root>/.spacetree/machine-state-name
branch := `PROTECTED/spacetree-state-for-` + label
```

A consumer of inherited content never reads `spacetree.toml` and never constructs `tree-shared/` itself; it resolves the shared marker and stops.
Agreement between the markers is an invariant of the spacetree rather than a check on every read: `spacetree status` reports markers that disagree.

Resolution fails explicitly, never by fallback, when:

- no `.spacetree-repo` entry exists above the cwd — the directory is not in a spacetree;
- the repo marker is not a symlink, or its target does not exist;
- `spacetree.toml` is missing, or it has no `name`;
- the shared marker is missing, is not a symlink, or its target does not exist;
- the shared marker is not in the same directory as the repo marker;
- the shared marker's target is not `<repo>/tree-shared`;
- a consumer of machine state finds `.spacetree/machine-state-name` missing, or holding something that is not a short label.

The markers are unconditional: `spacetree init` creates them, and `tree-shared/` exists even when it holds nothing yet, so absence always means a broken binding rather than "nothing to share".

The spacetree root path is the directory holding the markers, not the spacetree repo path.

## Interface / How to use

`spacetree root --json` resolves every path the model defines, plus the name, in one call, so no consumer re-derives any of them:

```json
{
  "name": "work",
  "path": "/Users/…/work-stuff",
  "spacetree_repo": "/Users/…/work-stuff/spacetree-home",
  "shared_root": "/Users/…/work-stuff/spacetree-home/tree-shared",
  "machine_state_name": "foo"
}
```

`path` is the spacetree root — the directory holding the markers — and every other path in the answer hangs off it.

`spacetree status` is the one place that verifies the bindings the rest of the design treats as invariants, and it exits non-zero when any is broken:

- the markers exist, are symlinks, and agree, with `shared_root` exactly `<spacetree repo>/tree-shared`;
- `.spacetree/machine-state-name` exists and holds a short label;
- `.spacetree/shared-state/` is checked out on `PROTECTED/spacetree-shared-state`, and `.spacetree/machine-state/` on `PROTECTED/spacetree-state-for-<name>`;
- every required registry entry has a location on this machine, and no entry names a path that is gone;
- each `PROTECTED/` branch that exists is protected against force-push and deletion on its forge, unless `--offline` says the network is off limits.

That last check is the reason protection appears twice in this design: the prefix states the intent, the forge enforces it, and nothing else would notice a rule that was never created.
`spacetree init` cannot create the rule itself, so it prints the recipe when it creates a `PROTECTED/` branch and leaves the forge untouched.

`spacetree status` is a subcommand of the same command that answers `spacetree root`, so one entry point both resolves and verifies.

Verification stops there: a consumer resolves its path and proceeds, rather than re-running the checks on every read.

The human side needs no interface of its own: the spacetree repo is an ordinary git repo, edited and committed with ordinary git on the branches the human uses.

Nothing in the harness commits, stages, or creates history on a branch the human owns.
The harness authors commits only on `PROTECTED/` branches — task state and machine-state backups — never on the branch the human reviews (see *Lifecycle & State*).

`spacetree init` is the only command that creates the markers, the state branches, and their checkouts (see *Branches & State Roots*).

## Related Trees Registry

The registry is what makes a spacetree more than a directory: it names the trees the spacetree works with, so tooling enumerates them without scanning the filesystem, and so a second machine rebuilds the same set from a clone.
It is a single versioned file, `related-trees.toml`, at the spacetree repo root.

Keys are table names rather than a repeated field, which makes TOML itself reject a duplicate:

```toml
[trees.self]
role = "meta"
source = "github:acme/spacetree-for-work"
default-path = "$root/spacetree-home"

[trees.greenhouse-hub]
role = "project"
source = "github:acme/greenhouse-hub"

[trees.team-tooling]
role = "tooling"
source = "github:acme/spacetree-tooling"
required = true

[trees.team-docs]
role = "docs"
```

`source` is a clone address, never a location: its scheme is what kind of source it is, so `github:org/name` names a repo on a forge (syntax inspired from flake refs).
A tree with no repo of its own has no source at all.
The scheme decides how a source is read, so an unrecognized scheme is a failure rather than a guess.

`default-path` is optional on every entry, and many entries carry none: it is where the tree is expected to sit.
The value is written with a `$root/` prefix, so the path itself says what it is relative to and a reader never has to guess whether an anchor is missing.
Being relative is the point, since the same registry then describes the layout on every machine.
It stays inside the root: a `..` segment is rejected, because the path is a position within the spacetree by definition, and an absolute path is rejected for the same reason.

An entry may carry neither a source nor a default path: a tree that the harness neither clones into place nor expects at a fixed location, and that is still a member worth naming for routing or indexing.

`required` defaults to false and states that the spacetree does not work without this tree.

Roles come from a closed set, and each exists because something reads it:

- `meta` — the spacetree repo itself: the entry that makes the owner of the registry one of the trees it names.
  It is located like any other tree, with a source to clone from and a default path to clone into.
- `project` — routed to and given workspaces; its key is also the project axis identity, the same key used in `wkg <spacetree>:<key>:<scope>` and in task-state directories.
- `tooling` — contributes to tool activation.
- `docs` — indexed for lookup.

The `meta` entry for the spacetree repo is keyed `self`, a fixed key that says nothing about the spacetree name: the registry can name its own owner without depending on what that owner is called.

Every entry's key names one file on the machine side, except `meta`, whose location the marker already holds.
Only `project` entries use the key as an identity beyond naming.

The machine half is one small file per tree, versioned on that machine's own `PROTECTED/` branch and never on a branch the human owns:

```toml
# <spacetree root>/.spacetree/machine-state/related-trees/team-docs.toml
path = "/Users/…/src/team-docs"
```

`path` is where that tree is checked out on this machine, and it overrides `default-path`.
A tree that is not required may carry no file and no `default-path`, and its absence is not an error: an optional tree is simply not present here.
A required tree with neither is a failure that names the tree and says what to run to locate it, because a requirement the machine cannot satisfy is broken configuration rather than an optional absence.

The registry and the machine state answer different questions and never restate each other: the registry holds no machine path, only a root-relative default, and the machine state never restates a role.
A location file whose key has no registry entry is stale, and `spacetree status` reports it.
A location file for the `meta` entry is stale for the same reason: the marker already answers where the repo is, and a second answer could contradict it.
`spacetree status` also compares the marker's target against the `meta` entry's `default-path`, so a checkout that drifted from the registry is reported rather than silently accepted.

## Inherited Content

The tree root is not a repo, so anything shared across the whole tree — agent instructions, skills, notes, scripts, templates — has nowhere versioned to live.
It would otherwise be duplicated per project or kept outside git entirely.
`tree-shared/` is that home: one directory in the spacetree repo whose content every member of the tree inherits.

```
<spacetree repo>/tree-shared/
├── AGENTS.md
├── .agents/skills/
├── docs/
├── scripts/
└── ...
```

The layout mirrors the tree-root layout on purpose.
Inheritance is then the boring operation of the same path one layer up, and a consumer needs no per-name mapping to know where shared docs are.

The names are fixed by the harness, not read from configuration.
`docs/` in the shared root means what `docs/` means at the tree root, and a new name joins the set only by changing this spec, together with the consumer that reads it.
That is what keeps inheritance a vocabulary rather than a second configuration file.

Inheritance is a privilege of membership: only trees and repos registered in the related-trees registry inherit shared content.
A repo the spacetree does not list is a stranger — it may sit on the same disk, but it follows no shared marker and claims nothing from the shared vocabulary.
Registration is what makes a tree part of the spacetree, so the registry is a membership list, not merely an index.

Nothing resolves shared content by reading `spacetree.toml`.
Consumers follow `<spacetree root>/.spacetree-shared-root`, and the marker alone says where shared content is.
A missing or broken shared marker is a failure, never a fallback to the tree root, because the tree root does not hold these files — falling back would silently hand a project an empty instruction set.

The tree root itself holds no agent files.
There is one shared copy, reached through the marker, and no second copy at the root to drift from it.

Projects are the exception, because a project is its own repo and its own files are ordinary project content.
A project that ships its own `AGENTS.md` or `.agents/skills/` is read on top of the inherited content:

- instructions: each is read, inherited first and the project's own after it, since later text refines earlier text.
- skills: the project's skill directory comes first in the search path, so a skill of the same name shadows the inherited one.
- `docs/`, `scripts/`, …: the project-local path wins, and the shared path is inherited or used as a fallback.

Outside those names, inherited content is simply present: a member of the tree sees it without declaring anything of its own.

`tree-shared/` is versioned in a repo that may be shared or public, so nothing secret belongs in it — not tokens, not private hostnames, not credentials.
Secrets stay in the environment, never in the spacetree repo.

## Placement / Scope

- The spacetree repo lives inside the spacetree root, at the `default-path` its `meta` entry carries — conventionally `$root/spacetree-home`.
  Being inside the root is what lets its location be a root-relative path at all, and it keeps the root self-describing: a directory that has the markers also has the repo they point at.
- The name is not fixed: the registry says where, and a spacetree that puts it elsewhere keeps working.
  The convention is `spacetree-home`, while the repo itself is named after the spacetree, `spacetree-for-<domain>`.
- A spacetree repo is never nested inside a project repo, and never inside another related tree.
- The spacetree root stays a plain directory: the markers are symlinks, and nothing else at the root is versioned.
- The state roots sit at the spacetree root beside the markers, never inside the repo and never on a branch the human owns (see *Branches & State Roots*).
- The process tree definition, `process-compose.yaml`, lives at the spacetree repo root, versioned with configuration so it survives a clone instead of being re-created per machine.
- Projects, their git repos, and their `.envrc`/flake setup are unaffected.

In scope:

- The markers and the invariant that keeps them in agreement.
- `spacetree.toml`: the `name` that tools resolve, plus the descriptive `description` and `variant`.
- The location and ownership of `projects/`.
- `related-trees.toml`, its role vocabulary, and the machine half that answers where a tree lives here.
- `tree-shared/` and the inherited content it supplies.
- The branch layout, the `PROTECTED/` convention, the state roots, and the machine-state name.
- `spacetree init` as the only command that creates markers, state branches, and their checkouts.
- The `spacetree root --json` shape and the `spacetree status` verification.

Out of scope:

- The project info schema and its resolution (owned by <spec:project-info>).
- What task state a workspace holds, and how it is committed (owned by <spec:beads-out-of-tree-storage>).
- Which daemons run against these roots, how they are supervised, and what the process tree contains; only where its definition lives is decided here (owned by <spec:spacetree-daemons>).
- Other spacetree-wide settings.
- The known-spacetrees list beyond its storing spacetree root paths (owned by <spec:cross-spacetree-task-routing>).
- Another spacetree as a related tree: the registry reserves no role for it for now.
- Extra checkouts of a related tree beyond its one location here.
- Content the shared tree holds beyond the fixed names: a new name needs its consumer first.

## Branches & State Roots

The spacetree repo holds content of kinds that must never mix in history: configuration, and state.
Branches are how the repo keeps them apart without a second repo.

```
<spacetree repo>/                    # the one repo, one clone
├── spacetree.toml                   # a branch the human owns
├── related-trees.toml
├── projects/<key>.toml
├── tree-shared/
└── .git/                            # worktree bookkeeping for the branches below

<spacetree root>/
├── .spacetree-repo -> spacetree-home/        # marker
├── .spacetree-shared-root -> spacetree-home/tree-shared/   # marker
├── spacetree-home/                            # the repo, at the registry's default path
└── .spacetree/                                # plumbing, the only hidden directory here
    ├── machine-state-name                     # names this machine's branch
    ├── shared-state/                          # worktree of PROTECTED/spacetree-shared-state
    │   └── beads-by-project/<key>/issues.jsonl
    ├── machine-state/                         # worktree of PROTECTED/spacetree-state-for-<name>
    └── runtime-state/                         # no branch, no backup
```

The markers stay at the spacetree root, because a marker is what root detection walks up for and what a consumer resolves without descending anywhere.
Everything that is plumbing rather than content lives under `.spacetree/`, and being inside a directory that is already hidden is why its entries carry no dot of their own.

Which branch the human works on is the human's business; this spec fixes no name for it.

The branches the harness owns sit under a `PROTECTED/` prefix, which is a branch prefix and not a ref namespace.
Forge protection covers branches and tags only, so a `refs/protected/...` ref could not be protected at all, while `PROTECTED/spacetree-shared-state` is an ordinary branch a protection rule can name exactly.
The prefix states the intent and the operator's rule enforces it: a `PROTECTED/` branch is written by the harness and must never be force-pushed, rewound, or deleted, by tooling or by hand.

The shared and machine state roots are git worktrees of those branches, which is why a fresh clone on another machine recovers the state without copying a directory around.
`.spacetree/runtime-state/` is not one of them: it holds sockets and scratch that are never worth a backup, so it has no branch and is safe to delete while nothing is running.

Each machine needs to know which machine state is its own, so `.spacetree/machine-state-name` holds a short label (`[a-z0-9][a-z0-9-]*`) as a plain unversioned file.
`spacetree init` picks it from `--machine-state <name>`, falling back to the OS login; later invocations read the file rather than re-derive it, so a changed login cannot silently map to a different branch.

`spacetree init` creates the state branches and their checkouts as part of scaffolding the spacetree; no other command creates either, and the manual equivalent is one worktree add per root:

```sh
git -C <spacetree repo> worktree add <spacetree root>/.shared-state PROTECTED/spacetree-shared-state
git -C <spacetree repo> worktree add <spacetree root>/.machine-state PROTECTED/spacetree-state-for-<name>
```

When a state branch does not exist yet, init creates it from an empty root commit, because creating an absent branch is not a rewrite of existing history.
A command that needs the checkout never creates it: a spacetree without one is a precondition failure that points at `spacetree init`.

Removing a worktree removes the checkout, never the branch.
Deleting a state root without `git worktree remove` leaves stale bookkeeping in the spacetree repo, which `git worktree prune` clears.

A state root never lives inside the repo, so the repo's working tree can always be committed without dragging state along.

## Lifecycle & State

The spacetree repo is versioned and human-owned: changes to configuration are committed by the human, reviewed as ordinary diffs.
The harness reads it on every spacetree-aware invocation and writes nothing to a branch the human owns, except where a command explicitly creates a project info in the store (see <spec:project-info>).

Task state commits go to `PROTECTED/spacetree-shared-state` only, through `wk commit` (see <spec:beads-out-of-tree-storage>).
Machine state follows the same rule on `PROTECTED/spacetree-state-for-<name>`, written when a command records where a related tree lives.
Those branches are touched once more by `spacetree init`, when it creates them and checks out their state roots; nothing else in the harness creates or rewrites them.

Bootstrap happens once per spacetree.
`spacetree init` writes `spacetree.toml` when it is absent, creates the markers, records the machine-state label, creates any missing state branch from an empty root commit, and checks out the state roots.
It never rewrites configuration that already exists, so running it again is harmless and is how an existing checkout is repaired.

Recovery on another machine is the same command over a fresh clone: clone the repo to the registry's `default-path`, then run `spacetree init --machine-state <name>`, which finds the existing branches instead of making new ones.

A missing or dangling marker means the directory is not a spacetree, or the spacetree is broken.
Tools error in either case and never fall back to the directory basename, so a renamed directory cannot silently rename a spacetree.

Moving the spacetree directory keeps the markers and therefore the binding to the repo intact.
Moving or renaming the spacetree repo breaks every marker that points at it, because each marker stores its target relative to the root.
Moving the whole root keeps the markers and the repo together, which is the move that has to work and the reason the targets are relative rather than absolute.

Removing a spacetree means removing the markers and the state roots, then the repo; the branches keep the history of state that was there, which is the point of versioning it.

## Alternatives & Tradeoffs

**Chosen — one repo per spacetree, reached through a symlink marker, with state on `PROTECTED/` branches.**
One clone carries everything the spacetree decides, one protection rule covers every branch the harness writes, and a consumer resolves a path without parsing anything.

**No repo at all, the root as it stands.**
Every spacetree keeps its configuration, skills, and docs loose in the tree, which is where unversioned agent content already lives.
Nothing is shared across machines, no change has history or review, and a second spacetree re-invents what the first one learned.

**The spacetree root itself becomes the git repo.**
No pointer to resolve at all, but the root holds other projects' checkouts, so the repo would track or ignore them by accident, and it contradicts the definition of a spacetree as a container that is not a repo.

**A TOML marker holding the repo path.**
Explicit and greppable, at the cost of every consumer parsing the marker instead of resolving it, and of a path that can drift from the marker's own state.
A symlink either resolves or does not, which is the property the resolution rules depend on.

**A fixed directory at the spacetree root.**
Zero ambiguity and no symlink semantics, but the repo cannot live anywhere the human wants it, and the marker stops being separable from the thing it points at.

**A repo per concern: configuration in one, task state in another.**
Clean separation of history without branch bookkeeping, at the cost of a set of repos per spacetree to create, clone, and protect, and of truth split across them.
A branch per concern keeps one clone, one protection setup, and one place to look.

**A marker that points only at the repo, with the shared tree found through `spacetree.toml`.**
Fewer markers, but every content consumer would parse configuration to find skills and instructions, and the shared root would be re-derived per read.
The shared marker exists so that content resolution stops at the marker and never opens configuration.

**Decision criteria**: a second repo earns its keep when state must be readable by people who must not see configuration, or when its retention differs sharply; otherwise one repo with `PROTECTED/` branches keeps the spacetree to a single clone.

## Related artifacts

- `README.md` — spacetree definition, the non-git root, and the gaps this spec closes: configuration as source of truth, and spacetree-defined skills and tools for agents.
- `<spec:project-info>` — the project info format and the store this spec locates.
- `<spec:beads-out-of-tree-storage>` — the task state that lives under the shared state root, and the workspaces inside it.
- `<spec:wk-init>` — the command that creates a workspace in shared state, and therefore requires `spacetree init` to have created the checkout.
- `<spec:spacetree-daemons>` — runtime state, the sockets under it, and the process tree.
- `<spec:cross-spacetree-task-routing>` — spacetree root detection, spacetree names, and the known-spacetrees list.
- `<spec:spacetree-rename>` — renaming a spacetree, which is constrained by the markers and the machine-state label.
- `tooling/spacetree/spacetree` — the existing `spacetree root --json` implementation.
- `integrations/opencode-plugin/spacetree-context.ts` — the agent-side consumer that injects inherited skills and instructions.
- `docs/what-is-a-spacetree.md` — the human-facing concept doc this spec's vocabulary has to stay compatible with.
