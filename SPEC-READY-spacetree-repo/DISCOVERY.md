# Spacetree Repo — discovery

## Problem

A spacetree root is a non-git container whose only spacetree-wide artifact is a marker.
Nothing at the spacetree level is versioned: hierarchy, naming, per-project metadata, and the material every project under the tree shares have no home.
That is the "spacetree config file as the source of truth" gap in `README.md`, and the reason spacetree-level skills and tools had no landing place.

## Inputs

- Repo context: the spacetree harness, its README, and the spacetree-daemons / routing specs.
- The out-of-tree work: per-project infos need a store that is versioned and shared across the spacetree.
- Constraint: the spacetree root must not become a git repo, and projects kept inside it must stay untouched.

## Settled decisions

- One spacetree repo per spacetree: a git repo holding configuration at its root and inherited content under `tree-shared/`.
- `.spacetree-repo` is a symlink at the spacetree root whose target is the spacetree repo directory, so its location is a fact the registry records; the repo sits inside the root, at the `meta` entry's default path.
- `.spacetree-shared-root` is a second symlink at the spacetree root, targeting `<repo>/tree-shared/`; it is co-authoritative with the repo marker, and `spacetree status` reports a mismatch.
- `spacetree.toml` at the spacetree repo root declares the spacetree `name`; the directory basename is never the name.
- `projects/<key>.toml` is the project info store for projects that link rather than track their project info.
- Root resolution walks up for `.spacetree-repo`, resolves the symlink, and errors on every failure mode instead of falling back.
- `spacetree root --json` reports the root, the repo, the shared marker target, and the declared name.
- Runtime state stays in `<spacetree root>/.spacetree/`, never in the spacetree repo.

## Settled by the spec

- Default location: `$root/spacetree-home`, from the `meta` entry's `default-path`.
- Symlink targets are stored relative to the directory holding the marker.
- `spacetree init` creates the markers, the machine-state label, the `PROTECTED/` state branches and their checkouts; it writes `spacetree.toml` when it is absent and never rewrites existing configuration.
- The known-spacetrees list stays, recording spacetree root paths; it is owned by `<spec:cross-spacetree-task-routing>`.
- `process-compose.yaml` lives at the spacetree repo root, versioned with configuration.

## Inherited content

The repo holds more than configuration: agent instructions, skills, shared docs, tooling, templates.
One consumer already reads that material today (`integrations/opencode-plugin/spacetree-context.ts` reads `<root>/AGENTS.md` and `<root>/.agents/skills`), and it must resolve it through the new markers.

### Settled decisions

- In-scope content classes: agent instructions (`AGENTS.md`), spacetree skills (`.agents/skills/`), shared docs/notes, tooling/scripts, templates/scaffolds.
- One namespace directory holds all of it: `tree-shared/` at the spacetree repo root.
- Configuration stays at the repo root (`spacetree.toml`, `projects/`); only inherited content lives under `tree-shared/`.
- `tree-shared/` mirrors the spacetree-root layout, so a consumer resolves `.spacetree-shared-root` and inherits `.agents/skills/`, `AGENTS.md`, `docs/`, `scripts/`, `templates/` from it.
- The spacetree root holds no agent files of its own.

### Settled by the expansion

- A project's own `AGENTS.md` and skills are read on top of the inherited copy, with the precedence fixed in *Inherited Content*.
- `tree-shared/` is the final name.
- Secrets stay in the environment; nothing secret goes in the spacetree repo.
- `spacetree status` is a subcommand of the same binary that answers `spacetree root`.
