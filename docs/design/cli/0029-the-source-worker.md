# Dedicated source worker thread for hot reload parsing

Hot reload compares project parses produced by SpacemanDMM. Because SpacemanDMM's parse contexts
are bound to the thread that creates them, all parsing runs on a single dedicated source worker
thread that other threads communicate with by passing messages.

## When and what the worker parses

- **Wait for the BYOND build:** Parsing depends on the BYOND build through `DM_VERSION` and
  conditional compilation, and the build is known only after the world connects and reports it. The
  source worker therefore waits for the world connection, reads the reported BYOND build, and only
  then parses.
- **Invalidation on configuration change:** A launch whose BYOND path differs from the previous one
  drops the parsed source and reads it again, just like a launch with a different `.dme`, `.dmb`,
  defines, build flags, or hot reload setting.
- **Rebuilds during a parse:** The worker reads the `.dmb`'s modification time before parsing
  rather than after, so a rebuild that happens while a parse is in progress is detected as a change
  on the next connection.
- **Incremental vs. full parses:** Up to 8 saved files (`MOST_ALONE`) are parsed one by one, while
  more than 8 trigger a full project parse. On tgstation, parsing a single file takes about 0.2 s
  and parsing the whole project takes about 10 s. The threshold is tuned by hand, and those timings
  justify it only loosely.

## Returning freed memory on Linux

Parsing a large project consumes significant memory, and glibc can retain large freed heaps instead
of returning them to the OS. On Linux with glibc, the source worker calls `malloc_trim(0)` after a
parse so that memory is returned to the OS.

## Applies to

- `auxtools-cli/src/code.rs`: `Code::run`, `MOST_ALONE`, `give_back_memory`
- `auxtools-cli/src/adapter/lifecycle.rs`: `same_code` check in `start`
