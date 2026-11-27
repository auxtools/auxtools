# File naming in `DbgFile` instructions

`DbgFile` names the source file for the line records that follow. DreamMaker's file names are not
plain paths:

- The name is relative to the `.dme`'s directory and uses forward slashes on every platform.
- For a nested include, DreamMaker uses the string written in its `#include` directive rather than
  its path from the `.dme`.
- A file built directly, rather than through a `.dme`, is named by its full path.

The compiler takes the proc's file name from `ProcSource::file`, which the caller provides in
DreamMaker's form, and queries `ProcSource::file_names` for other files. A wrong file name sends
the debugger to the wrong file, and callers that compile code from text (such as debugger
expressions) have no real file and must supply a name themselves.

## Switching files on `/type::var` reads

A read of `/type::var` can switch files because its value is written where the variable's default
is defined, which may be in another file:

- A `/type::var` read records the line where the variable's value is written and emits `DbgFile`
  for that file. The next compound term records its own line again.
- A `/type::var` default from another file that folds away emits no `DbgFile`, yet the next line
  still names the proc's file again as if the read had switched files.

## Comparison tool conventions

The comparison tool builds each case through a `world.dme` that includes `case.dm`, matching how a
project is built so DreamMaker records the file name as an include (as real builds do) rather than
as a full path. When DreamMaker's `DbgFile` name is a suffix of the path the tool expects, the tool
takes DreamMaker's name.

## Applies to

- `dmasm/compiler/src/compiler/procs.rs`: `ProcSource::file`, `ProcSource::file_names`, and the
  `/type::var` read
- `dmasm/compiler/src/compiler.rs`: `other_file` check in `Compiler::emit_expr`
- `dmasm/compiler/examples/compare_dreammaker/main.rs`: `run_case`
- `dmasm/compiler/examples/compare_dreammaker/compare.rs`: `DbgFile` check in `compare`
