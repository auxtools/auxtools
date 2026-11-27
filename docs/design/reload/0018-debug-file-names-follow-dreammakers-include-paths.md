# Debug file names follow DreamMaker's include paths

DreamMaker's `DbgFile` instructions name each source file by its include path rather than its path
in the project directory, while `__FILE__` uses project-relative paths instead. On Windows and Linux
across BYOND builds 515.1637–516.1688 (found by experiment), DreamMaker applies three rules:

- An include in a `.dme` uses its literal include path with forward slashes. For example, when
  `_std/__std.dme` includes `tgs.dm`, the file `_std/tgs.dm` is named `tgs.dm`.
- Other includes are resolved relative to the directory of the including file's debug name and keep
  `..` segments, as in `deep/../b.dm`.
- The root `.dme` keeps its build-time path.

The reload snapshot computes each file's debug name using these rules and records it whenever it
differs from the project path. Reloaded code therefore names files the same way DreamMaker does, so
the debugger sees matching file names in original and reloaded code.

## Applies to

- `dmasm/reload/src/debug_files.rs`: `names`, `normalize`
- `dmasm/reload/src/reload.rs`: `Snapshot::debug_names`
