# [READY] wk init

## Introduction

`wk init` is the one command that turns a freshly cloned project into a project the devtree harness understands.
It runs from the project root and performs every per-concern bootstrap step in a fixed order: write the project info, then create the beads workspace.
Each step has an owner spec; this spec owns the sequence, the shared assumptions, and the failure behavior that spans steps.

Today that sequence exists in pieces.
The pieces live in the `wk` frontend spec (`<spec:wk-beads-wrappers>`), the project-info spec (`<spec:project-info>`), the beads out-of-tree storage spec (`<spec:beads-out-of-tree-storage>`), and a Python tool that was drafted but never used, `beads-devtree-init`.
The pieces disagree on storage, and no reader can tell which artifact is authoritative.

This spec makes `wk init` the single authority for the command.
The concern specs keep owning their formats, paths, and invariants, and point here for the workflow that creates them.
The legacy wrapper is documented as superseded behavior, not as an alternative implementation.

Overriding stock `bd init` is already motivated elsewhere: the defaults that make it unfit for this harness are recorded in `docs/beads-init-complaints.md`.

The design follows the harness's human-first constraint (`README.md`): every step is visible to the human running the command, prints what it created, and can be reproduced by hand.
Nothing in the sequence commits, pushes, or rewrites history: `wk init` leaves reviewable working-tree changes and stops.

## Interface / How to use

Run `wk init` from the project root.
It takes no positional arguments; everything it needs comes from the project, the devtree, and its flags.

An interactive run asks for each decision in order and prints the result of every step.
The prompts and their wording belong to the specs that own each artifact; this spec fixes only their order and the manifest.

```
wk init: initialized greenhouse-hub
devtree root: <devtree>
project info: project-info.toml
beads workspace: <devtree>/.state/beads-by-project/greenhouse-hub/
nothing committed
```

The ownership prompt and its detected default come from `<spec:project-info>`; the workspace path and contents come from `<spec:beads-out-of-tree-storage>`.

Flags make the same run non-interactive:

- `--key=<key>` — the project key; required without a terminal.
- `--owner=me|other` — the ownership answer; required without a terminal.
- `--prefix=<prefix>` — the beads prefix; required without a terminal.
- `--dry-run` — print the manifest as a preview and perform no writes.
- `--non-interactive` — never prompt; fail instead of guessing.

```sh
wk init --non-interactive --key=greenhouse-hub --owner=me --prefix=ghh
```

A non-interactive run takes every answer as given, so a missing flag is an error rather than a default.

The final manifest is the human's review surface: it lists the files and directories created and the fact that nothing was committed.
Staging and committing stay with the human.
`--dry-run` prints that same manifest, marked as a preview, without creating anything.

## Init Sequence

`wk init` is a fixed pipeline: each step consumes what the previous one produced, and no step is skippable.

```pseudocode
resolve repo root, devtree root, config repo, devtree name
refuse when a project info or the beads workspace already exists
key, owner, prefix := project info step     # the answers that step collects
workspace          := beads workspace step  # workspace per <spec:beads-out-of-tree-storage>
print manifest
```

The order is forced by the data, not by taste:
the project key is created before anything keyed by it, because the workspace path embeds that key;
the beads prefix is a project-info attribute, so `bd init` cannot run before the project info exists;
and the manifest is printed once every step has reported its outcome.

Skill installation is not part of this command, and projects carry no skills of their own: devtree-level skills belong to `devtree init` and apply to every project under it.

### Preconditions

Before any write, `wk init` requires:

- the project root as the current directory, and that root is a git work tree, including linked worktrees;
- a `.devtree-root` marker above it, with a config repo that resolves (`<spec:devtree-config-repo>`); a missing or dangling one is a refusal, not a fallback.
- a state worktree at `<devtree>/.state/beads-by-project/`, created by `devtree init` (`<spec:devtree-config-repo>`); a missing checkout is a refusal, not something `wk init` creates;
- no project info in the repo (`<spec:project-info>`);
- no existing beads workspace for the resolved key (`<spec:beads-out-of-tree-storage>`).

A failed precondition stops the run before anything is written.
`wk init` never falls back to a default devtree, an implicit key, or an in-repo beads workspace.

### What each step owns

The project info step and the beads workspace step delegate their file rules to `<spec:project-info>` and `<spec:beads-out-of-tree-storage>`.
This spec owns their order, the flags each step reads, and the manifest.

`wk init` does not scaffold a missing config repo, marker, state branch, or state worktree: it refuses and points at `devtree init`, which owns devtree creation, its docs, and the devtree-level skills.
It also never installs git hooks, edits `AGENTS.md`, or commits.

### Open Questions

**`devtree init` contract** — What `devtree init` owns: config repo scaffolding, the marker, `devtree.toml`, the state branch and state worktree, and devtree-level skills, plus the exact refusal text `wk init` prints when the checkout is missing.
Non-blocking. `devtree init` is covered only in fragments today (`<spec:devtree-daemons>`, `<spec:devtree-config-repo>`); no spec owns the whole command.

## Project Info Step

This step collects the three answers `wk init` needs — project key, beads prefix, ownership — in a fixed order, then asks the project-info rules to create the file.

```pseudocode
key    := --key, or the prompt; required
prefix := --prefix, or the prompt; required
owner  := --owner, or the interactive prompt
create the project info
```

Only the ordering and the flag mapping are specific here.
Ownership detection, the prompt menu, both file forms, the store layout, the exclusion rule, and duplicate-key refusal all belong to `<spec:project-info>`.

The key and prefix are this step's output: the key names the beads workspace, and the prefix is what `bd init` receives.
Neither is re-derived later.

The step never edits an existing project info, because the precondition refused that case.

## Beads Workspace Step

This step creates the external workspace for the key the project info step collected.
The derived path, the workspace contents, and every invariant about placement belong to `<spec:beads-out-of-tree-storage>`;
this section fixes only how the command gets there.

```pseudocode
state worktree := <devtree>/.state/beads-by-project  # checkout of PROTECTED/beads-state-by-project, created by devtree init
workspace := state worktree/<key>
run bundled bd init with BEADS_DIR=workspace, cwd=project root
    --prefix=<prefix> --non-interactive --stealth --skip-agents --skip-hooks
print the workspace path
```

The state worktree is a devtree-level resource, not a per-project one: `devtree init` creates it when it scaffolds the devtree, and every `wk init` reuses it.
`wk init` never creates, moves, or prunes the worktree, and it never creates the state branch; both belong to `<spec:devtree-config-repo>`.
A devtree whose checkout is missing fails the precondition above rather than being repaired here.

`bd init` runs with the project root as its working directory and reaches the workspace through `BEADS_DIR`.
The workspace is never the cwd: beads writes its files at `BEADS_DIR` and reads the repo only to infer what it needs about the checkout.

Each flag removes a stock behavior the harness does not want:

- `--prefix=<prefix>` sets the issue prefix collected in the project info step.
- `--non-interactive` stops beads from opening prompts of its own, because `wk init` already collected every answer.
- `--stealth` keeps beads' git integration to the `.git/info/exclude` line and away from tracked repo files.
- `--skip-agents` and `--skip-hooks` refuse beads' generated agent instructions and git hooks.

The step does not stage or commit the new `issues.jsonl`, and it does not undo a commit made behind its back.
There is nothing to undo: no beads command runs with a repository it may commit to.

## Failure & Idempotency

`wk init` is not transactional, and it does not pretend to be.
A step that fails stops the run, leaves what it already wrote on disk, and exits non-zero after naming the failed step and the paths it had created.
There is no rollback: the outputs are ordinary files, and a human can see and remove them.

Every precondition is checked before the first write, so the common refusals leave nothing behind:

- the current directory is not the root of a git work tree;
- no `.devtree-root` marker above it, or a config repo that does not resolve;
- no state worktree at `<devtree>/.state/beads-by-project/`, which means `devtree init` has not run;
- a project info already exists in the repo;
- the workspace directory for the resolved key already exists;
- the key is already declared by another repo path (`<spec:project-info>`);
- a required flag is missing in a non-interactive run, or the key or prefix is invalid.

Because the project info is written before the workspace, only one partial pair is reachable: a project info with no workspace behind it.
That happens when the project info write succeeds and `bd init` then fails, or when beads leaves a half-written workspace directory before failing.
The failing run prints the leftover path, and the next run refuses on it rather than guessing whether to resume.

Re-runs are not idempotent in the "quietly succeed" sense: once the project info or the workspace exists, `wk init` refuses.
There is no `--force`, no resume mode, and no repair path.
Completing a half-finished run is a human operation: finish the missing step by hand, or remove the leftover and run `wk init` again.

The state worktree is devtree-level and owned by `devtree init`: `wk init` requires it, never creates it, and never removes it.
That is why a missing checkout is a precondition failure and not something a partial run can leave behind.

`--dry-run` runs the same preconditions and prints the same manifest, so it refuses exactly where a real run would.
A preview never reports success for a run that would stop.

Nothing is committed at any point, so there is no git state to undo and no commit to reset.
A written project info and a created workspace survive a failed run.

## Legacy `beads-devtree-init`

`beads-devtree-init` is the Python tool that was drafted for this bootstrap but never used.
`wk init` supersedes it before either reached a project: the command name, the packaging, and the move into the Rust frontend belong to `<spec:wk-beads-wrappers>`, and this section only records which of the draft's behaviors survive the rewrite.

The draft's internals are deliberately left out of the spacetree renaming (`<spec:spacetree-rename>`):
it still walks up for `.devtree-root` and prints `devtree root`, naming that predates the rename.
Because it is a dead draft that never ran in a project, the mismatch is recorded here rather than fixed.

Carried forward, because they are still the right behavior:

- refusing outside a git work tree, and resolving the devtree by walking up for the marker rather than guessing;
- prompting for the issue prefix instead of deriving one from the directory name;
- running `bd init` with `--stealth`, `--skip-agents`, and `--skip-hooks`;
- passing `--non-interactive` to beads when the human is not at a terminal;
- printing the devtree root and the paths that were created;
- refusing when a workspace already exists instead of replacing it.

Superseded, and deliberately dropped:

- **The in-repo `.beads/` workspace and its tracked files.** The workspace now lives outside the repo, so there is nothing to force-add and no `.beads/.gitignore` to curate (`<spec:beads-out-of-tree-storage>`).
- **The `git reset --soft` around bd's auto-commit.** `wk init` never commits and out-of-tree beads has no repo commit to undo, so the reset has no subject.
- **Skill installation and the `--skills in-repo|devtree|skip` choice.** Devtree-level skills belong to `devtree init`, and projects carry none.
- **The `--repo` flag.** `wk init` runs from the project root and takes no target argument.
- **`-i`/`--interactive`.** Interactivity is the default, and the explicit flag would only restate it.
- **Forwarding trailing arguments to `bd init`.** `wk init` accepts no pass-through arguments; anything beads-specific belongs to later `bd` calls through `wk`.

The prefix prompt carries forward, but not alone: it becomes one answer in the project info step, alongside the project key and ownership.

The script, its Nix package, and its bats suite stay in the tree as the unused draft they are, until the Rust frontend replaces them.
No behavior is added to the Python tool, and no fix lands there first.

## Placement / Scope

`wk init` is a verb of the `wk` frontend, and it runs at the project root.
The project is not selectable: no path, `--repo`, or positional argument names another checkout.
The current directory must be the root of the project's work tree; a run from a subdirectory is refused rather than resolved upward.

In scope for this spec:

- the order of the bootstrap steps and the preconditions checked before them;
- every prompt the command asks, the flag that replaces each one, and the non-interactive contract;
- the failure behavior and what a partial run leaves behind;
- the manifest and what it reports.

Out of scope, with the owning spec:

- the project info format, key grammar, ownership detection, resolution, and store layout — `<spec:project-info>`;
- workspace placement, contents, and how task state is committed — `<spec:beads-out-of-tree-storage>`;
- the `wk`/`wkg`/`wkt` family, its packaging, and the fact that `init` is the one intercepted verb — `<spec:wk-beads-wrappers>`;
- the marker, the config repo, and the state branch — `<spec:devtree-config-repo>`;
- devtree-level skills, which belong to the future `devtree init`;
- target resolution for other devtrees and projects — `<spec:cross-devtree-task-routing>`.

What a run may write:

- the project info, in whichever form `<spec:project-info>` selects, including its store entry and exclusion line;
- the workspace directory inside the existing state worktree;
- the manifest on stdout.

Nothing else.
It does not touch `AGENTS.md`, `CLAUDE.md`, `.claude/`, `.gitignore`, git hooks, or any file in another project, and it does not commit or push in either the project repo or the config repo.
The writes are confined to files a human would otherwise create by hand.

## Alternatives & Tradeoffs

The closest alternative is not writing the command at all: run stock `bd init` and create the project info by hand.

**Option A — stock `bd init` plus manual files.**
No code to maintain, and every repo can pick its own beads settings.
But `bd init` writes `.beads/` into the working tree, injects agent instructions and git hooks, and commits on its own, so each human has to know which side effects to undo.
The project info would also be hand-written per repo, with no ownership prompt and no uniqueness check, which is exactly the drift this harness exists to prevent.

**Option B — keep the `beads-devtree-init` draft and extend it.**
The wrapper already knows the marker walk-up and the harness-safe `bd init` flags.
But its model is the in-repo workspace this harness is abandoning, and its skills placement step no longer has a place in the design.
Extending it would mean rewriting the storage behavior and then porting the result to Rust anyway.

**Option C — two commands, one per concern.**
A `wk init` that only writes the project info, and a separate command that only creates the workspace.
Each step gets its own lifecycle, a failed step is retried in isolation, and no run leaves a half-finished pair.
The cost is that the human must know the order and run two commands, and the second one would have to re-derive the key and prefix or re-ask for them.

**Option D — let a clone trigger the bootstrap automatically.**
A hook or a shell integration notices a repo without a project info and offers to initialize it.
Zero commands to remember, but it writes to a repo the human did not ask it to touch, and it cannot prompt in every context a clone happens in.

**Chosen — one `wk init`, sequenced, interactive by default.**
The order is forced by the data, the answers are collected once, and a single manifest reports everything the run created.
The command stays a normal foreground command a human can read and repeat.

**Decision criteria**: use Option A when a repo is genuinely self-contained and beads' defaults are acceptable; use Option C when the project info and the workspace need independent lifecycles, such as a harness that starts tracking repos before they use beads.
Use the chosen design when both artifacts are always created together, which is the case for a project joining this harness.

## Related artifacts

- `<spec:project-info>` — the file this command creates, and the key, prefix, and ownership it collects.
- `<spec:beads-out-of-tree-storage>` — the workspace this command creates, its placement, and the state branch.
- `<spec:wk-beads-wrappers>` — the `wk` frontend that hosts the verb, and the Rust implementation that replaces the draft tool.
- `<spec:devtree-config-repo>` — the marker, config repo, and state worktree the command depends on.
- `<spec:cross-devtree-task-routing>` — the project key and registry this command feeds.
- `<spec:devtree-daemons>` — the `<devtree>/.state/` convention the state worktree lives under.
- `<spec:spacetree-rename>` — the vocabulary rename that deliberately leaves the legacy draft untouched.
- `docs/beads-init-complaints.md` — the stock `bd init` side effects that justify the flags and the external workspace.
- `tooling/beads-devtree-init/` — the unused Python draft, its Nix package, and its bats suite.
- `HANDOFF-20260920-beads-shared-server-devtree.md` — the one-Dolt-server-per-devtree decision left to the storage spec.
- `README.md` — the `repo-init` gap this command partly fills, and the human-first constraints it follows.

## Global Open Questions

**Relationship to `repo-init`** — Whether `wk init` becomes one step of the generic `repo-init` wrapper noted as a gap in `README.md`, or stays a standalone verb that `repo-init` calls.
Non-blocking; the command is specified standalone either way.

**Dolt server at first init** — Whether `wk init` ensures the devtree's Dolt server is running, or leaves the first `wk` call to start it.
Non-blocking; the server itself has no spec yet (`HANDOFF-20260920-beads-shared-server-devtree.md`), and this spec does not depend on which way it lands.

**Registry timing** — Whether init writes the project registry entry, or the registry learns about the project on first resolution.
Cross-referenced with `<spec:cross-devtree-task-routing>`, which owns the answer; `wk init` only guarantees the project info it reads exists.
Non-blocking.

**Protection bootstrap** — Whether the `devtree init` run that creates the state branch also enables forge protection, or only reports it as missing.
Owned by `<spec:devtree-config-repo>`; nobody in the harness commits during init, so an unprotected branch only matters at the first `wk commit`.
Non-blocking.

**Terminology & Key Concepts** (TKC) — Whether this section is needed for this spec.
Non-blocking. Refer to spec-writing skill for guidance.
