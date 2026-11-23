---
created-at: 20261103T2113
---

# [READY] Spacetree rename

## Introduction

The harness organizes work into rooted directory trees: a directory that gathers projects, documentation, tasks, configuration, and supporting tooling for one coherent domain.
The concept document this spec creates, `docs/what-is-a-spacetree.md`, is its authoritative description.

One word named both that general unit and its code-oriented case: "devtree".
The double duty carries these problems:

- It implies a single broad personal tree, which is not the granularity the harness wants.
- It leaves non-code spaces — administration, home maintenance, machine configuration — without a name for what they are.
- The definition in `README.md` is written around the root marker and task tracking, both of which apply to any tree, so the term describes nothing code-specific while implying that it does.

This spec settles the vocabulary, narrows `devtree` to a specialization, and carries the naming through every artifact in this repository in one pass.
It serves the human using the harness, the agents reading its documents, and the existing specs that currently reference `.devtree-root` and `devtree`.

What it delivers:

- Vocabulary — `spacetree` for the general unit, `devtree` for a code-oriented one.
- A user-level concept document at `docs/what-is-a-spacetree.md`, with `README.md` reduced to an overview that links to it.
- A hard cutover covering the repository name, Nix package attribute, CLI binary, root marker, side tooling, spec directory names, and spec prose.

The cutover is deliberately a single pass with no compatibility layer.
Nothing consumes the vocabulary yet — no installed tool, profile, autoenv file, or home-manager module reads `devtree` or the marker — so a hard cut costs nothing today, and doing it now is what keeps it cheap.

This work has priority over the specs that reference the old names.
Each of them inherits this vocabulary, so renaming after they land would mean rewriting them a second time.

Inspirations: the naming and boundary discussion of October 25, 2026; and the practice analysis in `README.md`, which observes two code-oriented trees holding roughly fifty repositories between them, planning kept in loose files, and no vocabulary for trees that are not code.

## Terminology & Key Concepts

**spacetree** (new!):
A rooted directory organizing one coherent domain into projects, documentation, task tracking, configuration, and supporting tooling.
Each spacetree defines its own local organization and may reference items that live in other spacetrees.
Its root is the directory carrying the `.spacetree-repo` marker.

**devtree** (updated!):
A spacetree whose projects are source-code repositories.
The term previously named any such tree and doubled as the general unit; this spec narrows it to the code-oriented case.
The narrowing is vocabulary only: nothing in the config, marker, resolution, scopes, or routing behaves differently because a tree is a devtree (see *Global Open Questions*).
It establishes no type system, no inheritance, and no capability profile.

**root marker** (replaced!):
The `.spacetree-repo` entry at a tree root, found by walking up from the current directory.
It replaces `.devtree-root` outright: no tool but the untouched `beads-devtree-init` draft reads the old name after the cutover.
Its form is settled by `<spec:spacetree-repo>`: a symlink whose target is the tree's spacetree repo.

**local organization**:
Where an item lives and how it is managed: the tree's own layout, naming, and conventions.

<!-- NOTE: Naming & IDs — omitted: no new identifiers are designed here; every identifier in scope is inherited from an existing artifact and listed in Rename surface. -->
<!-- NOTE: Interface / How to use — omitted: the spec introduces no CLI, config format, or API surface; the rename preserves existing interfaces under new names. -->

## Concept model

A spacetree is bounded by a domain, not by a type and not by a person.
The unit is the root directory: everything inside it belongs to that tree, nothing outside does.
What the tree holds — projects, documentation, tasks, configuration, tooling — follows from the domain, so the layout is local.

Boundaries are domain-specific.
No mandatory top-level container exists, so these are separate trees rather than branches of one personal tree:

- an administration tree, holding paperwork, deadlines, and reference documents;
- a home-maintenance tree, holding household systems, contractors, and recurring upkeep;
- a configuration tree, holding machine and tool configuration;
- a code tree, holding source repositories — a devtree.

The list is illustrative: no inventory of trees, and no set of directory names, is finalized here.

Work and personal are contexts a tree may be about, not containers it must sit in.

Each tree owns its organization systems: its planning and task tracking, its documentation, its projects, its configuration.
Ownership decides which system applies where, not which implementation is used.
Two trees may share a beads server, a doc generator, or a config format and still keep separate scopes and separate content.

Invariants the harness core must respect:

- Membership is decided by the root marker, never by the tree's contents.
- Core resolution, scopes, routing, and daemons assume no project shape: they never require a version-control repository, a `projects/` directory, or a build system.
- Tooling that is documented as devtree-specific — `wk`, environment activation, devshell integration — may assume repositories; the core still never asks what kind of tree it is in.
- A tree's planning, documentation, and configuration remain that tree's own, whatever backend they share.
- Calling a tree a devtree changes no behaviour today.

These invariants are intent, not enforcement: nothing checks at runtime for a code-shaped assumption.

## Concept documentation

`docs/what-is-a-spacetree.md` is the authoritative user-level description of the two terms.
Its reader is someone using the harness: no spec, no rationale for the naming decision, no migration narrative.

Content it carries:

- What a spacetree is, and that its root is the directory holding the marker.
- A brief why: one paragraph on the unit of organization and why a code-oriented case needed its own word.
- How boundaries work, with non-code examples, and that work and personal are contexts rather than containers.
- What each tree owns, including that shared backends do not merge scopes.
- What `devtree` means now, in one line, and that the core does not branch on it.
- The core-agnosticism statement: core tooling assumes no project shape; devtree-specific tooling may.

`README.md` keeps its overview, package inventory, practice analysis, and status, and replaces its definition of the tree with a link to the new document.
The definition is not duplicated in both files.

Neither `docs/beads-refs/*` (copied upstream reference) nor `docs/beads-init-complaints.md` (complaint record) is concept documentation; the new file stands alone in `docs/`.

## Rename surface

Every artifact renamed, in one table; what stays untouched follows it.

| Artifact | From | To |
|---|---|---|
| GitHub repository | `bew/devtree-harness` | `bew/spacetree-harness` |
| flake output | `devtree` | `spacetree` |
| tool source directory | `tooling/devtree/` | `tooling/spacetree/` |
| CLI binary | `devtree root --json` | `spacetree root --json` |
| root marker | `.devtree-root` | `.spacetree-repo` |
| HM module | `integrations/devtree-context-for-agents.hm-module.nix` | `integrations/spacetree-context-for-agents.hm-module.nix` |
| opencode plugin | `integrations/opencode-plugin/devtree-context.ts` | `integrations/opencode-plugin/spacetree-context.ts` |
| bats tests | `tooling/devtree/devtree.bats` | `tooling/spacetree/spacetree.bats`, following the binary |
| spec directories | `SPEC-READY-cross-devtree-task-routing`, `SPEC-DRAFT-devtree-config-repo`, `SPEC-READY-devtree-daemons` | same prefixes and slugs with `spacetree` |
| spec titles and prose | `devtree` naming any tree | `spacetree`, with `devtree` kept where the sentence is about code-oriented trees |
| active `TASK-*` note filenames | slugs containing `devtree` | slugs containing `spacetree`; `HANDOFF-*` filenames stay |

The prose rewrite follows one rule: `devtree` becomes `spacetree` unless the sentence is specifically about a code-oriented tree.
A blanket replacement is wrong, because it would erase the specialization the vocabulary is supposed to preserve.

Kept, with reasons:

- `beads-devtree-init` — a dead draft that never ran in a project; its name and internals are untouched, and its stale `.devtree-root` walk-up is recorded in `SPEC-READY-wk-init/`.
- Tree directory names (`~/self`, `~/work`) — the trees themselves are not being renamed, so nothing gains by renaming their directories.
- beads issue prefix and issue identifiers — the prefix is data, not vocabulary, and no live database depends on it.
- Dated `HANDOFF-*` notes — records of what was decided at a point in time; rewriting them falsifies the record.
- the spacetree repo's state branch, named `PROTECTED/spacetree-shared-state` — a branch name, not prose, and no beads setup exists to migrate it.

Outside this repository, nothing executes the vocabulary: no installed tool, shell profile, autoenv file, or home-manager module references `devtree` or the marker.
Documents elsewhere do name it, though: specs in the opencode dotfiles and in `wk-tools` refer to this repository and to its spec slugs by the old names, so they are swept in the same pass to keep those references resolving.
The other artifacts out there are two marker files in `~/self` and `~/work`, read by nothing; they are renamed in the same pass and now point at each tree's spacetree repo, because they record intent, not because a consumer needs the new name.

The `wkg` axis label and the `$XDG_STATE_HOME/devtree-global/` directory exist only in specs describing future tooling, so they follow the prose that names them and no deployed path changes.

## Migration sequencing

Hard cutover in one pass.
No alias, no dual-read, no release that understands both names.
The interval where tree resolution answers to neither name is inert, because nothing resolves a marker yet.

Ordered steps:

1. Rename the tool directory and the flake output together, in one commit: `tooling/devtree/` becomes `tooling/spacetree/` and the output attribute becomes `spacetree`.
   Splitting them leaves `nix build .#devtree` pointing at a missing path.
2. Rename the target files: the bats file, the HM module, the opencode plugin.
3. Rewrite spec prose and titles, last within the repository, so the text already refers to the paths that exist.
4. Rename the affected spec directories through `manage-spec rename`, which owns the directory name.
   Fall back to `git mv` only for a directory the tool does not own.
5. Rebuild and switch the Nix profile so `spacetree` is on `PATH`.
   Nothing on the machine runs it yet, so the old binary is simply dropped; it was never in a profile that anything depended on.
6. Rename the marker in each tree.
   Both files are empty and read by nothing, so this step cannot break resolution; it rides along in the same sitting only to keep the cutover a single pass.
7. Check that no state directory exists to move: `$XDG_STATE_HOME/devtree-global/` is named only in specs about future tooling, and no host has created one.
8. Rename the repository on GitHub and update the local remote.
   This can run at any point: GitHub redirects the old URL, so existing clones and links keep resolving.

Ordering constraints worth restating: the tool directory and the flake output move together; prose follows structural renames.
Nothing else is order-sensitive, because the vocabulary has no runtime consumer to sequence against.

This repository lives inside one of the trees being cut over, `~/self`, yet the migration cannot break its own environment for the same reason: no tool resolves that marker on startup.

Rollback is a `git revert` plus moving the marker files back.
Nothing is transformed in place, so no other state needs unwinding.

## Verification

The cutover is complete when a grep gate, a build, and one resolver call per tree all pass.

Grep gate inside the repository:

```sh
rg -n "devtree" --hidden --glob '!.git' .
```

Every remaining hit must match an exemption:

- `beads-devtree-init` — its name, its flake attribute, and the internals of that dead draft;
- dated `HANDOFF-*` notes;
- records of the rename itself: this spec's tables and alternatives, and the one line in the concept document stating the narrowing;
- a sentence where `devtree` deliberately names the code-oriented case.

Any other hit is leftover.

Grep gate outside it:

```sh
rg -l "devtree" ~/.dot ~/self/config-projects/dotfiles-alt ~/self/my-projects/wk-tools ~/work/.autoenv.zsh
```

Expected empty, followed by `~/self/.spacetree-repo` and `~/work/.spacetree-repo` existing with no `.devtree-root` beside them.

Build and tests:

```sh
nix build .#spacetree
bats tooling/spacetree/spacetree.bats
nix flake show
```

`nix flake show` lists a `spacetree` output and no `devtree` output.

Resolver, from inside each tree:

```sh
cd ~/self/my-projects && spacetree root --json
cd ~/work && spacetree root --json
```

Each prints one JSON object whose `name` is the name declared in the tree's spacetree repo and whose `path` is the tree root's absolute path.
The reported name follows that file, never the root directory's own name (see `<spec:spacetree-repo>`).
A call from a directory belonging to no tree exits non-zero with the `no marker found` error, unchanged from today.

Documents: `docs/what-is-a-spacetree.md` exists, `README.md` links to it, and no `docs/concepts.md` remains.

Negative checks, since a compatibility layer is the failure mode this design avoids:

- no second binary, wrapper, or shell function named `devtree`;
- no code reading `.devtree-root` outside the untouched legacy draft;
- no conditional accepting both marker names.

The beads issue prefix is not checked: it is out of scope and no live database uses it.

## Placement / Scope

This spec owns the vocabulary, the concept document, and the rename itself: which artifacts change, in what order, and how completion is proven.

It also settles the names the October 25, 2026 naming discussion left open — command, package, repository, and on-disk state directory — but only for the harness's own artifacts, on the user's later instruction.
It renames the harness's own artifacts and nothing else; no other tree's naming is decided here.

It owns no behaviour.
Every capability named here keeps its current semantics; only words and paths change.

Deferred, with the artifact that settles each; paths use their post-rename names:

| Topic | Settled by |
|---|---|
| marker form — empty file, symlink, or spacetree-repo pointer | `<spec:spacetree-repo>` |
| per-tree config schema, tree name field, variant field | same |
| which tools serve a tree's tasks and documentation | deliberately unspecified; the per-tree config settles any tool binding |
| machine-readable `devtree` designation | `<spec:spacetree-repo>`, where `variant` is descriptive metadata only |
| beads scopes, registry, routing | `SPEC-READY-task-scopes/` |
| daemons, out-of-tree beads storage | `SPEC-READY-spacetree-daemons/`, `SPEC-DRAFT-beads-out-of-tree-storage/` |
| CLI surface beyond the binary name | `TASK-spacetree-cli-spec-the-cli-surface.md` |
| `wk` wrappers and `wk init` | `SPEC-READY-wk-beads-wrappers/`, `SPEC-READY-wk-init/` |

The claim that the harness core stays spacetree-agnostic — resolution, scopes, routing, and daemons assuming no project shape — is advisory.
No test or code check enforces it.
It is a rule for reviewing future specs: a capability that only makes sense for a source-code tree belongs in devtree-specific tooling, not in the core.

The rename stops at names and paths.
It does not rewrite git history, and it does not touch beads data, the beads issue prefix, or the internals of `beads-devtree-init`.

## Alternatives & Tradeoffs

**Hard cutover** (chosen) against **marker dual-read plus a CLI alias**.
A transition release would read `.devtree-root` and `.spacetree-repo` and ship both binaries until a removal date.
It buys a downtime-free move for consumers that do not exist, and pays with a branch, an alias, and a deadline to remember.
With no consumer anywhere on the machine, the entire mechanism would be overhead from its first commit.

**Full rename** (chosen) against **concept-only rename**.
Concept-only would introduce `spacetree` in documents and leave `.devtree-root`, `devtree root --json`, and the repository name untouched.
The diff is a few files, but the defect survives: the artifacts would still use the general word for one specialization, which is exactly the confusion this work removes.
Vocabulary that contradicts the code is worse than the cost of one coordinated pass.

**Keeping `devtree` as the code-oriented specialization** (chosen) against **retiring the word**.
Retiring gives one term and no risk of misuse, at the price of making every sentence about a code tree generic, and losing a distinction the practice analysis shows is real.
Two terms cost ambiguity instead: a future author can still use `devtree` where `spacetree` is meant, and only the concept document and review catch it.

**Domain-specific boundaries** (chosen) against **one broad personal tree**.
A single personal tree gathering administration, home maintenance, and configuration would put one marker, one task system, and one documentation set over unrelated domains.
That was the shape an earlier proposal suggested, and it was rejected: the domains are handled separately, so a shared root would bind them with nothing shared.
The cost is more roots to create and maintain, each with its own systems.

Accepted costs of the choices above:

- a coordinated multi-file pass whose value is naming rather than function;
- no shipped revert path — undoing it means `git revert`;
- a permanently stale `beads-devtree-init`, recorded in `SPEC-READY-wk-init/`;
- spec directory renames break any external bookmark to those paths, of which none is known.

## Related artifacts

- `docs/what-is-a-spacetree.md` — created here; the user-level concept and its brief rationale.
- `README.md` — the project-global document, reduced to an overview that links to the concept document instead of defining the term itself.
- `<spec:spacetree-repo>` — owns the marker form, the per-tree config schema, and the open question about a descriptive `variant` field.
- `<spec:cross-spacetree-task-routing>` — renamed only; its prose follows the new vocabulary.
- `SPEC-READY-spacetree-daemons/`, `SPEC-READY-task-scopes/`, `SPEC-DRAFT-beads-out-of-tree-storage/`, `SPEC-READY-wk-beads-wrappers/` — directory and prose renames only; their designs follow the new vocabulary without changing.
- `<spec:wk-init>` — records why the legacy `beads-devtree-init` draft keeps the old naming in its internals.
- `tooling/spacetree/` — the CLI and its bats test, both renamed with the binary.
- `integrations/spacetree-context-for-agents.hm-module.nix`, `integrations/opencode-plugin/spacetree-context.ts` — the agent-context module and plugin, renamed and unregistered today.
- `TASK-spacetree-cli-spec-the-cli-surface.md` — the deferred CLI surface work, renamed with the other active task notes.

No artifact outside this repository points at these names; the markers are the only external traces.

## Global Open Questions

1. Whether `devtree` ever gains a machine-readable designation.
   No, for now: `<spec:spacetree-repo>` defines `variant` as descriptive metadata that changes no resolution, so the distinction stays vocabulary.
   A field that did change behaviour would be a new question for that spec.
