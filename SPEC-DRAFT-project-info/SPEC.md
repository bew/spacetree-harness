# [DRAFT] Project Info

> IMPORTANT: Before any drafting/planning/editing of this spec,
> agents MUST load one of the spec-writing skills first.

## Introduction

Every project in a spacetree is an independent git repo, but its identity is implicit.
It is either the directory name or the beads `issue_prefix`, and both change for reasons unrelated to the project: a directory rename, a re-cloned upstream, a prefix collision.
A git worktree makes the point concrete: the same project routinely appears under several directories (`.worktrees/<branch>`, `feature-x/`), and none of those names is the project's name.
Nothing at the project level says which external ticket system it belongs to, where its docs live, or which scheduler manages it.

Two needs force the issue.
Out-of-tree beads storage keys every workspace by project, and the cross-spacetree routing layer addresses projects by name; both want a stable identifier that is not a filesystem accident.
Project-level metadata (ticket system, docs system, external references) has no home at all today.

This spec introduces a single small file at the project root, `project-info.toml`.
Where the repo is ours to modify, the file is committed like any other config and reviewed as a diff.
Where the repo is not ours (an open-source clone), a dot-prefixed `.project-info.toml` symlink points at the same TOML inside the spacetree config repo, and git never sees it.

The project info is human-first: ordinary TOML, readable and greppable, edited by hand, with no daemon, no database, and no generated cache behind it.
Harness tooling reads it on demand and ignores sections it does not understand, so the file can carry project-specific extras without a schema change.

## Terminology & Key Concepts

**project info** (new!):
The TOML file that carries a project's key and project-level metadata; the subject of this spec.

**tracked project info** (new!):
A `project-info.toml` committed in the project repo, used when the repo is ours to modify.

**linked project info** (new!):
A `.project-info.toml` symlink in the project repo, pointing at the project info store; used when the repo is not ours, and never seen by git.

**project key** (new!):
The stable project identifier declared by the project info; the name the routing layer, the registry, and state directories use.

**beads prefix** (existing):
The `issue_prefix` beads uses for issue IDs, such as `ghh-1a2b`; owned by the project info, but only beads consumes it.

**project info store** (new!):
The `project-infos/<key>.toml` directory inside the spacetree config repo, defined by the spacetree-config-repo spec.

**config repo** (existing):
The per-spacetree git repository the `.spacetree-root` symlink points at; defined by the spacetree-config-repo spec.

## Naming & IDs

Two filenames exist, and exactly one location: the project repo root.

- `project-info.toml` — tracked project info, committed with the repo.
- `.project-info.toml` — linked project info, a symlink into the config repo.

**Project key** grammar: `[A-Za-z0-9][A-Za-z0-9-]*`.
It addresses the project, so it must survive renames of the directory, the repo, and the beads prefix.
Case is preserved, but uniqueness within a spacetree is enforced case-insensitively, because state directories may land on a case-insensitive filesystem.
Uniqueness across spacetrees is not required, because the routing grammar qualifies the key with the spacetree.

**Beads prefix**: beads' own `issue_prefix` grammar, supplied by the project info and not interpreted by the harness.
It is a separate identifier from the project key and may differ from it (`key = "greenhouse-hub"`, `prefix = "ghh"`).

Resolution order, evaluated at the repo root of the current checkout:

```pseudocode
if both project-info.toml and .project-info.toml exist -> error (ambiguous)
if project-info.toml exists                            -> tracked project info
if .project-info.toml exists                           -> linked project info
if the checkout is a linked worktree                   -> repeat this order at the main worktree root
otherwise                                              -> error (no project info)
```

There is no upward search: a nested repo resolves its own file, and a repo without one is not addressable, even if a parent repo has one.
The worktree fallback is not a search either: `git rev-parse --git-common-dir` names the same repository's main checkout, so a project info found that way describes the very same project, and resolution reports it as inherited from the main worktree.
Sharing a parent's identity is instead an explicit opt-in: a nested checkout may create `.project-info.toml` pointing at the parent's `project-info.toml`, which gives it the parent's key and therefore the parent's beads workspace.
That is the same outcome as inheriting by search, except that it is visible in the repo and appears in a diff.

For a linked project info, the symlink conventionally targets `<config repo>/project-infos/<key>.toml`, and the store filename equals the key declared inside the file.
Resolution does not enforce that location: a symlink pointing elsewhere still resolves, and then only the file content is read.
A store filename that disagrees with the key declared inside it is an error rather than a silent preference, since it means two identities now name one file.
That check applies only to files inside the store: a link to another repo's `project-info.toml` carries a filename that says nothing about the key.

## Interface / How to use

A complete project info:

```toml
# Project info for greenhouse-hub: identity, plus where its tickets and docs live.

key = "greenhouse-hub"

[beads]
prefix = "ghh"

["tickets:external"]
system = "github"          # github | jira | none
ref = "bew/greenhouse-hub" # owner/repo, JIRA project key, or omitted when system = "none"

["docs:external"]
system = "confluence"
ref = "space/ENG"          # space key, URL slug, or omitted when system = "none"
```

A project info opens with a one-line comment saying what the file is and what it is for; comments are for humans and nothing parses them.

The core is those sections; `key` is required and the rest are optional.
A project without `[beads]` is still tracked for tickets and docs, but has no beads workspace, and any beads command fails with `project has no beads prefix` rather than creating one.
Section names containing `:` must be quoted in TOML, as the example shows.
Any other top-level table is preserved and ignored by the harness, so a project can carry its own sections without a schema change.
A project info is edited by hand after creation; `wk init` is its only writer, and nothing rewrites an existing one.

`wk init` (`<spec:wk-init>`) sets one up interactively and asks who owns the repo, which decides the form:

```console
$ wk init
  Project info for "greenhouse-hub"
  origin: git@github.com:bew/greenhouse-hub.git
  detected: not yours (gh: no push access)
  Is this repo yours?
  [m] me    — commit project-info.toml to this repo
  [o] other — symlink .project-info.toml to the spacetree config repo
  owner [m/o]: o
```

Ownership detection answers one question — do you have push access to the repo at `origin` — with three possible results: `mine`, `other`, or `unknown`.

```pseudocode
if origin is on github.com and gh is available and authenticated:
    gh api repos/{owner}/{repo} --jq .permissions.push -> mine when true, other when false
else if origin is on github.com and `git config github.user` is set:
    origin owner equals github.user -> mine, otherwise other
else:
    unknown
```

The presence of a remote proves nothing about ownership: a private repo of yours and someone else's look identical from the remote alone.
Detection only chooses the suggested default — the interactive prompt always asks.
The prompt reports which rung produced the answer, as the `gh: no push access` line does above.
`unknown` means no default is offered and the choice must be made without one.
Only an interactive run may call `gh`; a non-interactive run makes no network call and requires the flag.

```sh
wk init --owner=me
wk init --owner=other
```

Creating a linked project info writes the store file at `<config repo>/project-infos/<key>.toml` if it does not exist yet, creates the `.project-info.toml` symlink, and adds that filename to `.git/info/exclude`.
Creating a tracked one only writes `project-info.toml`, and leaves exclusion alone.
Neither form touches an existing project info, and `wk init` refuses when both files are already present.

Doing it by hand is equally supported, which is the human-first path:

```sh
# tracked
$EDITOR project-info.toml
git add project-info.toml

# linked, in a repo you do not own
ln -s ~/self-config/project-infos/greenhouse-hub.toml .project-info.toml
printf '.project-info.toml\n' >> .git/info/exclude
```

A nested repo that shares its parent's identity is the same operation with another target:

```sh
ln -s ../project-info.toml .project-info.toml
```

## Resolution & Identity

The project key is the addressing identity.
Everything that names a project uses it: `wkg <spacetree>:<key>:<scope>`, the project registry, `external:<key>:<cap>` references, and `<spacetree>/.state/beads-by-project/<key>/`.
`external:<key>:<cap>` follows the beads syntax and is currently inert: `bd` stores the edge without resolving it, and the grammar and resolution are open in `<spec:cross-spacetree-task-routing>`.
The beads prefix is an attribute, consumed only by beads, and renaming it never moves a workspace or changes an address.

Identity never derives from the environment.
The directory name, the git remote, the worktree branch, and the beads prefix are all ignored as identity sources, which is why a worktree checkout under `.worktrees/feature-x` resolves the same key as the main checkout.

The repo root is the boundary of a project, and one key may legitimately appear in several checkouts: worktrees, re-clones, and an explicitly linked nested repo all resolve to the same project and therefore the same workspace.
That is the point of linking a nested repo, and it is also the reason a second independent clone is not a second project.

Resolution failures are distinct from validation failures, so callers can tell them apart:

- no `project-info.toml` and no `.project-info.toml` → refuse: `no project info in <repo root>`;
- both files present → error: `ambiguous project info`;
- unreadable or invalid TOML, or a missing `key` → error naming the file and the field;
- linked store file whose name disagrees with its `key` → error: `project info store name does not match key`.

A project info without a `[beads]` section is valid, but unaddressable by beads commands, which fail with `project has no beads prefix`.

Uniqueness is enforced at creation, not at resolution: `wk init` consults the spacetree registry and refuses a key already declared by a different repo path.
There is no override flag; a repo that moved is not a case `wk init` handles.
Resolution itself never scans, so the check costs nothing on the command path.

## Placement / Scope

A project info lives at the repo root and nowhere else: not in a subdirectory, not in a parent, not in a global `XDG` location.
The repo root is the unit the routing layer addresses, so one file per repo keeps identity and addressability the same question.

The linked form is excluded through `.git/info/exclude`, never through the repo's own `.gitignore`.
The distinction matters: `.git/info/exclude` is per-clone and untracked, so linking a project info cannot produce a diff in a repo someone else owns, while a `.gitignore` entry would.

The project info store (`project-infos/<key>.toml`) lives in the spacetree config repo, so linked project infos are versioned and reviewed there rather than in the repos they describe.
The harness never writes to a tracked project info; `wk init` creates files, and every later edit is a human edit.

In scope:

- Both filenames, their forms, and the exclusion rule.
- The core TOML schema and the preservation of unknown sections.
- The answers `wk init` collects for the core schema, and the rules for creating either form.
- Resolution semantics and their failure modes.

Out of scope:

- The `wk init` command itself — its flags, prompts, and ordering belong to `<spec:wk-init>`.

- The config repo itself and how its path is found (spacetree-config-repo spec).
- The beads workspace (beads out-of-tree storage spec).
- Registry indexing and routing grammar (cross-spacetree task routing spec).
- Any other spacetree-wide setting that might later land in `spacetree.toml`.

## Alternatives & Tradeoffs

The simplest viable alternative is no project info at all: derive the project key from the directory name, and read tickets/docs from the git remote.

It costs nothing to implement and needs no file in any repo.
It also breaks exactly where the design is needed: a worktree checkout (`feature-x/`) and a re-clone produce different keys, an open-source clone has no usable metadata, and a project cannot state its docs or ticket system at all.

The chosen design pays one small file per project to make identity explicit.
Tracked project info keeps identity in the project's own history, where it is reviewed and survives a fresh clone.
Linked project info covers the repos we do not own, without creating a diff, at the cost of depending on the spacetree config repo being present.

**Always symlink into the config repo.**
Uniform, and every project info is versioned in one place.
Costs: repos we own no longer state their own identity, a clone outside the spacetree has no identity at all, and the config repo becomes a single point of failure for every project.

**Always tracked, never linked.**
The most portable and the simplest rule to explain.
Costs: it is impossible in a repo someone else owns, where even an untracked file would have to be excluded by hand, and it makes the harness unusable for open-source trees.

**A spacetree-wide registry keyed by path.**
No file in any repo; one TOML maps repo paths to keys, ticket systems, and docs.
Costs: paths go stale on every move and clone, the metadata is invisible from inside the project, and every worktree needs its own entry.

Decision criteria: use the tracked form whenever the repo is ours to modify, and the linked form otherwise.
Prefer deriving nothing from the environment; a wrong key that silently shares or splits a workspace is worse than a refusal that asks for one line of TOML.

## Related artifacts

- `<spec:spacetree-config-repo>` — where the config repo and the `project-infos/` store live.
- `<spec:wk-init>` — the bootstrap command that creates a project info and a beads workspace.
- `<spec:beads-out-of-tree-storage>` — the workspace keyed by the project key.
- `<spec:cross-spacetree-task-routing>` — the registry and the `spacetree:project:scope` grammar that consumes the key.
- `README.md` — spacetree definition and the project-metadata gap this spec fills.
- `bd where --json` — beads' own idea of a workspace identity, useful as prior art for what a project needs to answer.
- `SPEC-DRAFT-project-info/DISCOVERY.md` — the discovery notes behind this design, including the cases that were rejected.

## Global Open Questions

**Terminology & Key Concepts** (TKC) — Whether this section is needed for this spec.
Non-blocking. Refer to spec-writing skill for guidance.

**Secrets and environment values** — Whether a project info may reference credentials or environment-specific values (e.g. a JIRA token source).
Non-blocking. The current core deliberately holds only identity and pointers, since secrets belong in a secret manager rather than a tracked TOML.

**Registry timing** — When the spacetree registry learns about a project: on `wk init`, on first resolution, or on a registry refresh.
Non-blocking. The routing spec owns the registry; this spec only requires that duplicate keys be refused at creation.
