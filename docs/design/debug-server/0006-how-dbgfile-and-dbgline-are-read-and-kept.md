# Reading and preserving `DbgFile` and `DbgLine` instructions

DreamMaker emits `DbgFile` and `DbgLine` instructions only for worlds built with `DEBUG`. Even in
`DEBUG` builds, three quirks in DreamMaker's output shape how the debug server resolves locations,
steps through code, and updates debug information on hot reload:

- At the start of the last proc of an included file, DreamMaker can insert the `.dme` file and line
  between two copies of the same location. More generally, it can emit a `.dme` file and line
  between the source locations of a proc from an included file.
- For nested includes, DreamMaker's debug file names can differ from paths relative to the project
  directory, and a new compile can produce the same file name with an added directory prefix.
- When execution is paused at an offset, a `DbgLine` or `DbgFile` at that offset has not executed
  yet.

In addition, a sleeping frame's offset points after the sleep instruction.

## Location lookup and stepping

Place lookup, stepping, and line remapping must all account for `.dme` positions appearing inside an
included proc:

- **Place lookup:** `Code::place` with `before` treats a `DbgLine` or `DbgFile` at the offset as not
  yet executed and returns the preceding location. It must not report the `.dme` position that
  DreamMaker inserts between two copies of a location. `place_of` passes `before` whenever a
  breakpoint sits at the offset.
- **Sleeping frames:** To report where a frame is sleeping, `place_running` looks one word back into
  the sleep instruction, so a `DbgLine` immediately following the sleep instruction is not counted
  and the line before it is reported.
- **Break On Next:** Break On Next never stops on a `DbgLine` or `DbgFile`. It never stops at the
  `.dme` position.

## Reloaded code and line remapping

During a hot reload, `adapt_debug_info` decides which debug instructions new bytecode retains:

- A new proc keeps its `DbgFile` and `DbgLine` instructions only if the world's own procs have them.
  Otherwise they are removed, so worlds built without `DEBUG` receive no debug lines in reloaded or
  new procs and cannot accept line breakpoints in them.
- Changed code keeps its debug instructions only if the proc's original bytecode starts with a
  `DbgFile`.
- The reload preserves DreamMaker's original file name whenever the new name only adds a directory
  prefix to it.

When a reload shifts line numbers without changing bytecode, `remap_lines` updates only the lines of
the proc's own file (the first file named by its bytecode). Lines belonging to any other file, such
as the `.dme`, stay unchanged.

## Applies to

- `debug_server/src/code_index.rs`: `Code::place`
- `debug_server/src/server/code.rs`: `Codes::place_running`, `place_of`, `frame_place`
- `debug_server/src/instruction_hooking.rs`: `BreakOnNext` arm of `Server::handle_instruction`
- `debug_server/src/hot_reload.rs`: `adapt_debug_info`, `remap_lines`
