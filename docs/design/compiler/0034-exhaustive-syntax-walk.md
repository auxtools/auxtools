# Walking SpacemanDMM syntax trees exhaustively

Several parts of dmasm visit every node of a proc's syntax tree, including hot reload (to find what
a proc reads) and the compiler (to find names and static reads). Because SpacemanDMM, the parser,
gains node kinds and fields over time, a walk using wildcard matches would skip new syntax without
any sign. The syntax walk therefore names every variant and every field of SpacemanDMM's syntax tree
without wildcards. When a SpacemanDMM update adds a node or field, the build fails in `syntax.rs` by
design, and the fix is to handle the new node in the walk rather than add a wildcard.

`walk_proc` invokes its callback for each node in parent-before-child order, with one exception for
identifiers. Every identifier written in the code, whether declared or used, is reported as a `Name`
node, including each part of a type path. Unlike other nodes, each `Name` is reported after the node
that contains it rather than before, so callers that look for names must expect a `Name` after its
containing node. String and resource text is not reported as a name.

## Applies to

- `dmasm/compiler/src/syntax.rs`: `Node`, `walk_proc`, and the walk functions
