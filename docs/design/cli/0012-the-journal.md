# Keeping a numbered journal across world reboots

Runtime errors, reload results, world loads, output, and the results of spawned code happen whether
or not a client is connected or waiting. Because a CLI command that waits for one of these events
can start after the event occurs, or have its connection cut by a reboot while waiting, the world
maintains a numbered journal in the process that persists across world reboots.

## Protocol operations

- **Reading and following:** `Request::Journal { after, kinds, limit, follow }` returns entries
  numbered after `after` matching `kinds`, along with the first and last entry numbers. With
  `follow` enabled, subsequent entries of those kinds arrive as `Journaled` events until the client
  sends another `Journal` request. A later `Journal` request without `follow` stops those events.
- **Adding:** `Request::JournalAdd` receives no response (see
  [requests without a response](0006-requests-without-a-response.md)), and the added entry is not
  sent back to the client that added it.
- **Spawning:** `Request::Spawn` responds with `Response::Spawned`, carrying a spawn number, once
  the expression is compiled. The spawned code's result is not returned as a response, and instead
  goes into the journal as a `Spawned` event with that spawn number.

## Avoiding missed events in the CLI

The CLI tracks journal positions so that events are not missed across requests or reboots:

- **Spawn and wait:** Reads the journal's last entry number before sending `Spawn`, then follows
  from that number. Otherwise, a result journaled between the two requests would be missed.
- **Waiting without `--since`:** Starts after the journal's last entry. If the world reboots before
  answering, the wait reconnects and queries from the start of the journal for world entries.
  Because a rebooting world journals its load before accepting the reconnection, the wait starts
  just before the last `Loaded` entry and does not miss the load.
- **Registering as a watcher:** If a reboot cuts the connection while registering as a watcher, the
  wait reconnects and continues from the same entry number, because the new world keeps the journal.
- **Reconnecting while the outgoing world is shutting down:** The listener continues accepting
  connections while the ending world's debug server shuts its clients out, and that server tells any
  waiting connection to disconnect. A wait that reconnects that quickly registers as a watcher on a
  connection that is about to close. `reconnect` then waits `LISTEN_INTERVAL` and reconnects again
  until the next world answers, the process exits, or the wait's deadline expires.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::Journal`, `Request::JournalAdd`, `Request::Spawn`,
  `Response::Spawned`, `JournalEntry`, `JournalEvent`, `JournalKind`
- `debug_server/src/server/journaling.rs`: `handle_journal`, `journal`
- `debug_server/src/journal.rs`
- `auxtools-cli/src/cli.rs`: `spawn`
- `auxtools-cli/src/cli/log.rs`: `journal_end`, `watch_through_reboots`, `reconnect`
