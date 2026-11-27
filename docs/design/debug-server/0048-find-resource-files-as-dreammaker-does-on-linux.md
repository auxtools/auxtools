# Finding resource files as DreamMaker does on Linux

On Linux, DreamMaker resolves a file path one component at a time. An exact match of a component
wins, and otherwise DreamMaker picks the first directory entry that matches without regard to case,
in sorted order.

The debugger resolves resource paths the same way so a reload finds the same file that the build
did. It tries the path as given and under each resource directory, falling back to this
case-insensitive search when the exact path does not exist.

## Applies to

- `debug_server/src/assemble_env.rs`: `resource_file`, `resource_paths`, `case_insensitive_path`
