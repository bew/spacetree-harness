# routing-vs-aggregate — exploration

## Findings

- Gas City's `gc bd --rig <name>` is explicitly "just routing sugar" (chdir + exec), with no cross-rig joining.
- `bd` hard-scopes every query by `issue_prefix` and refuses cross-project reads, so any cross-project view must be built outside `bd`.
- Cross-project *linking* is already native: `bd repo add` + `bd repo sync`, and `external:<project>:<capability>` dependency targets.
  Correction: neither is native in this model. Verified against bd 1.3, `external:` targets are stored as edges with no row in the local database, so they never block; `bd repo` hydrates other workspaces into one database and conflicts with one workspace per project. Both are unresolved — see the *Cross-project dependency references* entry in `<spec:cross-spacetree-task-routing>`.
- One Dolt server per spacetree (shared-server topology) means a per-spacetree SQL join across prefixes is technically reachable, but it would couple the routing layer to that topology.

## Design crux

Does `bdg`/`bdt` stay an address-and-route layer, or also own a cross-project aggregate path?

## Candidate directions

- (settled) **A — pure routing.** `bdg`/`bdt` resolve a target, move to it, and exec `bd`. One command, one project.
- (rejected) **B — per-spacetree aggregate** (`bdg all …`). Would make the tool a query engine and couple it to the Dolt topology.
- (rejected) **C — cross-spacetree aggregate.** Adds a federation/merge layer on top of B; heaviest.

## Decisions

- (settled) **Pure routing only.** No aggregate/reporting command is part of `bdg`/`bdt`.
  Cross-project reporting, if ever wanted, is a separate concern.
- (settled) **Consequence: the registry carries no Dolt connection info.** Since there is no aggregate path, `spacetree-global/` maps names to workspaces only; server sockets stay the business of the beads shared-server wrapper.

## Open threads — none
