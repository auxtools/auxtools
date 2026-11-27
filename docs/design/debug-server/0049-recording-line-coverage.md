# Recording line coverage

The debug server records line coverage by counting executions of DreamMaker's `DbgLine`
instructions, leaving every other opcode running at native BYOND speed through the opcode jump
table. Coverage measures only how many times execution reaches each source line that has code, with
no branch or proc coverage, and works only on worlds compiled with `DEBUG`.

## Counting `DbgLine` executions

DreamMaker emits a `DbgLine` instruction (whose operand is the line number) before the bytecode of
each line, and a `DbgFile` instruction (whose operand is the string ID of the file name) whenever
the source file changes, only in worlds built with `DEBUG` (see
[reading and preserving `DbgFile` and `DbgLine` instructions](0006-how-dbgfile-and-dbgline-are-read-and-kept.md)).
Because coverage relies on those instructions, a world built without `DEBUG` cannot be measured, and
`Request::Coverage` fails with `"the world has no line numbers because it was compiled without debug
information. Add #define DEBUG to the .dme and rebuild"`.

When the instruction hook is about to execute a `DbgLine`, `coverage::enter` reads the line number
from the instruction's operand (the word after the opcode) and the file from the `filename` field of
BYOND's `ExecutionContext`, which the preceding `DbgFile` has already set. The operand must be read
directly because the `DbgLine` at the current offset has not executed yet, so the frame's own
`line` field still holds the previous line.

Counting `DbgLine` executions rather than every instruction is what keeps coverage overhead low.
While a recording is active and nothing else needs the instruction hook, the debug server restricts
the hook with `auxtools::instruction_hook::hear_only` to two opcodes: `Throw` (`0x12D`, which it
always listens to, as described in
[data breakpoint lifetimes and detecting DM throws](0008-when-data-breakpoints-end.md)) and
`DbgLine`. Every other instruction runs at native BYOND speed through BYOND's jump table (see
[entering the instruction hook from BYOND's dispatcher](../auxtools/0008-how-the-instruction-hook-enters-from-byonds-dispatcher.md)
and
[synchronizing rewrites to the opcode jump table](../auxtools/0009-the-opcode-jump-table-is-rewritten-only-under-one-lock.md)).

Before line coverage existed, `rest()` in `debug_server/src/instruction_hooking.rs` was reached only
when every opcode was routed to the handler, because `Throw` (the only opcode heard while resting)
never passes the `passes` fast path. With `DbgLine` also heard while resting, the handler runs on
every line and reaches `rest()` on every call. Because `hear_only` acquires a lock and rewrites the
entire jump table, `rest()` now returns immediately unless `auxtools::instruction_hook::hears_all()`
reports that every opcode is routed to the handler. `hears_all` is a public function in the
`auxtools` crate that reads the same flag maintained by `hear_all` and `hear_only`.

Because of this early return, any code that changes which opcodes should be heard while resting must
call `hear_all()` first so the next `rest()` chooses the resting set again. `coverage::start`,
`coverage::stop`, and `coverage::discard` all do this, and incoming client requests already call
`hear_all()` in `Server::handle_request`.

Hit counts are stored per file in a map from the file name's string ID to a vector of `u32` counts
indexed by line number. The last file used is cached as a pointer so consecutive lines in the same
file skip the map lookup. The recorder copies the file name the first time its string ID is seen, so
it never reads a string that BYOND may have freed before the report is built. Counts saturate at
`u32::MAX`, and line numbers of 4,194,304 (`MOST_LINES`, 2^22) or greater are ignored.

## Which lines count

A line has code if at least one instruction other than `DbgLine` or `DbgFile` follows its `DbgLine`
before the next `DbgLine` or `DbgFile`. `Code::lines_with_code` in `debug_server/src/code_index.rs`
lists those lines and their files for a single proc, and `coverage::enter` applies the same rule
during recording by ignoring any `DbgLine` whose next instruction (the word two words after its
opcode) is another `DbgLine` or `DbgFile`.

This filter exists because of the DreamMaker quirk recorded in
[reading and preserving `DbgFile` and `DbgLine` instructions](0006-how-dbgfile-and-dbgline-are-read-and-kept.md):
at the start of the last proc in an included file, DreamMaker can insert the `.dme` file and line
between two copies of the same location (`DbgFile x.dm`, `DbgLine 10`, `DbgFile world.dme`,
`DbgLine 7`, `DbgFile x.dm`, `DbgLine 10`, then the code). Without the filter, the report would list
a `.dme` line that has no code and would count the proc's first line twice per call. With it, the
first copy and the `.dme` position are neither listed nor counted. The launcher test suite verifies
this behavior on `/proc/last_in_file`, the last proc of `attach_host.dm`, on Linux and under Wine:
it calls the proc once and checks that its line is counted once and that no file other than
`attach_host.dm` is listed.

Several other rules determine what appears in the final report:

- **Zero-hit lines:** The report lists every line with code in the world, not just executed lines,
  so unexecuted lines appear with zero hits. When recording stops, the debug server disassembles
  every proc in the world (`Code::read` on `original_code`, the proc's bytecode with breakpoints
  removed), collects `lines_with_code`, and adds the recorded hits to the lines found. Any hit on a
  line not found in that pass is dropped.
- **Launcher helper procs and evaluations:** Lines from the launcher's helper procs
  (`auxtools_launcher.dm`, `HELPER_FILE`) are omitted. Expressions evaluated by the debugger run in
  such a helper proc, so they are not counted either, though world procs called by an evaluation are
  counted.
- **Hot reloads:** After a hot reload, the report reflects the code present in the world when the
  recording stops. Hits recorded on a line before a reload are reported against whatever that file
  and line hold at the end.
- **Sleeping procs:** A line counts when execution reaches its `DbgLine`. Resuming a proc after a
  sleep does not count that line again.

## Protocol and recording ownership

A client starts a coverage recording by sending `Request::Coverage` (which carries no payload) and
receives `Response::Coverage(Result<(), String>)`: `Ok(())` if recording started, or an error
message explaining why it did not. There is no protocol request to stop a recording. Instead,
results are delivered in a `Coverage` payload holding `files`, a list of `CoveredFile` entries
sorted by file name, where each entry holds `file` (the debug file name) and `lines`, a list of
`CoveredLine { line, hits }` sorted by line number. Both `Request::Coverage` and
`Response::Coverage` were added at the end of `Request` and `Response`, preserving existing bincode
indices and keeping `PROTOCOL_VERSION` at 15 (see
[protocol versioning and wire compatibility](../cli/0005-protocol-compatibility.md)). The debug
server returns raw counts and does not format report files, leaving LCOV and Cobertura output to the
client (see
[collecting and reporting line coverage](../cli/0033-collecting-and-reporting-line-coverage.md)).

Only one coverage recording can run in a world at a time, owned by the client stored in
`Sessions::covering`. `Request::Coverage` fails in two cases: when a recording is already running
(with `"another coverage recording is already running"`), or when the world was built without
`DEBUG` (with the message noted above).

Because there is no stop request, an active recording ends in one of three ways:

- The world shuts down, reboots, or may be ending, and the debug server sends
  `Response::CoverageEnded(Coverage)` to the owning client.
- The owning client disconnects, and the recording is discarded.
- The debug server recovers from a panic, and the recording is discarded.

Coverage and the instruction profiler are independent and can run together, though every instruction
reaches the handler while the profiler is active.

## Applies to

- `debug_server/src/coverage.rs`: `Recording`, `enter`, `start`, `stop`, `discard`,
  `lines_with_code`, `MOST_LINES`
- `debug_server/src/code_index.rs`: `Code::lines_with_code`
- `debug_server/src/instruction_hooking.rs`: `rest`
- `debug_server/src/lib.rs`: `DebugServerInstructionHook::handle_instruction`
- `debug_server/src/server/coverage.rs`: `Server::coverage`
- `debug_server/src/server/sessions.rs`: `Sessions::covering`, `remove_client`
- `auxtools/src/instruction_hook.rs`: `hears_all`, `hear_only`, `hear_all`
- `auxtools-protocol/src/messages.rs`: `Request::Coverage`, `Response::Coverage`, `Coverage`,
  `CoveredFile`, `CoveredLine`
