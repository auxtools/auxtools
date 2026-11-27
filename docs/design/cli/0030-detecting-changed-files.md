# Detecting changed source files by modification time

The source worker polls project files for changes using modification times, accounting for coarse
kernel timestamps, concurrent saves during parsing, and platform differences in how Windows reports
directory entries.

- **Lag margin and re-reads:** On Linux, file modification times come from a coarse kernel clock
  and can lag the wall clock by one timer tick, and a file can also be saved while a parse is
  reading it. `STAMP_LAG` is set to 50 ms as a safety margin rather than a measured tick length.
  Any modification time within `STAMP_LAG` of the start of a parse, or after it, is dropped so the
  next poll reads the file again in case it changed mid-read.
- **Directory listings on Windows:** On Windows, directory listings already include each file's
  modification time, so the watcher reads timestamps from directory listings instead of opening
  every file, matching file names case-insensitively.
- **Individual symlink lookups:** In a Windows directory listing, a symlink reports the link's own
  timestamp rather than its target's. Symlinks are therefore excluded from the listed times, and
  their targets' modification times are looked up one by one.

## Applies to

- `auxtools-cli/src/code.rs`: `STAMP_LAG`, `Watched`, `unless_saved_since`, `listed_times`,
  `listed`
