# Process and pipe behavior of world test stand-ins

The launcher's tests use stand-ins for worlds so they can run without BYOND. Where the launcher
depends on a world's behavior, those stand-ins must behave like a real world:

- **Integration test entry point:** The integration test stand-in launches the test binary again as
  a world process. The `world_process` function in `auxtools-cli/tests/standin.rs` is marked
  `#[test]` not as a real test, but as the entry point selected for that child process. Removing it
  as an apparently empty test breaks every test that starts a world process through the stand-in.
  (Renaming it to something like `world_process_entry` with an assertion message would make that
  role explicit in the code.)
- **Draining client requests on Windows:** On Windows, the client's end of a named pipe closes only
  after the server has read the client's last request. Because the debug library reads every
  request, the fake world servers in the unit tests of `local.rs` do the same. In particular,
  `chatty`, which continuously sends events, reads on a separate thread while writing so that a
  client's pipe closes under the same conditions as with a real world.

## Applies to

- `auxtools-cli/tests/standin.rs`: `world_process`
- `tests/support/src/standin.rs`: `world_process`
- `auxtools-cli/src/local.rs`: `chatty` and the other fake servers in the `tests` module
