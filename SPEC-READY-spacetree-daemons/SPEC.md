# [READY] Spacetree Daemons

## Introduction

Work inside a spacetree often needs local long-running services — the beads Dolt server, prefect, and similar — that should be reachable while working in that tree.

Today there is no spacetree-scoped way to start, observe, and stop such services.
They are started ad hoc, their logs are scattered, and nothing ties their lifetime to the tree they serve.

The harness is human-first: any capability must work manually, with agent support as a layer on top.
The daemon feature therefore aims at a human at a shell in a spacetree root, not at an agent.

The chosen backend is **process-compose (PCS)**, a single static binary that supervises a declarative process set and exposes start/stop/status/logs through a client/server model.
Rather than reimplement supervision, the `spacetree` CLI wraps PCS as `spacetree procs` (with `procs-ensure` and `procs-attach` companions), forwarding its arguments, so the human learns one entry point and inherits PCS's own client commands.

This spec covers the interface and lifecycle of that wrapper: where the per-spacetree PCS server and its socket live, how process sets are grouped, and how a set is ensured idempotently.
It is scoped to long-running processes on macOS, with minimal supervision (start/ensure, logs, stop, status) and no restart policy or healthchecks.
Defining the process tree from Nix is wanted but deferred; the interim config is a plain `process-compose.yaml` at the spacetree root.
Implementation is future work; this document fixes the design.

## Terminology & Key Concepts

- **PCS** — process-compose, the supervision backend; a single binary that is both server and client.
  - **PCS server** — the detached PCS process that owns a project's processes for one spacetree.
  - **PCS socket** — the Unix domain socket the client uses to reach that server.
- **process set** (or **set**) — a named group of long-running processes owned by one spacetree; maps 1:1 to a PCS **namespace**.
- **ensure** — to make a process set running if it is not, idempotently, without disturbing an already-running set.
- **wrapper** — the `spacetree procs` command family, which resolves the spacetree root and delegates to PCS.

## Naming & IDs

- **Spacetree marker**: `.spacetree-root`, a symlink whose target is the spacetree config repo (see `<spec:spacetree-config-repo>`); root resolution resolves that target, and the spacetree root is the directory holding the symlink.
- **Init command**: `spacetree init` — scaffolds the spacetree, creating the `.spacetree-root` symlink to the config repo, the state branch worktree at `.state/beads-by-project/`, and a basic `process-compose.yaml` when none exists.
- **Wrapper command**: `spacetree procs [args…]` — with no arguments it prints a status overview of the spacetree's processes (initially by delegating to a PCS status subcommand); with arguments, every argument after `procs` is forwarded verbatim to PCS.
- **Ensure command**: `spacetree procs-ensure <namespace>` — takes one positional argument, the process set to ensure.
- **Attach command**: `spacetree procs-attach` — attaches the interactive TUI to the running server; quitting the TUI does not stop the server.
- **Process set / namespace**: named by the config author, kebab-case; the name is used verbatim as the PCS namespace.
- **Config file**: `process-compose.yaml`, fixed, at the spacetree root.
- **Socket path**: `<spacetree root>/.state/process-compose.sock`, fixed, one per spacetree.

## Interface / How to use

The `spacetree` CLI exposes the process machinery through a small subcommand family: `procs`, `procs-ensure`, and `procs-attach`.

`spacetree procs [args…]` is a passthrough to PCS.
It resolves the enclosing spacetree root, sets the working directory to that root, passes the spacetree socket with `-u`, and execs PCS with any remaining arguments; with no arguments it prints a status overview instead.
It never starts the PCS server itself.
Because the working directory is the spacetree root, PCS auto-discovers `process-compose.yaml`, so no explicit config path is passed.

Example invocations of `spacetree procs …`:

- `spacetree procs` — status overview of the spacetree's processes.
- `spacetree procs up -D` — start the server with the whole config, detached.
- `spacetree procs up -D -n web` — start the server with only the `web` set.
- `spacetree procs process logs web` — print logs for one process.
- `spacetree procs process stop web` — stop one process.
- `spacetree procs down` — stop the server.

`spacetree procs-ensure <namespace>` is the idempotent primitive; other spacetree commands call it to guarantee a set is running.
If the socket is not live, it starts the server scoped to that namespace (`up -D -n <namespace>`).
If the socket is live, it issues `namespace start <namespace>`, which is a no-op when the set is already running.
It returns only once the set's processes are up.

`spacetree procs-attach` attaches the interactive TUI (to watch logs and inspect the running processes).
Quitting the TUI detaches the client and leaves the server and its processes running.

`process-compose.yaml` at the spacetree root declares the processes, each assigned to a namespace.

`spacetree init` scaffolds a spacetree: it creates the `.spacetree-root` symlink to the spacetree config repo in the current directory, checks out the state branch at `.state/beads-by-project/`, and writes a basic `process-compose.yaml` if none exists.
Creating the symlink and the worktree requires a config repo, so init either creates one or refuses when none exists (see *Global Open Questions* in `<spec:spacetree-config-repo>`).
It never overwrites an existing config, so it is safe to re-run.

## Lifecycle & State

One PCS server serves each spacetree, owning all process sets in that tree.
It listens on the fixed socket `<spacetree root>/.state/process-compose.sock`, which is what `spacetree procs` and `spacetree procs-ensure` address.

The server starts detached (`up -D`), either by an explicit `spacetree procs up …` or by `spacetree procs-ensure` when the socket is not live, and then outlives the shell that started it.
The wrapper itself never starts the server, so a plain `spacetree procs process list` on a spacetree with no server fails as PCS would.

A running server is observed with `spacetree procs-attach` and stopped with `spacetree procs down`.
Its lifetime is independent of any single set: stopping one set (`namespace stop`) leaves the server and other sets running.

Because the socket path is fixed, liveness is decided by whether the socket answers, not by the presence of the socket file.

## Process Sets as Namespaces

The wanted "sets of processes" map 1:1 onto PCS **namespaces**.
The spacetree runs one PCS server, and every set is a namespace inside it.

Each process declared in `process-compose.yaml` carries a `namespace:` field naming its set; that single field turns a flat list of processes into named sets.

Namespaces are flat, not nested: a set belongs directly to the spacetree, and there is no sub-set hierarchy.
Sets are the unit of grouping for start, stop, and ensure, and they coexist under the one server; grouping buys independent lifecycle, so `spacetree procs namespace stop web` stops only the `web` set and `spacetree procs-ensure web` guarantees only `web`.

One server with namespaced sets is preferred over one server per set: it keeps a single socket and state directory per spacetree, while PCS's namespace primitive already supplies the per-set control.
A server-per-set split would multiply sockets and invite orphaned servers without adding capability.

## Placement / Scope

The PCS config lives at the spacetree root as `process-compose.yaml`; `spacetree init` seeds it there.
The root is the fixed location because PCS auto-discovers the file from the working directory, and `spacetree procs` always runs from the root.

Runtime state lives under the spacetree's `.state/` directory, with the socket at `<spacetree root>/.state/process-compose.sock`.
`.state/` is a spacetree-wide convention, created lazily by whatever tooling needs it.
`.state/beads-by-project/` is the one versioned exception: it is a git worktree of the config repo's state branch, not throwaway runtime state (see `<spec:beads-out-of-tree-storage>`).

The feature targets macOS only for now.

In scope: the `spacetree` process command family (`procs`, `procs-ensure`, `procs-attach`), `spacetree init`, the config contract, and the server lifecycle.
Out of scope: defining the process tree from Nix (deferred), restart policies and healthchecks, one-shot processes (long-running only), alternative supervision backends, and cross-spacetree coordination.

One placement wrinkle: the spacetree root is not itself a git repo, so a hand-written `process-compose.yaml` placed there is not version-controlled by default.
That is accepted for now; moving the file into the config repo, which is versioned, is an open question in `<spec:spacetree-config-repo>`.

## Alternatives & Tradeoffs

**Chosen — PCS wrapper, one server per spacetree, sets as namespaces.**
Grouping is native (namespaces), status and logs are machine-readable via the PCS API, it ships as a single static binary, and servers are socket-scoped so each spacetree stays isolated.
Costs: a third-party dependency and a thin wrapper to fix cwd and socket per spacetree.

**Simplest — no wrapper, run PCS directly.**
Fewer moving parts, and PCS already has a capable CLI.
But the user would have to set the socket path and working directory per spacetree by hand, and there would be no spacetree-scoped `init` or `procs-ensure` primitive for other tooling to call.
The wrapper exists to centralize exactly those three things.

**tmux server per spacetree (the original idea).**
Ubiquitous and interactive; attach is a first-class tmux feature.
Costs: process and log state is only parseable from pane text, there is no native grouping or status API, and programmatic start/ensure would mean scripting tmux commands per pane — brittle and log-unfriendly.
It is weaker precisely where the spec needs guarantees.

**launchd user agents.**
Native macOS supervision and restart handling.
But plists are verbose, state is not groupable per set, logs go through `log` rather than per-process files, and it is not naturally spacetree-scoped.
It also brings supervision features this spec explicitly excludes.

Only the no-wrapper option is genuinely simpler; PCS wins on grouping and scriptability, tmux loses on machine-readable state, and launchd on ergonomics and scope fit.

## Related artifacts

- **Existing spacetree CLI** (`pkgs/spacetree/spacetree`) — already resolves the enclosing spacetree by walking up for the `.spacetree-root` marker; `spacetree procs` and `spacetree init` build on that same resolution, extended to follow the marker symlink.
- **Spacetree context plugin** (`integrations/opencode-plugin/spacetree-context.ts`) — resolves the marker for agent support; shares the spacetree-root notion this spec relies on.
- **Spacetree config repo spec** (`<spec:spacetree-config-repo>`) — defines the marker symlink, the config repo, and the state branch that backs `<spacetree>/.state/beads-by-project/`.
- **Beads shared-server handoff** (`HANDOFF-20260920-beads-shared-server-devtree.md`) — the prior decision to run one Dolt server per spacetree under `<spacetree>/.state/`; this spec generalizes that single-server-per-spacetree pattern to PCS.
- **process-compose upstream docs** — the source for the config schema (`processes`, `namespace`, `up -D`) and the client subcommands the wrapper forwards.
- **Exploration files** in this directory (`EXPLO-BRIEF.md`, `EXPLO-backend.md`, `EXPLO-pcs-wrapper.md`, `EXPLO-nix-process-tree.md`) — record the backend and interface discussion this spec condenses.
- **`README.md`** — lists the spacetree config, tree-scoped definitions, and `.state/`-style gaps this feature sits beside.

## Global Open Questions

**Nix-managed process tree** — Where Nix definitions of the process tree live and how they are rendered into PCS config, instead of a hand-written `process-compose.yaml`.
Non-blocking. Deferred by the user; interim config is plain YAML at the spacetree root.
