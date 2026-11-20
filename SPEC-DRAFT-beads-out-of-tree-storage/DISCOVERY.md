# Beads Out-of-tree Storage

## Problem

`bd` installs its workspace inside the repo (`<repo>/.beads/`): tracked files, gitignore edits, hooks, sometimes a commit.
That pollutes every project, couples task data to project history, and forces per-repo bootstrap work.
The user wants beads set up, its files, and its storage outside the repo.

For: the `wk`/`wkg`/`wkt` frontend, and the per-devtree shared Dolt server handoff.

## Inspirations

- `HANDOFF-20260920-beads-shared-server-devtree.md`: one Dolt server per devtree, and its blocking open question about how `bd` learns the socket path without a repo-tracked file.
- `docs/beads-init-complaints.md`: the bd init complaints (CLAUDE.md, `.claude/`, AGENTS.md injection, hook takeover, auto-commit, no dry-run).
- `tooling/beads-devtree-init/`: the superseded `bd init` draft wrapper.

## Facts verified against bd 1.3.0

- `BEADS_DIR=<dir>` is supported, and the workspace is that directory itself, not a parent of `.beads`.
- With `BEADS_DIR` set, `bd where --json`, `bd create`, and `bd list` work from the repo; without it, bare `bd` fails with `no_beads_directory`.
- `bd init --stealth` with cwd = the repo and `BEADS_DIR` external leaves the working tree clean, creates no commit, and only appends to `.git/info/exclude`.
- Running `bd init` with cwd = the workspace or its parent leaks `.gitignore` / `*.gate.lock` outside the workspace, so cwd must be the project repo.
- `bd dolt set data-dir <abs path>` exists.

## Constraints stated by the user

- Move the whole workspace, not just DB storage.
- One beads workspace per devtree, under the devtree `.state/`.
- No migration work: zero projects use `.beads` today.
- Upstream `bd` stays off `PATH`; `wk` is the only entry point.

## Settled decisions

- Workspace path: `<devtree>/.state/beads-by-project/<key>/`, keyed by project key so a prefix rename never moves data.
- The repo keeps zero beads artifacts.
- One Dolt server per devtree at `<devtree>/.state/dolt.sock`; the socket setting lives in the workspace `config.yaml`, outside the repo, which answers the handoff's blocking question.
- `wk init` runs `bd init` with `BEADS_DIR` set and cwd = repo root, `--prefix` from the project info, plus `--non-interactive --skip-agents --skip-hooks --stealth`.
- No force-add of beads files, no auto-commit undo.
- Git worktrees of a project share the one workspace automatically, because `BEADS_DIR` does not depend on cwd.

## Open threads

- Dolt server lifecycle: lazily started by `wk`, or managed by the devtree daemon manager.
- Backup policy, since task data is no longer in project git.
- Pruning workspaces whose project disappeared.
- Exact `config.yaml` keys for the socket connection.
