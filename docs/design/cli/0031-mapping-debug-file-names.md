# Mapping DreamMaker debug file names to local paths

DreamMaker records a file name for each proc in the `.dmb`'s debug information, and those names do
not always match paths relative to the project directory. A nested `.dme` can include `_std/tgs.dm`
yet record it as `tgs.dm`, a file included from outside the project directory is recorded under the
name the include chain gives it with `..` components (for example,
`#include "..\testing\testing.dm"` in a `.dme` under `game/` is recorded as
`../testing/testing.dm`), and the `.dme` itself is recorded under the absolute path it had at build
time, which may not exist on the current machine.

## How the adapter maps names and paths

- **Debug names to local paths (`WorldFiles::path`):** The debug adapter resolves debug names to
  local files through the `.dme`'s include list. Any name missing from the list is joined to the
  project directory. If that file does not exist and the cached include list is at least 5 seconds
  old, the adapter re-reads the include list. The resulting path is normalized lexically, so
  `../testing/testing.dm` under `/home/me/proj/game` gives `/home/me/proj/testing/testing.dm`
  rather than `/home/me/proj/game/../testing/testing.dm`. An editor then opens the same document a
  user opens by hand, and breakpoints set in it come back under the path the lookup below handles.
- **The `.dme`'s own name:** An absolute debug name is kept as is, unless that path does not exist
  locally and its file name matches the project's `.dme`, in which case the adapter substitutes the
  local `.dme`.
- **Local paths to debug names (`WorldFiles::name`):** A path inside the project directory is
  mapped back to its debug name using the include list, or converted to a relative path with forward
  slashes. For a file outside the project directory, an editor sends its real absolute path (for
  example `/home/me/proj/testing/testing.dm`), which the debug server's suffix match cannot connect
  to a debug name with `..` such as `../testing/testing.dm` (leaving breakpoints unverified with
  "no executable code at or after this line"). The adapter therefore computes the path relative to
  the project directory with leading `..` components (`relative_to`) and looks it up in the include
  list. For this lookup, the include list's path-to-name map holds every included file, including
  those whose debug name equals their path. If the include list names the file, its debug name is
  sent. If the include list does not name it, or if `relative_to` returns nothing (when either path
  is not absolute or the two share no root, such as two Windows drives), the path is sent unchanged
  and the debug server resolves it by suffix.

## How the CLI maps file locations

In `wait pause --break` and `trace`, `Place` parses any location ending in `:` followed by a line
number as a file and line, regardless of what it starts with. Otherwise, a location starting with
`/` or `(` is parsed as a proc path. A proc path cannot end in `:` and a line number because a
definition index is written `#N` and the only proc name with a colon is `operator:=`. Treating
every location starting with `/` as a proc path would break absolute file paths on Linux, where
`--break /home/me/proj/code/map.dm:12` would fail with "the world has no proc
/home/me/proj/code/map.dm:12".

The CLI has no include list. For an absolute file path, `World::name_in_world` asks the world for
its directory (`WorldInfo`, the directory of the `.dmb`) and sends the path relative to that
directory, with `..` components where needed, using the same `relative_to` helper. Relative paths
are sent as written, and an absolute path is sent unchanged if the world's directory is not known.

## How the debug server matches names

When the debug server looks up a file name, it first checks for an exact match against its recorded
debug names, ignoring case and slash direction. Only when no name matches exactly does it fall back
to matching by suffix at a path separator, because `map.dm` and `code/map.dm` can be distinct
files.

## Applies to

- `auxtools-cli/src/adapter/files.rs`: `WorldFiles`, `WorldFiles::path`, `WorldFiles::name`,
  `relative_to`, `STALE`
- `auxtools-cli/src/cli.rs`: `Place`, `World::name_in_world`
- `debug_server/src/server/code.rs`: `procs_in`
