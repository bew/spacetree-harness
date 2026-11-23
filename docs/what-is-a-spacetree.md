# What is a spacetree

A spacetree is a rooted directory that gathers everything for one coherent domain: its projects, documentation, tasks, configuration, and supporting tooling.
The tree's own structure is its business — no layout is imposed.

Its root is the directory carrying the `.spacetree-repo` marker, a symlink to the tree's own git repo.
Tooling walks up from wherever you are to find that marker, and so locates the enclosing tree.
The tree's shared material — agent instructions, skills, docs, scripts — is reached through a second, optional marker, `.spacetree-shared-root`, whose target is the `tree-shared/` directory inside that repo.
The repo is where the tree keeps what it owns: its configuration and that shared material, versioned like any other project.

A tree also has a name, and the name is stable: it does not follow the root directory's name or its location.
Renaming or moving the root directory leaves the tree's name untouched, so tasks, notes, and tooling that refer to the tree keep working.
The name is declared in the tree's configuration, in the repo's `spacetree.toml`.

## Why the word exists

The unit of organization used to be called a *devtree*, a name that assumed code.
Not every domain is code: home maintenance, administration, and reference collections organize the same way and have no build to run.
`spacetree` names that general unit, and `devtree` names its code-oriented case.

## Boundaries

A tree's boundary comes from its domain, not from who owns it.
There is no single personal tree: home maintenance, administration, and personal software each get their own, because each is organized differently.
Work and personal are contexts a tree can belong to, not containers it must sit inside.

Trees may reference each other, and such a reference is an ordinary link rather than a hierarchy.
Keeping an item local to one tree and linking it from another is the normal way to cross a boundary.

## What a tree owns

Each tree owns its projects, its planning and task system, its documentation system, and its configuration.
Two trees may share a backend for one of those, and even then each keeps its own scope: a shared task store holds separate task sets per tree, never one merged set.

## Where `devtree` fits

`devtree` names a spacetree whose work is code: repositories, development environments, build and test tooling.
Calling a tree a devtree is vocabulary and nothing more: no type system, no inheritance between trees, no capability profile, and nothing in the harness branches on the word.

## What the core assumes

Core harness tooling — root resolution, task scopes, routing, daemons — assumes no project shape, so it behaves the same in a tree of documents as in a tree of repositories.
Tooling documented as devtree-specific, such as project environment activation or development shells, may assume repositories.
