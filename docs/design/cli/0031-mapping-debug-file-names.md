# Mapping DreamMaker debug file names to local paths

DreamMaker records a file name for each proc in the `.dmb`'s debug information, and those names do
not always match paths relative to the project directory. A nested `.dme` can include `_std/tgs.dm`
yet record it as `tgs.dm`, and the `.dme` itself is recorded under the absolute path it had at
build time, which may not exist on the current machine.

## How the adapter maps names and paths

- **Debug names to local paths:** The debug adapter resolves debug names to local files through the
  `.dme`'s include list. Any name missing from the list is joined to the project directory. If that
  file does not exist and the cached include list is at least 5 seconds old, the adapter re-reads
  the include list.
- **The `.dme`'s own name:** An absolute debug name is kept as is, unless that path does not exist
  locally and its file name matches the project's `.dme`, in which case the adapter substitutes the
  local `.dme`.
- **Local paths to debug names:** A path inside the project directory is mapped back to its debug
  name using the include list, or converted to a relative path with forward slashes. A path outside
  the project directory is sent to the debug server unchanged, because the server resolves it by
  suffix and the adapter must not rewrite it.

## How the debug server matches names

When the debug server looks up a file name, it first checks for an exact match against its recorded
debug names, ignoring case and slash direction. Only when no name matches exactly does it fall back
to matching by suffix at a path separator, because `map.dm` and `code/map.dm` can be distinct
files.

## Applies to

- `auxtools-cli/src/adapter/files.rs`: `WorldFiles`, `WorldFiles::path`, `WorldFiles::name`,
  `STALE`
- `debug_server/src/server/code.rs`: `procs_in`
