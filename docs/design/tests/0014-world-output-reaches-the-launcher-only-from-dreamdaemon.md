# World output reaches the launcher only from `dreamdaemon`

The launcher receives a world's output only when hosting it with `dreamdaemon` (`dd.exe` on Windows
and `DreamDaemon` on Linux), so `launcher_test` asserts forwarded output only in that case.

Windowed BYOND host programs (`dreamseeker` and `dreamdaemon-gui`, which runs `DreamDaemon.exe`
with its window) display `world.log` inside their own windows rather than passing it to the
launcher. Linux has only DreamDaemon. Accordingly, `launcher_test` checks that world output was
passed on only when the server is `dreamdaemon` or `dd`, or on Unix. For any other server, it checks
only that the world exited.

## Applies to

- `tests/launcher_test/src/main.rs`: output check after `exited` in `launch`
- `auxtools-cli/src/byond.rs`: `Server::program`
