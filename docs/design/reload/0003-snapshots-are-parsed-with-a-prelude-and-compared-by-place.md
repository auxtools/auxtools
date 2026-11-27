# Parsing snapshots with a prelude and detecting moved code

Snapshots are parsed with SpacemanDMM, using a synthetic prelude file so conditional compilation
follows the target BYOND version rather than SpacemanDMM's defaults, and tracking moved code and
variable defaults by body comparison and source location rather than SpacemanDMM's file IDs.

## Prelude file

Codebases define linter-only items under `SPACEMAN_DMM` and branch on `DM_VERSION` and `DM_BUILD`.
Before parsing, a prelude file undefines `SPACEMAN_DMM`, sets `DM_VERSION` and `DM_BUILD` to the
target BYOND version, and adds the caller's defines. Pushing the prelude as its own file leaves the
`.dme` with the same line numbers it has on disk.

## Line moves and file numbering

The running world's bytecode records the source line of each statement, and a proc body's lines can
shift while its header stays in place (for example, when a line is added inside a comment or an
excluded `#if` branch). When files are reloaded, every unchanged body in those files is compiled
with the old code before the new definitions go in, and line moves are computed by comparing the
two. `diff` does the same for the files it is told changed. A comment edit that shifts lines
therefore updates the line numbers of procs below it even though their code is unchanged.

SpacemanDMM numbers the files it reads, so inserting an include renumbers every subsequent file
without moving any source inside those files. A variable default whose location changed is recorded
as moved, and locations are compared by file path, line, and column rather than by SpacemanDMM's
file ID. Inserting an include therefore does not make every default in later files appear moved.

## Applies to

- `dmasm/reload/src/reload.rs`: `Snapshot::parse` (the `PRELUDE` file), `Snapshot::reload_files`,
  `Snapshot::place`, default comparison in `compare_types`
