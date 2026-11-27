# The idle-world waker's client has no key and receives no `winset()` calls

The idle-world waker connects to the world's port without logging in. BYOND creates a client for
that connection with no key, and calling `winset()` on that client raises a runtime error.

The debug server therefore counts and updates only clients that have a key, for example when a skin
is reloaded. `dap_test` stands in for the waker by opening a plain TCP connection to `world.port`
and waiting until the world creates a client for it. With that client present, a skin reload reports
0 clients, and `auxtoolsReconnect` reconnects 0.

## Applies to

- `debug_server/src/skin.rs`: client counting and `winset` code
- `debug_server/src/waker.rs`
- `tests/dap_test/src/groups/skins.rs`: waker connection
