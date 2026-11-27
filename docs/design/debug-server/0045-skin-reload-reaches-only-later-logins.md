# A skin reload reaches only later logins

BYOND sends the skin's text to a client at login as server-to-client message `0xAF`. Resending that
message to an already connected client makes it rebuild its windows without the setup it received
at login. Instead, a skin reload installs the new skin only for future logins and lets the user
reconnect existing clients.

- A skin reload adds the skin file with `fcopy_rsc()` and writes the resulting resource ID directly
  into the world's skin field, so clients that log in afterward receive the new skin. Writing the
  ID directly is sound because a resource added with `fcopy_rsc()` remains in the world's cache for
  the rest of the session.
- The reload counts connected clients before writing the new skin ID. If counting fails, the reload
  fails before changing anything, avoiding a failure report after the write has already applied.
- When DreamSeeker hosts the world, reconnecting DreamSeeker's own client restarts the world
  outside the debugger. `reconnect` therefore reports `Hosted` and does nothing in that case.
  Otherwise, it sends `.reconnect` to each client that has a key (see
  [the idle-world waker](0020-the-idle-world-waker.md)).

## Applies to

- `debug_server/src/skin.rs`: `set`, `clients`, `reconnect`, `hosted`
