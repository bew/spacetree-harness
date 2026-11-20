# Discovery notes

## Name and slug

- Name: **Spacetree rename**
- Slug: `spacetree-rename`

## Problem

"devtree" carries two jobs: the general organizational unit of the harness, and the code-oriented case of it.
The double duty implies a single broad personal tree, which contradicts the domain-specific boundaries wanted
(administration, home maintenance, configuration, work, personal code as separate trees).
The harness also has no settled vocabulary for trees that are not code-oriented, so cross-tree relationships
have no name for the space they connect.
This spec settles the vocabulary, defines `devtree` as a specialization, documents both, and carries the naming
through every artifact in this repo in one pass.

## Inspirations and prior material

- The naming and conceptual-boundaries discussion of October 25, 2026 — primary input for the concept model.
  Its record is folded into this spec; nothing outside the spec preserves it.
- `README.md` — current project-global doc; describes the practice analysis and the `.devtree-root` marker.
- The READY and DRAFT specs that already depend on the old names (`cross-devtree-task-routing`, `devtree-daemons`,
  `task-scopes`, `wk-beads-wrappers`, `devtree-config-repo`, `beads-out-of-tree-storage`, `project-info`, `wk-init`)
  — they define the actual rename surface.

## Decisions taken during discovery

Beyond the October 25, 2026 naming discussion, the user settled the following:

- **Both corrections from the October 25, 2026 naming discussion are carried as stated**: a single broad personal tree was rejected in favour of
  domain-specific trees, and `devtree` as a capability profile was an assistant proposal, not a decision. The spec
  records the first as the boundary model; the second stays only as the sentence that `devtree` establishes no type
  system, inheritance, or capability profile.

- **Scope is concept model plus full migration plan.** This overrides the October 25, 2026 naming discussion's statement that no rename was
  authorized. The concept-model checklist from that same discussion still applies to the concept sections.
- **Rename surface**: repo name, Nix package attr, CLI binary, `.devtree-root` marker, side tooling names,
  spec directory names, and spec prose.
- **beads DB prefix and issue IDs are out of scope**: no live beads setup exists. The `devtree-harness` prefix
  strings in `docs/beads-refs/data-model.md` are copied upstream examples and stay.
- **Transition is a hard cutover in one pass** — no marker dual-read, no CLI alias, no deprecation window.
- **`beads-devtree-init` keeps its name** and is marked legacy; `SPEC-READY-wk-init` already supersedes it.
- **`devtree` has no machine-readable meaning yet.** The doc states the distinction is vocabulary only;
  a machine-readable designation is deferred to a future spec.
- **Harness core stays spacetree-agnostic, advisory only.** No git/repo assumptions in resolution, scopes,
  routing, or daemons; recorded as intent rather than an enforced invariant.
- **Concept documentation lives in a new `docs/what-is-a-spacetree.md`**, written for users of the harness,
  with `README.md` linking to it.
- **That doc is user-level**: it explains the concept and gives a brief why, keeps decisions and tradeoffs in the
  spec, and notes in one line that `devtree` now names only the code-oriented case — no migration narrative.

## Constraints from the environment

- `~/self/.devtree-root` and `~/work/.devtree-root` exist as empty regular files (not symlinks).
  A repository-wide and home-wide search for `devtree` outside this repo returns nothing, and no autoenv file,
  profile, or module reads either marker, so the marker rename is cosmetic and needs no cutover window.
- `integrations/opencode-plugin/devtree-context.ts` is not registered in `~/.config/opencode`, so side-tooling
  renames stay inside this repo.
- `~/work/.autoenv.zsh` and the tree directory names (`~/self`, `~/work`) are untouched.
- Tree directory names are not being renamed; only the marker and harness artifacts are.
- The `wkg` axis label and `$XDG_STATE_HOME/devtree-global/` appear only in specs about future tooling; no
  deployed path or data exists for either, so neither needs a migration step.

## Project-global docs

- `README.md` — global today; keeps intro, package inventory, practice analysis, and gains a link to concepts.
- `docs/what-is-a-spacetree.md` — does not exist yet; created by this spec's migration as the authoritative
  user-level vocabulary document.
- Not concept docs: `docs/beads-refs/*` (copied upstream reference), `docs/beads-init-complaints.md` (complaint record).

