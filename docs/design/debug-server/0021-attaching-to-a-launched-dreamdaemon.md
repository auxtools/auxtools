# Attaching to a launched DreamDaemon

The launcher starts DreamDaemon with `AUXTOOLS_DEBUG` set. On Linux it loads the debug library
through `LD_PRELOAD`, and on Windows the library attaches when its exported `auxtools_debug_attach`
function is called.

Three constraints govern what the debugger does during attachment:

- **Linux constructor and `LD_PRELOAD` cleanup:** On Linux, the debugger attaches from the library
  constructor, which runs when DreamDaemon loads the library. By then DreamDaemon's dependencies,
  including `libbyond.so`, are loaded, so the debugger can find BYOND's functions. Because programs
  the world starts with `shell()` inherit the environment and are not BYOND, the debugger removes
  `LD_PRELOAD` from the environment as soon as it loads so those child programs do not load the
  debug library as well.
- **Capturing startup arguments:** DreamDaemon modifies the `.dmb` path in its own argument memory
  after startup. The debugger captures the command line and working directory when it attaches,
  before DreamDaemon changes them, and reports the captured copy.
- **Recording the world load before listening:** After a reboot, clients reconnect and read the
  journal to learn what happened. The debugger records the world load in the journal before it
  starts listening, so a client reconnecting after a reboot sees the reboot immediately.

## Applies to

- `debug_server/src/launched.rs`: `on_load`, `auxtools_debug_attach`, `COMMAND_LINE`, `attach`,
  `start`
