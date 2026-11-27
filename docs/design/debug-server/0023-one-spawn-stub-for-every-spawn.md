# One spawn stub runs every spawn

The launcher's `/proc/auxtools_spawn_stub` executes code that a client asks to spawn. Because calls
that are already running keep their own bytecode even while they sleep, the debug server installs
each spawn's code into this single spawn stub just before calling it, even while earlier spawns are
sleeping.

The stub is compiled with no locals. The locals table the stub has when first seen is the empty
table, and the server keeps it as the base for each spawn's own locals table.

## Applies to

- `debug_server/src/server/spawn.rs`: `next_spawn`, `set_stub_locals`
