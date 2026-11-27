# Renaming the stand-in world process on Linux

On Linux, the stand-in world renames its process to `standin-world` by writing to `/proc/self/comm`
before connecting, preventing concurrent `auxtools` commands from discovering it as a DreamDaemon
process.

The stand-in world is the test binary itself, hard-linked or copied into a fake BYOND directory as
`bin/DreamDaemon` (`bin/dd.exe` on Windows) so the launcher starts it like a real world, and the
test answers the launcher over that process's socket. Because `auxtools list` and commands run
without a PID connect to every process named `DreamDaemon` on the machine, another test suite
running such a command would otherwise connect to the stand-in, which would then hand that
connection to its test as if it came from the launcher.

`world_process` performs this rename only on Linux. Other platforms leave the process name
unchanged: on Windows, the stand-in stays named `dd.exe`, and whether a command given no PID can
connect to it there has not been checked.

## Applies to

- `tests/support/src/standin.rs`: `world_process`, `launch_arguments`
