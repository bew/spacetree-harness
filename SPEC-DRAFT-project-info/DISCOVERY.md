# Project Descriptor

## Problem

Project identity and project-level metadata have no machine-readable home.
Each project's identity is guessed from its directory name, and beads uses `issue_prefix` as the project identity.
Projects that are not ours to modify (open-source clones) still need tracking, so the metadata cannot always live in the repo.

For: the task-routing layer (project addressing), the spacetree registry, and external references (`external:<project>:<cap>`).

## Inspirations

- beads' own `metadata.json` / `config.yaml` scheme, which shows the cost of splitting identity across files.
- The per-project info idea that came up while specifying out-of-tree beads storage.
- Existing repo practice: `.envrc`/flake files at repo root, tracked where the repo is controllable.

## Constraints stated by the user

- Core schema plus open sections: a small fixed core (key, tickets, docs), extra named sections allowed and ignored by the harness.
- Key and beads prefix are separate identifiers.
- No automatic lookup tier and no derive-from-dirname fallback: refuse when no project info is found.
  Sharing a parent's identity is an explicit `.project-info.toml` link, not an upward search.
- In repos we do not control, link the project info instead of tracking it.
- The link is dot-prefixed, so it never collides with a tracked `project-info.toml`.
- The project info store belongs to the spacetree config repo, not the spacetree root.

## Settled decisions

- Tracked `project-info.toml` where the repo is controllable; otherwise a gitignored `.project-info.toml` symlink.
- Symlink target: `<config repo>/project-infos/<key>.toml`.
- Both files present is an error; neither is a refusal.
- Exclusion goes through `.git/info/exclude`, never the repo's `.gitignore`.
- The project key, not the beads prefix, is the addressing identity.
- `wk init` asks tracked vs linked, with non-interactive flags.
- No migration: no project uses beads today.

## Open threads

- Exact non-interactive flag spelling.
- Whether the symlink target must resolve inside the project info store.
- Key charset vs beads prefix charset.
- Behaviour of `wkg` when a target project has no project info.
