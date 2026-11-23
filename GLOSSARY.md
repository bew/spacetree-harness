# Glossary

Scope: the spacetree harness — tree identity and the projects a tree owns.

What each thing in this scope is called, and what that name means.

**Spacetree**, **Spacetree root**: a rooted directory gathering everything for one coherent domain — its projects, tasks, documentation, and configuration.
Its root is that directory itself, found by walking up to the `.spacetree-repo` marker and never the spacetree repo's path, while its name comes from that repo's `spacetree.toml` rather than the directory, so moving the root changes neither.
_Code_: `.spacetree-repo`, `spacetree.toml`, `spacetree info`

**Spacetree repo**: the one git repository per spacetree holding everything the spacetree itself owns — configuration at its root, inherited content under `tree-shared/`.
The `.spacetree-repo` marker points at it, so its location is a fact the marker carries rather than a name the harness assumes, and the harness reads it without writing to a branch the human owns.
_Code_: `.spacetree-repo`, `spacetree.toml`, `related-trees.toml`, `projects/`, `tree-shared/`

**Shared root**: the directory inherited content is reached through, conventionally the spacetree repo's `tree-shared/`, named by the optional `.spacetree-shared-root` marker.
When that marker is absent or cannot be honoured the shared root is unbound (`null`) rather than the spacetree root; a consumer resolves the marker and stops, never reading `spacetree.toml` to find it.
_Code_: `shared_root`, `tree-shared/`, `.spacetree-shared-root`

**Project**: one body of work a spacetree owns, identified by a project key that survives directory renames, re-clones, and worktrees.
The repo root is its boundary, so every checkout of that repo is still the same project, and no two projects in one spacetree share a key.
_Code_: `project-info.toml`, `.project-info.toml`, `key`

**Domain**: the coherent area of life or work a spacetree organizes.
A tree takes its boundary from its domain rather than from who owns it, so two trees never share one merged set of tasks or documents.
_Avoid_: scope

**Tree**: a rooted directory hierarchy; a spacetree, a project's repo, and a group of projects are all trees.
Say spacetree when the tree is a spacetree, so that a bare `tree` only appears where the context already fixes which tree is meant.
