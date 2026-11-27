# DreamSeeker clients on process exit and hosted-world reconnects

A DreamSeeker client joined to a world cannot reconnect to a new OS process, and a world hosted by
DreamSeeker itself (rather than DreamDaemon) cannot reconnect its host without restarting the world.

When a world process exits, the debug adapter drops the DreamSeeker clients it joined instead of
keeping them for a restart. Clients joined with `closeWithWorld` are terminated, while other clients
are left to close on their own. After a restart, the user must join again, whereas a reboot within
the same process keeps connected clients alive.

`Request::Reconnect` instructs each logged-in client to run `.reconnect` and responds with the
number of clients it asked. For a world hosted by DreamSeeker, it does nothing and responds with
`Reconnected::Hosted`. Skin and script reloads that require a reconnect therefore cannot refresh the
host of a hosted world.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::Reconnect`, `Reconnected`
- `debug_server/src/skin.rs`: `reconnect`
- `auxtools-cli/src/adapter/join.rs`: `Joined::world_ended`
- `auxtools-cli/src/adapter/lifecycle.rs`: `relaunched`, `ended` (callers of `Joined::world_ended`)
