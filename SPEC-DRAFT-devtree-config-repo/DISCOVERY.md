# Devtree Config Repo — discovery

## Problem

A devtree root is a non-git container whose only devtree-wide artifact is an empty `.devtree-root` marker file.
Nothing at the devtree level is versioned: hierarchy, naming, and per-project metadata have no home.
That is the "devtree config file as the source of truth" gap in `README.md`.

## Inputs

- Repo context: the devtree harness, its README, and the devtree-daemons / routing specs.
- The out-of-tree work: per-project infos need a store that is versioned and shared across the devtree.
- Constraint: the devtree root must not become a git repo, and projects kept inside it must stay untouched.

## Settled decisions

- One config repo per devtree, a git repo, holding devtree-wide configuration.
- `.devtree-root` becomes a symlink whose target is the config repo directory; the repo may live anywhere, inside or outside the devtree.
- `devtree.toml` at the config repo root declares the devtree `name`; the directory basename is never the name.
- `project-infos/<key>.toml` is the project info store for projects that link rather than track their project info.
- Root resolution walks up for `.devtree-root`, resolves the symlink, and errors on every failure mode instead of falling back.
- `devtree root --json` gains `config_repo` and reads `name` from the config.
- Runtime state stays in `<devtree>/.state/`, never in the config repo.

## Open threads

- Recommended default location.
- Absolute vs relative symlink target.
- Whether `devtree init` scaffolds and git-inits the config repo.
- Fate of the global known-devtrees list.
- Whether `process-compose.yaml` moves into the config repo.
