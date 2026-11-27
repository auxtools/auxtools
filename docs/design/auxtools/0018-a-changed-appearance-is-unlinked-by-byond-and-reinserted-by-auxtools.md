# Unlinking and reinserting changed appearances

BYOND stores appearances in a red-black tree ordered by its appearance comparator, threaded with
links from each node to the next smaller and next larger node. When a resource reload changes an
appearance's icon or mouse pointer, that appearance's position in the tree order changes, but
BYOND's insert function always allocates a new slot and cannot relocate an existing entry. auxtools
therefore unlinks modified nodes using BYOND's own functions and reinserts them directly.

## Tree layout and BYOND functions

BYOND inserts a blank node with ID `0xFFFF` into the tree while leaving it out of the ID table. Each
appearance tree node has the following layout:

| Offset | Field |
|---|---|
| 0x00 | left child |
| 0x04 | right child |
| 0x08 | parent |
| 0x0C | next larger node |
| 0x10 | next smaller node |
| 0x14 | id |
| 0x18 | colour (u16): 0 red, 1 black, 0xFFFF a free slot |
| 0x30 | icon |
| 0x38 | misc id of the overlay appearance-id list |
| 0x3C | misc id of the underlay appearance-id list |
| 0x70 (515), 0x74 (516) | mouse pointers: three values, each a tag byte with its data at +4 |

Starting in `516.1648`, `icon_w` and `icon_z` appear before the mouse pointers. In the appearance
table itself, the tree header is at `0x14` and holds the root, the smallest node, and the largest
node. The ID table pointer is at `0x40` and its capacity is at `0x44`.

BYOND's unlink and rebalance functions are located through the code that passes the string
`"unique atomic images"` to a call. The next call after that invokes the function that releases an
appearance's slot, which calls unlink first and rebalance afterward:

- **Windows (all builds):** rebalance is the slot-release function's second call.
- **Linux before `515.1644`:** rebalance is its second call.
- **Linux from `515.1644` (including late 515 builds):** rebalance is its fourth call.

Both functions take `(tree, node)` and return a node, using `cdecl` on Linux and `thiscall` on
Windows. If the two recipes resolve to the same function, resource reload is refused.

## How `repoint` updates the tree

`repoint` updates changed appearances in six steps:

1. Records which appearances show each changing appearance as an overlay or underlay, reading them
   from live nodes (nodes whose color is not free) before any appearance changes.
2. Unlinks each changing node with BYOND's unlink function. When the node returned by unlink is not
   red, passes that node to BYOND's rebalance function.
3. Writes the new icons and mouse pointers.
4. Recomputes bounds using BYOND's function for the changed appearances and for every appearance
   that displays one of them, directly or through other overlays and underlays. Appearances that
   display a changed one remain linked in the tree during this step. Using the users recorded in
   step 1, each appearance is ordered after the overlays and underlays it displays. When overlays
   and underlays form a cycle, that topological order cannot be completed, so the remaining
   appearances are appended in discovery order (changed appearances first, then their users, nearest
   first). In a cycle, an appearance's bounds can therefore be computed before the bounds of an
   overlay it shows.
5. Reinserts each node itself using BYOND's comparator and node layout so BYOND can continue to
   search and rebalance the tree. A node that compares equal to an existing node goes to the left.
   If a reinserted node equals a neighbor, it is reported as a duplicate and both IDs are kept.
6. Runs a full tree check again and emits a warning if the check fails.

The tree check expects the blank `0xFFFF` node: it accepts one node whose ID is `0xFFFF` and is not
present in the ID table, and rejects any other node whose ID does not lead back to it.

Depending on how far `repoint` progresses, the caller receives one of three outcomes:

- **An error (`Err`), with nothing changed.** `repoint` returns `Err` before any mutation when
  resource reload is unavailable on the current BYOND build, when the appearance manager is not set
  up yet, or when the pre-change tree check fails. The failed-check error states that the resource
  reload changed nothing. A caller can fix the cause and try again.
- **Success, with nothing changed.** Returned when no appearance uses a replaced resource.
- **Success, with warnings, after the change.** Once unlinking begins, `repoint` does not return an
  error. Duplicates are reported as warnings, and a failed post-change tree check produces a warning
  telling the user to restart the world because the tree has already been modified and retrying will
  not undo it.

## Caveats and build limits

- Code that walks the tree must expect the blank `0xFFFF` node and must not look it up by ID.
- Searches and the duplicate check rely on equal nodes always being placed on the same side (left).
- `repoint` records overlay and underlay users from live nodes before any appearance changes, and
  the code does not show which change would spoil a later read.
- `find` refuses resource reload on builds newer than `NEWEST_CHECKED` (`516.1689`). Two assumptions
  are only known to hold up to that build: the call positions of unlink and rebalance, and the
  constant node and table offsets listed above. The tree check validates the child, parent, and
  threaded-list links, the ID, the color, and the table's header, ID table, and capacity, but does
  not read the icon, overlay and underlay lists, or mouse pointers. Giving unlink and rebalance
  their own recipes is therefore not enough to lift the build check.
- Dependency order for bounds recomputation is guaranteed only when overlays and underlays do not
  form a cycle.

## Applies to

- `auxtools/src/appearance_tables.rs`: offset constants, `UNUSED_ID`, `find`, `Appearances::insert`,
  `Appearances::check`, `shown_in`, `bounds_order`, `equal_neighbour`, `repoint`
- `auxtools/src/registry/linux.rs`: `APPEARANCE_RELEASE`, `APPEARANCE_RELEASE_EARLY`,
  `appearance_tree_unlink`, `appearance_tree_rebalance` recipes
- `auxtools/src/registry/windows.rs`: `APPEARANCE_RELEASE`, `appearance_tree_unlink`,
  `appearance_tree_rebalance` recipes
- `debug_server/src/resource_reload.rs`: `apply` (passes warnings on)
