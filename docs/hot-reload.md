# Hot reload (experimental)

When you save a source file, the auxtools launcher compiles and applies supported changes directly
to a running BYOND world. For installation and launching a world, see [README.md](../README.md).

## Configuration

Hot reload is enabled by default for launched worlds. Pass `--no-hot-reload` on the CLI or set
`"hotReload": false` in VS Code to disable it. Give the launcher the same `-D` defines used to
compile the `.dmb` (and `--dme` if the project file is not beside the `.dmb`).

## Supported changes

Hot reload supports changes to:

- Proc bodies
- Added or removed procs and verbs
- New types and variables (including static locals)
- Variable types and defaults
- `static`, `const`, and `tmp` qualifiers
- Proc parameters and `set` statements
- `parent_type`
- Resource files (icons, sounds, and other assets used by the code, including files found through
  `FILE_DIR`)
- `.dmf` skins
- Inline `client/script` text

## Reboot, rebuild, and partial reloads

- **Reboot required:** Changing the initializer of an existing global or static keeps the running
  world's current value until a `world.Reboot()` (`auxtools reboot`). After a reboot, the launcher
  reapplies reloads before the world runs code, except reloads that introduce a resource missing
  from the build, which wait until `world/New`.
- **Rebuild required:** Changing a proc's declaration (such as turning a proc into a verb or adding
  `final`), reordering a proc's own definitions, modifying world or client settings stored in the
  `.dmb` (such as `client.control_freak`), or changing scripts in resource or included `.dms` files
  requires rebuilding the `.dmb` and restarting the world.
- **Partially applied reloads:** If earlier steps succeed and a later step fails, the world runs a
  mix of old and new code. The launcher lists each failed step and `auxtools wait reload` exits with
  code `1`. Rebuild the `.dmb` and restart the world to load all changes cleanly.

## Skins and client scripts

Updates to the active `.dmf` skin or `client/script` text take effect when clients log in. Run
`auxtools reconnect` to ask connected clients to reconnect (if DreamSeeker hosts the world, the
command reboots the world instead).

## Runtime behavior

In-flight proc calls finish on the old bytecode, and existing objects keep their verbs and built-in
defaults while new objects pick up updated computed defaults. Edits that touch only proc bodies
re-parse only the saved file, while structural edits re-parse the project.

On BYOND `515.1641–516.1689` (Windows) and `515.1644–516.1689` (Linux), `text2path()` finds newly
reloaded paths even after an earlier lookup returned `null`. On other builds, cached misses persist
until reboot.

See [Hot reload (experimental)](../editors/vscode/README.md#hot-reload-experimental) and
[Icons, sounds, and other files](../editors/vscode/README.md#icons-sounds-and-other-files) in the VS
Code extension README for full limitations (including generated assets and `FILE_DIR` path
differences), and [tests/hot_reload_test](../tests/hot_reload_test) for tested scenarios.
