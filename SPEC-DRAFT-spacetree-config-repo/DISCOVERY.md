# Spacetree Config Repo — discovery

## Problem

A spacetree root is a non-git container whose only spacetree-wide artifact is an empty `.spacetree-root` marker file.
Nothing at the spacetree level is versioned: hierarchy, naming, and per-project metadata have no home.
That is the "spacetree config file as the source of truth" gap in `README.md`.

## Inputs

- Repo context: the spacetree harness, its README, and the spacetree-daemons / routing specs.
- The out-of-tree work: per-project infos need a store that is versioned and shared across the spacetree.
- Constraint: the spacetree root must not become a git repo, and projects kept inside it must stay untouched.

## Settled decisions

- One config repo per spacetree, a git repo, holding spacetree-wide configuration.
- `.spacetree-root` becomes a symlink whose target is the config repo directory; the repo may live anywhere, inside or outside the spacetree.
- `spacetree.toml` at the config repo root declares the spacetree `name`; the directory basename is never the name.
- `project-infos/<key>.toml` is the project info store for projects that link rather than track their project info.
- Root resolution walks up for `.spacetree-root`, resolves the symlink, and errors on every failure mode instead of falling back.
- `spacetree root --json` gains `config_repo` and reads `name` from the config.
- Runtime state stays in `<spacetree>/.state/`, never in the config repo.

## Open threads

- Recommended default location.
- Absolute vs relative symlink target.
- Whether `spacetree init` scaffolds and git-inits the config repo.
- Fate of the global known-spacetrees list.
- Whether `process-compose.yaml` moves into the config repo.
