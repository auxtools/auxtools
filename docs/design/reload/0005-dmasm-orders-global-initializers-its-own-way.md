# How dmasm orders global and static initializers

[The global init proc](0004-a-reboot-after-a-reload-runs-dmasms-global-init-proc.md) must run
global and static variable initializers in an order where any variable, type, or proc an
initializer reads is ready first. Because BYOND's own ordering is complicated, dmasm computes its
own order from the source declarations rather than porting BYOND's algorithm or reading the build's
order.

The goal is a reasonable order that matches on checked codebases rather than an exact copy of
BYOND, so dmasm's order may differ from a real build in cases not yet checked.

## Source ordering rules

DreamMaker reads variables declared in the block of a single `var` line as a tree where entries
starting with the same path share a node. For example, `list/a`, `b`, and `list/c` under
`var/global` are visited as `a`, `c`, `b`, whereas declarations on separate `var` lines are visited
in source order. dmasm builds its initializer order as follows:

- **Include chains:** Each file's position is the chain of include lines that first reaches it,
  from outermost to innermost. An include resolves against the including file first and then
  against the `.dme`, which has an empty chain. Files with no located include come after all
  others, in first-read order. Every file is placed in first-read order when the snapshot has no
  project directory or when the order is built without proc statics (`initializers` with `procs`
  set to `false`).
- **`var` blocks:** Within a `var` block, each declaration is sorted by the line of the `var`, then
  the first line of each tree node along the path to it, then its own line. Because the parsed tree
  does not keep block structure, dmasm recovers it by rereading the source lines.
- **Absolute root variables:** Root variables declared on a line starting with `/var/` come before
  all other declarations, in path-tree order. This is also determined from the source line because
  the parsed tree does not record the leading slash.

## Dependency walk and late passes

1. As the type tree is walked, each initializer runs in place if the procs, types, and values it
   reads are ready, and otherwise waits.
2. At the end of the walk, waiting initializers of absolute root variables get the first pass.
3. Next come the built-in types visited after user source, in this order: `/sound`, `/icon`,
   `/matrix`, `/database`, `/exception`, `/regex`, `/mutable_appearance`, `/dm_filter`,
   `/generator`, and `/particles`.
4. The remaining waiting entries then get passes.
5. When a pass makes no progress, one final pass runs. If that pass also makes no progress, the
   remaining initializers run in the order found, which settles any cycle.

## Applies to

- `dmasm/reload/src/reload/global_init.rs`: `Walk`, `Walk::finish`, `Walk::absolute`,
  `Walk::absolute_order`, `include_chains`, `var_blocks`, `LATE_TYPES`, `visited_late`
