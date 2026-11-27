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

## Delivering macros with reloads

The source worker also reads the project's macros (`#define`s) from each full parse, and the debug
adapter gives them to the world with a `Defines` request (see
[one-way requests and in-order server responses](0006-requests-without-a-response.md)) so that
expressions typed by the user can use them.

Macros from a reload cannot be sent afterwards in a separate `Event::Macros` once the reload has
been applied, reported, and written to the journal. A client waiting for the reload (such as
`auxtools wait reload`) and then evaluating an expression that names a new or changed macro could
run before the world received the updated macros, seeing the old value or "undefined variable".
Instead, a reload carries its macros with it:

- `Event::Reload` and `Event::NothingChanged` carry `macros`, set when the parse the reload was
  made from has macros that differ from the ones last given (`Code::changed_macros`).
- The adapter sends `Defines` via `Adapter::take_macros` (called in `Adapter::reload` and for
  `Event::NothingChanged`) after the world has applied the reload and before writing the journal
  entry and printing the result. A reload the world does not apply does not change the macros.
- A save that changes only macros and no code is reported as a reload that changed nothing ("no
  runtime changes"), and its macros travel the same way.
- A reload of one to `MOST_ALONE` files parsed alone carries no macros, because such a parse does
  not change them (a changed `#define` takes a full parse).

`Event::Macros` remains for cases with no reload to carry them: the macros of the build when
watching starts, the macros of a rebuilt `.dmb` read again at a reboot, and a parse that stopped
early (which has no macro history, so the macros come from a separate preprocessor pass after the
reload).

## Returning freed memory on Linux

Parsing a large project consumes significant memory, and glibc can retain large freed heaps instead
of returning them to the OS. On Linux with glibc, the source worker calls `malloc_trim(0)` after a
parse so that memory is returned to the OS.

## Applies to

- `auxtools-cli/src/code.rs`: `Code::run`, `MOST_ALONE`, `give_back_memory`, `Code::changed_macros`,
  `Code::give_macros`
- `auxtools-cli/src/adapter/lifecycle.rs`: `same_code` check in `start`
- `auxtools-cli/src/adapter.rs`: `Adapter::take_macros`
