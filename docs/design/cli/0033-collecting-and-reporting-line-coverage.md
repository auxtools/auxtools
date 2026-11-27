# Collecting and reporting line coverage

From the command line, line coverage is collected only for an entire world run through
`auxtools launch`, such as a unit-test world that shuts itself down, where the client cannot request
results after the world process exits. The debug server therefore pushes unsolicited
`Response::CoverageEnded(Coverage)` events when a world shuts down, reboots, or may be ending, and
the launcher writes the LCOV and Cobertura report files on the client side.

## Delivering coverage when a world ends

The debug server sends `Response::CoverageEnded(Coverage)` to the client that owns the active
recording (see [recording line coverage](../debug-server/0049-recording-line-coverage.md)) before
the world unloads or exits:

- **Clean shutdown:** `Server::world_shutting_down` calls `coverage_ended` before broadcasting
  `Response::ShuttingDown`, so the owner receives its coverage lines first, followed by
  `ShuttingDown` and `Disconnect`. The report is built while the world's procs and strings are still
  loaded.
- **Reboot:** `world_del` in `debug_server/src/launched.rs` calls `Server::world_may_end` in its
  rebooting branch (described below). No `ShuttingDown` follows (see
  [stopping and rebooting a world through DM evaluation](0010-ending-and-rebooting-a-world.md)).
- **Termination or crash:** A world that is terminated or crashes sends nothing, and its coverage is
  lost.

### Ends the debug server cannot be sure of

The launcher's helper override of `world/Del` (see
[the launcher's helper procs](../debug-server/0022-the-launchers-helper-procs.md)) reports how the
world's own `Del()` completed, and the debug server sorts the call into three cases: a reboot is
under way (`world/Reboot` was entered and has not returned), `Del()` returned normally (the world is
shutting down), or `Del()` raised a runtime error.

Two of those cases are ambiguous:

- **Runtime error in `Del()`:** When `world.Del()` is called as a proc (as the launcher's stop and
  `auxtools stop` evaluate it), the runtime error stops the call and the world continues running
  (see
  [stopping and rebooting a world through DM evaluation](0010-ending-and-rebooting-a-world.md)).
  When the world is deleted with `del(world)`, BYOND destroys it even though `Del()` raised a
  runtime error: the test world's process exits with code `0` after `del(world)` even when
  `world/Del` fails with a runtime error before calling `..()`. The debug server cannot tell these
  apart. This is not a corner case: tgstation's unit-test build ends its run with `del(world)` after
  `world/Reboot()` has returned, and in the test environment its `world/Del()` raised a runtime
  error at that point. Before this case was handled, a full unit-test run under
  `auxtools launch --lcov` wrote no report because the process exited without the server sending
  anything.
- **`del(world)` inside `world/Reboot()`:** A world can call `del(world)` from inside
  `world/Reboot()`, causing `world_del` to run in the rebooting branch even though the world is
  ending rather than rebooting.

In both ambiguous branches (rebooting and failed `Del()`), the server calls `Server::world_may_end`.
It sends `CoverageEnded` with everything recorded so far, waits until that frame has been written to
the owner's connection (`wait_until_sent`, up to `COVERAGE_PATIENCE`, 10 seconds), and then starts a
new recording for the same owner. If the world ends, the report was already delivered before the
process could exit. If the world continues running after a failed `world.Del()` call, recording
continues and the client sums the parts together. In an actual reboot, the new recording is
discarded along with the rest of the server's state when the world unloads.

Waiting for the write is what makes delivery reliable. Messages to a client are queued for its
writer thread (see
[debug server client connections and writer threads](../debug-server/0017-client-connections.md)),
and an exiting process does not wait for that thread. Each client counts the frames queued for it
(`Client::queued`) while its writer thread counts frames written and flushed (`Client::written`).
`wait_until_sent` waits until the two counts meet, the writer ends, or the deadline expires. On a
clean shutdown, `world_shutting_down` already waits up to 1 second for every writer to finish after
`ShuttingDown` (see
[stopping and rebooting a world through DM evaluation](0010-ending-and-rebooting-a-world.md)), and
waits up to `COVERAGE_PATIENCE` instead when it has just sent a coverage frame. The wait ends as
soon as the writers are done.

The launcher keeps collecting `CoverageEnded` events until the world process exits.

## Recording a whole run (`auxtools launch`)

`auxtools launch --lcov FILE` and `--cobertura FILE` record a world from its first instruction. The
launcher's debug adapter is the first client of a launched world, and when the launcher sets
`AUXTOOLS_DEBUG` to `wait` (see
[attaching to a launched DreamDaemon](../debug-server/0021-attaching-to-a-launched-dreamdaemon.md)),
`start` in `debug_server/src/launched.rs` calls `Server::process_until_configured`, handling
requests until `Request::Configured` arrives before any DM code runs. The launcher requests that
wait when debugging, when hot reload is enabled, and when coverage is requested, so
`--no-hot-reload --lcov FILE` still records from the first line.

On every connection to the world (the initial launch and every reconnect after a reboot), the
adapter sends `Request::Coverage` in `Adapter::connected` before sending `Request::Configured`.
Recording therefore starts before the world's first instruction, global variable initializers
included, on every world load. If the request fails (a world compiled without `DEBUG`), the launcher
prints `"Coverage disabled: REASON"` and lets the world run on.

`CoverageEnded` reaches the adapter's event listener (`ToAdapter::input` in
`auxtools-cli/src/debuggee.rs`), which forwards it to the launcher as `Note::Coverage`. The launcher
adds each event to a `Report` (`auxtools-cli/src/coverage.rs`), merging lines of the same file and
summing their hit counts so a run with reboots yields one report covering all world loads.

When the world process exits, the launcher writes the requested files and prints
`"Coverage: N of M lines covered (P%) in F of G files"`. If no `CoverageEnded` ever arrived, writing
fails with `"no coverage report written: world exited without reporting coverage"`. That error
determines the launcher's exit code (`1`) only if the world itself exited with code `0`. A world
that failed or was terminated keeps exit code `4`.

Several details govern launcher-managed coverage:

- **Detached runs:** With `--detach`, the coverage options are passed to the background launcher,
  which writes the report files when the world exits (for example, after `auxtools stop`).
- **Single-recording limit:** Because the launcher holds the world's single coverage recording, any
  other protocol client that sends `Request::Coverage` fails with
  `"another coverage recording is already running"`.
- **DAP sessions:** The `coverage` flag in the adapter's `LaunchArguments` carries `#[serde(skip)]`
  so it is never read from a DAP launch request, and only the CLI's `launch` sets it.
  `Output::coverage` does nothing when the adapter serves a DAP client.

## Client-side report formats

Report files are written by the launcher (`auxtools launch`) rather than the world, keeping
report-formatting code out of the debug library. File names in both formats are DreamMaker's debug
file names (relative to the `.dme`'s directory, see
[mapping DreamMaker debug file names to local paths](0031-mapping-debug-file-names.md)), with
backslashes replaced by forward slashes.

Both writers are written by hand with no added dependencies (the `auxcov` crate removed in auxtools
4 wrote Cobertura XML through `grcov`, which pulled in over a hundred packages):

- **LCOV tracefile (`Report::lcov`):** Emits `TN:` (empty test name), `SF:` followed by the file
  name, one `DA:line,hits` entry for each line with code, `LF:` (lines found), `LH:` (lines hit),
  and `end_of_record` for every file.
- **Cobertura XML (`Report::cobertura`, DTD `coverage-04.dtd`):** Emits one `<package>` per
  directory, one `<class>` per file with `name` (the file's base name) and `filename`, and one
  `<line number hits>` for each line with code. Branch rates and complexity are written as `0`,
  `<source>` is `.`, `version` is `auxtools VERSION`, `timestamp` is seconds since the Unix epoch,
  and rates have four decimal places.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Response::CoverageEnded`
- `debug_server/src/server/coverage.rs`: `Server::coverage_ended`, `Server::world_may_end`
- `debug_server/src/server/sessions.rs`: `world_shutting_down`, `wait_until_sent`, `hang_up`,
  `COVERAGE_PATIENCE`, `Client::queued`, `Client::written`, `Sessions::post`, `Sessions::writer`
- `debug_server/src/launched.rs`: `world_del`, `start`
- `auxtools-cli/src/coverage.rs`: `Report`, `Report::add`, `Report::lcov`, `Report::cobertura`
- `auxtools-cli/src/adapter/lifecycle.rs`: `Adapter::record_coverage`, `Adapter::connected`,
  `Adapter::start`
- `auxtools-cli/src/debuggee.rs`: `ToAdapter::input`
- `auxtools-cli/src/dap.rs`: `Note::Coverage`, `Output::coverage`
- `auxtools-cli/src/cli/launch.rs`: `report_coverage`, `run`, `detach`
