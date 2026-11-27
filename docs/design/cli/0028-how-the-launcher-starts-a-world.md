# How the launcher starts a world

Starting a BYOND world under auxtools requires ordering arguments as DreamDaemon and DreamSeeker
expect, preventing duplicate debug library loads, injecting the library before BYOND runs, and
tying the world process lifetime to the launcher.

## Arguments and environment

- **Port argument order:** DreamDaemon and DreamSeeker expect the port as the first argument after
  the `.dmb`, ahead of all other arguments. Whenever a port is supplied, the launcher places it
  first and appends the remaining arguments after it.
- **Clearing `AUXTOOLS_DEBUG_DLL`:** Some worlds load the debug library themselves using the path
  in `AUXTOOLS_DEBUG_DLL`. Because the launcher already loads the debug library, it removes
  `AUXTOOLS_DEBUG_DLL` from the world's environment so the world's own code cannot load it a second
  time.
- **Linux library loading:** On Linux, the launcher injects the debug library through `LD_PRELOAD`
  and prepends BYOND's directory to `LD_LIBRARY_PATH`.

## Windows entry-point injection and job ownership

Windows has no `LD_PRELOAD`, yet the debug library must still be loaded before BYOND's code begins
executing. Because both the launcher's injection code and the world process are 32-bit, `kernel32`
functions reside at the same addresses in both processes.

1. The launcher creates the world process in a suspended state.
2. It patches the process's entry point with a 6-byte sequence (`push stub; ret`) to jump to an
   injection stub it writes into the process's memory.
3. The stub saves registers, calls `LoadLibraryW` with the debug library path, looks up the
   `auxtools_debug_attach` export with `GetProcAddress`, and calls it. Both `LoadLibraryW` and
   `GetProcAddress` are called at the addresses read inside the launcher, which is valid because
   both processes are 32-bit.
4. The stub restores the original entry point bytes (which requires the entry point to be writable
   inside the process) and resumes execution at the entry point. If `LoadLibraryW` or
   `GetProcAddress` fails, the stub skips calling the export and only restores the entry point.

To tie the world's lifetime to the launcher on Windows, the launcher attaches the world to a job
object created with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Because the job handle is used only to
terminate and close the job, and Windows allows those operations from any thread, the
`windows::Job` wrapper is marked `Send`.

## Applies to

- `auxtools-cli/src/adapter/lifecycle.rs`: `Prepared::of`
- `auxtools-cli/src/game.rs`: `Game::start`, `windows::Job`
- `auxtools-cli/src/inject.rs`: `attach_on_entry`, `PATCH_LEN`, `ATTACH_EXPORT`
