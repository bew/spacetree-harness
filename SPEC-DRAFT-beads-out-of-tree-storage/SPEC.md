# [DRAFT] Beads Out-of-tree Storage

> IMPORTANT: Before any drafting/planning/editing of this spec,
> agents MUST load one of the spec-writing skills first.

## Introduction

beads is the task tracker behind this harness, and upstream `bd` assumes a `.beads/` workspace at the root of the repo it runs in.
That assumption forces every project to carry tracker state in its own history — either as tracked files that pollute diffs, or as stealth/gitignore tricks that hide them.
It also duplicates state across git worktrees and ties task data to the life of a clone.

The harness wants the setup, the files, and the storage outside the repo.
Moving only the database data directory would leave the workspace — config, metadata, README — inside the repo.
This spec moves the entire workspace instead: beads 1.3 accepts a `BEADS_DIR` environment variable naming the workspace directory, and the harness verified that a workspace created there receives every file `bd init` produces while the repo stays untouched.

Workspaces live at `<spacetree root>/.spacetree/shared-state/beads-by-project/<project key>/`, checked out from a protected state branch of the spacetree repo and keyed by the identity defined in the project info spec, so task history is versioned without a repo of its own and a prefix rename never moves any data.
`wk` is the only entry point and sets `BEADS_DIR` for every call; upstream `bd` is deliberately not on `PATH`.
Because task history is the one thing a mistake cannot recreate, that branch is protected against force-push and deletion at the forge, and `wk` refuses to publish to it otherwise.

The design stays human-first: a workspace is ordinary files a human can inspect, back up, or delete, and losing one never requires touching a repo.

In scope: workspace placement, `BEADS_DIR` wiring, workspace creation, and the invariants that follow.
Out of scope: beads' own semantics and verbs, the routing grammar, and the project info schema, each owned by its own spec.

## Terminology & Key Concepts

**beads workspace**:
The directory holding one project's tracker state as a unit — configuration, metadata, README, and the Dolt database.
`bd` reads and writes it as one thing, and nothing else in the harness interprets its contents.

**external workspace**:
A beads workspace placed under `<spacetree root>/.spacetree/shared-state/beads-by-project/<project key>/` rather than inside the project repo.
It holds the files `bd init` produces; the contents are pinned down in *Placement / Layout*, and the command that creates it belongs to `<spec:wk-init>`.

**`BEADS_DIR`**:
The beads environment variable that names an external workspace's directory.
When it is set, that directory is the workspace itself, not a parent holding a `.beads/` child.

**project key**:
The identity from the project info spec that names the workspace directory.
The beads prefix is a separate attribute, so a prefix change never moves the workspace.

**state branch**:
The branch `PROTECTED/spacetree-shared-state` of the spacetree repo, holding task state rather than configuration.
It is protected at the forge against force-push and deletion.

**state worktree**:
The checkout of that branch at `<spacetree root>/.spacetree/shared-state/`.
`beads-by-project/` is this consumer's directory inside it, and every external workspace lives below that.
The human can read the task files there directly.

## Naming & IDs

A workspace path is derived, never stored:

```pseudocode
repo root      := git rev-parse --show-toplevel, or the main worktree's root for a linked worktree
spacetree root := walk up from repo root to the `.spacetree-repo` symlink
project info   := resolve at repo root (see the project info spec)
state worktree := <spacetree root>/.spacetree/shared-state  (worktree of the spacetree repo, branch `PROTECTED/spacetree-shared-state`)
workspace      := state worktree/beads-by-project/<project key>
```

The path depends on two things only: which spacetree the repo sits in, and the project key.
The beads prefix, repo name, checkout path, and branch do not appear in it.

- **Directory name**: the project key verbatim.
  Keys are lowercase, so the name is too.
- **No prefix in the path**: the prefix lives in the workspace's own config and metadata, so renaming it never moves data.
- **Spacetree scope**: state belongs to one spacetree.
  The same repo cloned into another spacetree gets that spacetree's workspace, never the first one's.
- **Missing project info**: there is no workspace path, and beads commands fail rather than guessing one from the directory name.
- **Versioned file**: only `issues.jsonl` is tracked on the state branch.
  Everything else in a workspace is generated and ignored.
- **State branch name**: `PROTECTED/spacetree-shared-state`, fixed for every spacetree.
  A fixed name lets a forge protection rule name the exact branch, and the `PROTECTED/` prefix marks what the branch requires.
- **One repo, two branches**: task state shares the spacetree repo but not its branch, so configuration history and task history never mix.

## Interface / How to use

### Every invocation

`wk` derives the workspace on every call and exports it to the wrapped `bd`:

```pseudocode
wk <verb> <args>:
    root      := spacetree root, from the cwd's repo
    info      := project info, resolved at the repo root
    workspace := root/.spacetree/shared-state/beads-by-project/info.key
    require workspace exists unless verb is init
    BEADS_DIR := workspace
    exec bundled bd <verb> <args>
```

`BEADS_DIR` is assigned, never inherited: a stale value in the caller's environment is overwritten, so a shell can never point `wk` at another project's workspace.

`bd` is deliberately absent from `PATH`, and that is what makes the derivation authoritative.
Without `BEADS_DIR` and without a `.beads/` directory, bare `bd` fails with `No active beads workspace found.` — verified behaviour, and the intended one: there is exactly one way to reach a workspace.

`wkg` and `wkt` set nothing themselves; they resolve a target and exec `wk`, which then derives the workspace for the target repo (see the routing spec).

A missing workspace on any verb other than `init` is an error, never an implicit creation:

```console
$ wk list
error: no beads workspace for "greenhouse-hub"; run `wk init`
```

### Committing

Task state reaches git only through an explicit snapshot, which also publishes it:

```pseudocode
wk commit:
    require workspace exists
    stage <workspace>/issues.jsonl on the state branch
    commit
    remote := the spacetree repo's remote
    if remote is absent or its forge cannot protect branches:
        warn "task history is local only; nothing was pushed"
        exit non-zero
    require state branch is protected (see State Branch Protection)
    git fetch, then push
```

`wk commit` stages the workspace's `issues.jsonl` and commits it on the state branch, then pushes; `wkg commit` does the same for a targeted spacetree or project.
The verbs are owned by the `wk` frontend spec, and automated snapshots are deferred (see *Global Open Questions*).
Nothing else writes to the state branch, and no beads command commits on its own.

A commit always survives a failed push: the local snapshot is kept, the push is refused, and the next `wk commit` retries after the cause is fixed.

### `wk init`

Workspace creation belongs to `wk init`, specified in `<spec:wk-init>`.
From this spec's side, that command must satisfy these properties:

- **At the derived path**, named by the project key and carrying the project's beads prefix;
- **Inside the state worktree**, so the workspace lands on the spacetree repo's state branch, not in machine-only state;
- **With the repo untouched**, because no beads artifact belongs in a working tree.

How it creates the workspace — the `bd init` invocation, the working directory, the flags, and the cleanup around beads' git integration — is that spec's business, not this one's.

## Placement / Layout

`<spacetree root>/.spacetree/shared-state/` is a worktree of the spacetree repo, checked out on its state branch.
`beads-by-project/` is the directory beads state owns inside it, one workspace per project key:

```text
<spacetree root>/.spacetree/shared-state/   # worktree of PROTECTED/spacetree-shared-state
├── beads-by-project/           # the directory beads state owns inside the shared state root
│   ├── .gitignore              # keeps everything but each workspace's issues.jsonl out of git
│   ├── greenhouse-hub/         # one external workspace per project key
│   │   ├── issues.jsonl        # the only tracked file
│   │   ├── config.yaml         # beads settings, generated
│   │   ├── metadata.json       # database name, project id, generated
│   │   ├── README.md           # beads' generated workspace readme
│   │   └── embeddeddolt/       # the workspace's Dolt database, generated
│   └── spacetree-harness/
└── …                           # other consumers' state
```

`beads-by-project/` carries a tracked `.gitignore` that ignores everything except each workspace's `issues.jsonl`, so beads' generated files never enter a commit and `git status` stays readable.
Because it sits inside the shared state checkout, task state is versioned through the spacetree repo's `PROTECTED/spacetree-shared-state` branch; the machine and runtime state roots beside it are not.

Because it is a real checkout, the human reads task files directly:

```sh
cd ~/self/.spacetree/shared-state/beads-by-project/greenhouse-hub
git log --oneline -- issues.jsonl
jq -r 'select(.status=="open") | .title' issues.jsonl
```

Task files can also be edited by hand; beads re-reads `issues.jsonl` on the next import.

A workspace is self-contained within the worktree.
Deleting its directory removes that project's tasks from the checkout and nothing else: no repo file points into the workspace, and no other project shares the directory.
Their old contents stay in the state branch's history until pruned.
A clean checkout into a fresh spacetree starts with no workspace until `wk init` runs there.

The repo side keeps zero beads artifacts:

- no `.beads/` directory, tracked or ignored;
- no `.gitignore` entry for beads;
- no generated `CLAUDE.md`, `.claude/` settings, or git hooks;
- only the `.git/info/exclude` line that `bd init --stealth` leaves, which is repository metadata and never shows in a diff;
- plus the project info file, or its `.project-info.toml` symlink.

Because the workspace is keyed per project rather than per checkout, every worktree and every clone of the repo inside the same spacetree resolves to the same workspace.

## State Branch Protection

Task history is the one thing a mistake cannot recreate, so the state branch must be protected at the forge against force-push and deletion.
The requirement is on the branch, not on the harness: GitHub branch protection, GitLab protected branches, or any equivalent that denies both operations.

`wk commit` verifies before it pushes, and refuses rather than publishing history that a later `push --force` could erase:

```console
$ wk commit
error: state branch `PROTECTED/spacetree-shared-state` allows force-push and deletion
hint: enable branch protection on the spacetree repo's remote, then retry
```

On GitHub the check reads the branch's protection or ruleset settings:

```console
$ gh api "repos/{owner}/{repo}/branches/PROTECTED%2Fspacetree-shared-state/protection" \
    --jq '{force_push: .allow_force_pushes.enabled, deletion: .allow_deletions.enabled}'
{"force_push": false, "deletion": false}
```

The branch name is URL-encoded because it contains a slash.
An absent protection setting, a 404, or a forge without the concept all count as unprotected.

Rules the harness holds:

- never force-push the state branch;
- never delete the state branch;
- never rewrite its history, including a re-init that would start the branch over.

A deliberate history rewrite is a human operation: unprotect, rewrite, protect again.
The harness has no verb for it.

## Invariants & Consequences

Invariants the design holds:

- **Zero beads artifacts in a repo.** The working tree only ever gains a project info file or its symlink; the one other trace, a line in `.git/info/exclude`, is repository metadata.
- **One workspace per project key per spacetree.** No checkout, branch, or worktree gets its own copy.
- **The workspace path never depends on the prefix, the repo name, or the checkout path**, so renaming any of those moves nothing.
- **No workspace without both a project key and a beads prefix**, and no implicit creation on any verb other than `init`.
- **`bd` never runs without `BEADS_DIR`**, because `bd` is not on `PATH` and there is no in-repo `.beads/` to fall back on.
- **Only `issues.jsonl` is versioned.** Every other file in a workspace is generated on the machine that uses it.
- **The state branch is never force-pushed, deleted, or rewritten**, and `wk commit` refuses to push while the forge does not block those operations.

Consequences worth accepting knowingly:

- **Task history leaves project git.** Task changes are not code-reviewed, not part of a repo's history, and not restored by checking out an old commit.
  History exists, but on the spacetree repo's state branch.
- **Task state propagates between machines** by cloning or pulling the spacetree repo and, where `spacetree init` has checked it out, reading the state branch worktree.
  Dolt-native transport (Dolt remotes, `bd dolt push`/`pull`) is out of scope and not used.
- **Protection requires a forge.** A local-only spacetree repo, or one on a forge without branch protection, cannot satisfy the requirement: each `wk commit` warns and refuses to push, leaving history local.
- **A fresh clone inside a fresh spacetree starts with no tasks.** The workspace belongs to the spacetree, not to the repo, so a repo cloned into a spacetree where `spacetree init` has not run cannot be initialized with `wk init` at all.
- **Deleting a workspace removes the project's tasks from the checkout**, while deleting a repo leaves an orphan workspace behind.
  Neither leaves a dangling reference inside the workspace's own state, which is why neither needs a cleanup protocol there.
  Cross-project `external:` edges are the exception: they are strings another project stores, and neither a rename nor a deletion can update them (see *Global Open Questions*).
- **No migration is required.** No project uses beads today, so nothing has to move; the harness repo's own tracked `.beads/` directory is deleted when this lands rather than migrated.

## Alternatives & Tradeoffs

The simplest alternative is upstream's default: a `.beads/` directory tracked in the repo.

**Option A — `.beads/` in the repo (upstream default).**
No environment variable, no derivation, and task history rides in project history for free.
But the workspace lands in the working tree, bd injects `CLAUDE.md`, `.claude/` settings, and hooks, and a repo with fifty siblings still has no spacetree-level view.
This is the direction the harness is explicitly moving away from.

**Option B — `.beads/` in the repo, only the database moved out.**
`bd dolt set data-dir` relocates the Dolt database while config, metadata, and `issues.jsonl` stay in the repo.
It is the smallest change and keeps a repo self-describing, but it leaves the artifacts that actually annoy — tracked workspace files, generated skills, hook takeover — and still keys state to a checkout.

**Option C — external workspace, no versioning.**
Workspaces inside the machine-state root as plain machine state.
Simple and fully isolated from git, but task history is lost with the machine and cannot move between machines at all.

**Option D — Dolt-native transport.**
`bd dolt push`/`pull` against a Dolt remote, or `bd backup` to a filesystem or DoltHub.
Beads-native, structured, and built for exactly this, but it needs a server or remote that the harness does not run, and it cannot be nested inside git — the two versioning models do not compose.

**Chosen — external workspace on a spacetree repo state branch.**
The workspace lives outside every project repo, and its one meaningful file, `issues.jsonl`, is versioned in the spacetree's existing spacetree repo on a separate branch, checked out as a worktree.
Advantages: repos stay clean, history and cross-machine propagation come from plain git the human already owns, and `issues.jsonl` is a readable, diffable file.
Costs: a second branch and a worktree to keep tidy, only `issues.jsonl` survives versioning (Dolt history does not), and task changes never appear in project history.

**Decision criteria**: use Option A or B when a repo must be self-contained and its task history should travel with its code; use Option C when tasks are disposable; use Option D when a Dolt remote already exists and structured history matters more than human readability.
Use the chosen design when many repos share one spacetree, repos must stay pristine, and versioning should reuse git the human already has.

## Related artifacts

- `<spec:project-info>` — defines the project key that names every workspace, and the `[beads] prefix` the workspace carries.
- `<spec:wk-beads-wrappers>` — owns `wk`, its bundled `bd`, and the `wk commit`/`wkg commit` verbs.
- `<spec:wk-init>` — owns `wk init`, the command that creates the workspace this spec places.
- `<spec:cross-spacetree-task-routing>` — resolves `wkg`/`wkt` targets to a project before `wk` derives the workspace.
- `<spec:spacetree-repo>` — the state roots under `<spacetree root>/.spacetree/`, the `PROTECTED/` branch convention, and the spacetree repo whose state branch holds task history.
- `HANDOFF-20260920-beads-shared-server-devtree.md` — the per-spacetree Dolt server decision that this spec deliberately leaves out of scope.
- `docs/beads-init-complaints.md` — the in-repo side effects that motivated moving the workspace out.
- `tooling/beads-devtree-init/beads-devtree-init` — the existing bootstrap whose `bd init` handling `wk init` replaces.

## Global Open Questions

**Divergence on commit** — `wk commit` fetches before pushing, but nothing settles what happens when the remote state branch has moved: rebase, merge, or refuse until a human intervenes.
`issues.jsonl` conflicts are line-level and may not be semantically mergeable, which is the reason this is undecided.
Blocking for `wk commit` implementation.

**Protection verification surface** — Whether the check reads classic branch protection only, repository rulesets only, or both, and whether the result is cached per session.
Non-blocking; the requirement is that an unprotected branch is detected, not which API answers.

**`wk rename`** — Deferred idea: a command to rename a project key and/or its beads prefix, moving the workspace directory and rewriting the project info.
Non-blocking. Keying workspaces by project key makes the move a single rename, but the command itself is not designed yet.
A rename also leaves every `external:` edge that names the old key dangling; see the next entry.

**Cross-project references outliving their target** — A `bd` dependency on `external:<project>:<cap>` is stored as an opaque string in the depending project's `issues.jsonl`.
Nothing here can update it when the target key is renamed or its workspace is deleted, and nothing notices that the target is gone: `bd` keeps the edge as an unresolved reference with no row behind it.
Which side owns the consequence — the routing spec's resolution design, a `wk rename` rewrite pass, or an accepted permanent dangling reference — is undecided.
Blocking for the claim that the workspace needs no cleanup protocol, which the *Invariants & Consequences* entry now qualifies.
Non-blocking otherwise: the workspace never reads those edges.

**Automated snapshots** — Whether `wk` eventually commits on a timer, and at what cadence.
Non-blocking; automatic pushes without human review are the reason this stays manual for now.

**Terminology & Key Concepts** (TKC) — Whether this section is needed for this spec.
Non-blocking. Refer to spec-writing skill for guidance.
