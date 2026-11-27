# auxtools Debugger

Debug, profile, and hot-reload BYOND worlds in VS Code with
[auxtools](https://github.com/auxtools/auxtools). The extension supports line, proc, instruction,
and data breakpoints, stepping by source line or bytecode instruction, sleeping-proc call stacks,
variable inspection and expression evaluation, live disassembly, instruction profiling, and
experimental hot reload, all without modifying your world's source.

The extension requires VS Code 1.80 or later on Windows or Linux and bundles both the `auxtools`
launcher and the debug library (`debug_server.dll` on Windows, `libdebug_server.so` on Linux). By
default it also adds the launcher directory to `PATH` in VS Code terminals so CLI commands work
there. It registers `.dm` and `.dme` files for breakpoints and hover evaluation. Install a DM
language extension such as the
[DM language client](https://github.com/SpaceManiac/vscode-dm-langclient) for syntax highlighting
and editing features.

Supported BYOND builds are `515.1637–515.1647` and `516.1648–516.1689` (newer builds are attempted
with resource reload disabled). See the [auxtools repository](https://github.com/auxtools/auxtools)
for supported BYOND builds, the command-line workflow, static bug checking, and the `auxtools` Rust
library.

## Getting started

Compile your world with DreamMaker first, because the launcher does not build `.dmb` files. To use
line breakpoints, make sure `DEBUG` is defined in your `.dme` (`#define DEBUG`) before compiling so
DreamMaker writes line numbers into the `.dmb`. Many codebases define it already. Without it, line
breakpoints stay unverified, though function (proc) breakpoints still work.

Press `F5` or run **auxtools: Debug in DreamDaemon** to launch your compiled `.dmb` in console
DreamDaemon (`dd.exe` on Windows, `DreamDaemon` on Linux):

- If the [DM language client](https://github.com/SpaceManiac/vscode-dm-langclient) is installed and
  its active environment has a compiled `.dmb`, `F5` and **auxtools: Debug in DreamDaemon** launch
  that `.dmb` directly. A launch configuration that omits both `dmb` and `dme` also resolves them
  from the active environment.
- Otherwise, the extension searches the workspace for `.dmb` files, launches the sole match
  directly, or opens a picker when more than one exists.

On Windows, the extension opens DreamSeeker automatically when it launches a world in DreamDaemon.
Set `auxtools.autoJoin` to `false` to disable auto-joining, or run **auxtools: Join with
DreamSeeker** at any time to open another client. Clients opened by the extension stay open and
reconnect across a reboot, and close when the world process exits unless `auxtools.closeDreamSeeker`
is `false`. On Linux (where DreamSeeker is unavailable), launch notifications and **auxtools: Join
with DreamSeeker** offer to copy the world's `byond://` address instead.

Use **Run Without Debugging** (`Ctrl+F5`) or set `"noDebug": true` to disable breakpoints, pausing,
and stepping while keeping hot reload active.

If the debug library fails to initialize (for example, on an unsupported BYOND build), a launched
world keeps running without debugging or hot reload. Stop the session to terminate it.

### Connecting multiple clients

BYOND allows only one client per key. When a second client logs in with a key that is already
connected, the server closes the first connection and the second client takes over its mob. Because
a client not signed in to BYOND joins as a guest named `Guest-NUMBER` (derived from its
`computer_id`), multiple guest clients on the same computer receive the same key and disconnect each
other.

To test a local world with several clients, join with DreamSeeker as usual (or let auto-join open
the first client), then run **auxtools: Join with DreamSeeker as a New Guest** (also available in
the status bar World Actions menu) once for each additional client. Without prompting for a key or
displaying one in VS Code, the command generates a key in BYOND's guest form (`Guest-` followed by a
random number, such as `Guest-2328261779`), queues it for the next guest login, and opens
DreamSeeker on the world (Windows) or offers its `byond://` address to copy (Linux). Because the key
keeps BYOND's guest form, the world treats the client like any other guest, and `client.key`,
`client.ckey`, and `mob.key` show the assigned key.

- **Guest logins only:** Each queued key goes to one guest login. When several keys are waiting, the
  most recently requested one is given first, and a guest who logs in when none are waiting gets
  BYOND's `Guest-NUMBER`. A DreamSeeker client signed in to BYOND joins with its own key and leaves
  any waiting key for the next guest.
- **Reboots and reconnects:** When the world reboots or clients reconnect (**auxtools: Reconnect
  Clients** or `auxtools reconnect`), the world reissues every key it has given since the process
  started, in the order they were first requested, so returning guests stay connected with distinct
  keys. Which client receives which key depends on login order, and keys whose original clients have
  since closed are still reissued to the next guests who connect.

This replaces auxtools 3's Windows-only `#guest_override` Debug Console command and works on both
Windows and Linux. It is available only through the debug adapter, with no `auxtools` CLI command.
Other DAP clients can pass an optional `ckey` argument (up to 30 printable ASCII characters with at
least one letter or digit, excluding `<`, `>`, `"`, `\`, and `|`) to the custom `auxtoolsJoin`
request to assign a specific key, from which BYOND derives `client.ckey` in its usual way.

## Launch configurations

Add a configuration to `.vscode/launch.json` to customize how a world starts. **Add
Configuration...** provides starter templates:

```json
{
    "type": "auxtools",
    "request": "launch",
    "name": "Debug in DreamDaemon",
    "dmb": "${workspaceFolder}/world.dmb",
    "server": "dreamdaemon"
}
```

When the DM language client is installed, dynamic configuration fills in
`${workspaceFolder}/${command:dreammaker.getFilenameDmb}` for `dmb`,
`${workspaceFolder}/${command:dreammaker.getFilenameDme}` for `dme`, and
`dm: build - ${command:dreammaker.getFilenameDme}` as `preLaunchTask`.

| Option | Description |
| --- | --- |
| `dmb` | Path to the compiled `.dmb` to run. If omitted, derived from `dme` (or from the DM language client's active environment when both `dmb` and `dme` are omitted). |
| `dme` | Path to the project `.dme` file. Defaults to the `.dme` beside the `.dmb`, and its directory is the project directory used for source lookup and hot reload. |
| `server` | Program that hosts the world. Default: `dreamdaemon` (`dd.exe` on Windows or `DreamDaemon` on Linux, also aliased as `dd`). On Windows, `dreamdaemon-gui` runs windowed `dreamdaemon.exe` and `dreamseeker` hosts and joins in `dreamseeker.exe`. |
| `defines` | Array of preprocessor defines used to build the `.dmb` (`NAME`, `NAME=VALUE`, or DreamMaker flags such as `-DA -DB=2`). Used by hot reload and expression evaluation. |
| `hotReload` | Reload supported source and resource changes on save (default: `true`). Stays active when `noDebug` is `true`. |
| `noDebug` | Disable breakpoints, pausing, and stepping while keeping output, lifecycle controls, and hot reload (default: `false`). |
| `sleepingThreads` | Maximum number of sleeping procs shown in the Call Stack view while paused (default: `100`, or `0` to hide sleeping procs). |
| `byondPath` | Path to a BYOND installation directory (containing `bin`, or `bin` itself), overriding `auxtools.byondPath` and automatic discovery. |
| `port` | TCP port the world listens on (`1–65535`). Omit to let DreamDaemon choose a free port. Cannot be used with `"server": "dreamseeker"` on Windows. |
| `args` | Array of arguments passed after the `.dmb`, replacing the defaults (`["-trusted"]`, plus `"-close"` for `dreamdaemon-gui` on Windows). |
| `env` | Object of environment variables (`{ "NAME": "VALUE" }`) passed to the world process. |
| `debugLibrary` | Path to a custom debug library (`debug_server.dll` on Windows, `libdebug_server.so` on Linux), overriding `auxtools.debugLibrary` and the bundled library. |

Windowed DreamDaemon (`dreamdaemon-gui`) may prompt before running a world in trusted mode. Select
"Don't ask me this again" to suppress the prompt.

### SS13 Build Flags

With versions of the [SS13 Build Flags extension](https://github.com/spacestation13/SS13-BuildFlags)
that expose an API, auxtools automatically appends the selected flags on launch and restart whenever
`preLaunchTask` matches the task SS13 Build Flags builds with. When you change flags during a
session:

- If the new flags do not change compiled code, they take effect immediately.
- If they change code, hot reload holds that change and later code edits until a reboot.
- If they change maps or world settings, rebuild and restart the world.

Switching back to the previous flags unblocks held edits.

For attaches, and for launches with older versions of SS13 Build Flags that have no API, any
configuration that omits `defines` picks up the selected flags automatically when
`ss13BuildFlags.injectionMode` is set to `cli-args`. Set `"defines": []` to opt out, or combine them
with custom defines using `"defines": ["${command:ss13BuildFlags.current}", "EXTRA"]`.

### Output, stop, reboot, and restart

Console DreamDaemon writes world output to the Debug Console, with clickable source links on runtime
error locations. `dreamdaemon-gui` and `dreamseeker` display output in their own windows. Across a
`world.Reboot()`, the debug adapter reconnects automatically and restores breakpoints and hot
reloads.

- **Stop** requests a graceful shutdown through `world/Del()` so the world can clean up (for
  example, saving state). If the process has not exited after 5 seconds, the launcher terminates it.
  Press **Stop** a second time or choose **Disconnect** to terminate a launched world immediately. A
  launched world that shuts down on its own also gets a 5-second grace period before the launcher
  terminates any lingering process. If debugger initialization failed, **Stop** terminates the
  process immediately.
- **Restart** runs the configuration's `preLaunchTask` (if configured) and then reboots the world in
  place with `world.Reboot()`, keeping the same process and port so connected clients reconnect. The
  world loads any rebuilt `.dmb` (resetting the hot reload baseline if the `.dmb` changed) or
  reapplies active reloads. If the launch options changed or the debug server is disconnected,
  **Restart** stops the running process and launches a new one.
- **auxtools: Reboot World** (also available in the status bar world actions menu and hot reload
  notifications) calls `world.Reboot()` in place without running `preLaunchTask`. It works in both
  launched and attached sessions. If the world has not rebooted after 60 seconds, the Debug Console
  reports that the reboot timed out.

## Attaching to a running world

Run **auxtools: Attach to Running World** (or use an `attach` launch configuration) to connect to
any local world started with the auxtools launcher, whether from another VS Code window or the
`auxtools` CLI. Multiple VS Code windows and CLI sessions can attach to the same world at once, each
with its own breakpoints, data breakpoints, and runtime error filters. Pausing pauses the world for
every session, and any session can resume it.

Disconnecting from an attached session leaves the world running (and leaves it paused if other
debugging sessions remain connected). The Debug Console also reports hot reloads applied by another
launcher.

```json
{
    "type": "auxtools",
    "request": "attach",
    "name": "Attach to Running World",
    "processId": "${command:auxtools.pickWorld}"
}
```

| Option | Description |
| --- | --- |
| `processId` | Process ID of the running world (number or string), or `"${command:auxtools.pickWorld}"`. If omitted, attaches automatically when one world is running or opens a picker when several are running. |
| `dme` | Path to the project `.dme` file, if it is not beside the running `.dmb`. |
| `defines` | Preprocessor defines used to build the `.dmb`. Falls back to SS13 Build Flags when omitted. Set `[]` to disable. |
| `hotReload` | Enable hot reload from this session (default: `false`, so it does not compete with the launching session). Only one session can own hot reload for a world at a time. |
| `sleepingThreads` | Maximum number of sleeping procs shown in the Call Stack view while paused (default: `100`). |

## Debugging features

### Breakpoints

Click the editor gutter to set a line breakpoint, or use **Add Function Breakpoint** with a proc
path such as `/mob/proc/Login` (append `#1` for the proc's second definition, `#2` for the third,
and so on). Breakpoints set on global or static variable initializers are verified with a note that
those lines run only at world startup.

- **Conditions** are DM expressions evaluated in the paused frame. Falsy results skip the hit. A
  runtime error in a condition also skips the hit and is reported once in the Debug Console. If a
  condition does not compile, the breakpoint remains unverified. Hover over it to see why.
- **Hit counts** control which hits pause execution: `5` or `>= 5` pauses on the fifth hit and every
  hit after, `== 5` (or `= 5`) pauses on the fifth hit only, `% 5` pauses on every fifth hit, and
  `>`, `<`, and `<=` compare against the count. Only hits that satisfy the condition increment the
  counter. The count resets when you edit the condition or hit count or when the world reboots or
  restarts, and persists when you add or remove other breakpoints or apply a hot reload that keeps
  the breakpoint on its statement.
- **Logpoints** print a message to the Debug Console without pausing. Interpolate DM expressions in
  braces (for example, `health is {health} for {src}`), and double braces (`{{` and `}}`) for
  literal braces. Expression errors are printed inline in the message. Logpoints also support
  conditions and hit counts.

With hot reload enabled, breakpoints placed in a file with unsaved edits remain unverified until you
save and the reload completes, while existing breakpoints stay on the running code and move with
their statements after a reload. Breakpoints track line edits whether you save from VS Code, another
editor, or a branch checkout (VS Code notifies the debug adapter of unsaved buffers, while other DAP
clients place new breakpoints using the file on disk). With hot reload disabled, breakpoints refer
to the source the `.dmb` was compiled from, and the Debug Console warns if a file has changed since
the build.

Open **Loaded Scripts** from the **...** menu of the Run and Debug view to browse files listed in
the world's debug information along with BYOND's `stddef.dm`. Files that declare only types,
variables, or macros contain no procs and are not listed.

### Sleeping procs

While the world is paused, the Call Stack view shows the active proc first, followed by sleeping
procs (up to `sleepingThreads`, default `100`). You can inspect stack frames, view or edit
variables, and evaluate expressions in sleeping procs just like active ones. Stepping a sleeping
proc finishes once that proc wakes up.

### Data breakpoints

While paused, right-click a variable in the Variables view and choose **Break on Value Change**. The
world pauses immediately after the variable's value changes, usually on the next line, and logs the
old and new values in the Debug Console.

- **Supported targets:** You can set data breakpoints on object variables, list entries (and
  associative list values), globals, locals, arguments, `.` (dot), and pointer targets. A data
  breakpoint on a variable that holds a list triggers only when the variable is assigned a different
  list. Set one on a specific list entry to detect changes inside the list. Reassigning the same
  value does not trigger.
- **Unsupported targets:** You cannot set data breakpoints on `src`, `usr`, built-in constants, list
  keys, or computed expressions such as `a + b`.
- **Conditions and hit counts:** Conditions may reference `value` (the new value) and `old` (the
  previous value), such as `value < old`. Hit counts and condition error reporting work as for other
  breakpoints.
- **Lifetime:** Data breakpoints on locals or arguments last only for that call and end when the
  proc returns or unwinds. Data breakpoints on object variables end when the object is deleted.
  Across a `world.Reboot()`, data breakpoints set from the Variables view (including on globals) end
  and must be set again. Only a data breakpoint requested by global name alone, without a stack
  frame or Variables scope, is restored. Ended data breakpoints are marked in the Breakpoints view.

Data breakpoints are checked between bytecode instructions, so they also catch engine-driven changes
such as atom movement. Because checking slows execution while active, remove data breakpoints when
you no longer need them.

### Runtime errors

In the Breakpoints view, **Runtime errors** pauses on uncaught runtime errors and thrown values (on
by default), and **Caught runtime errors** also pauses on errors inside `try` blocks.

Click a filter's pencil icon to match messages case-insensitively with comma-separated substrings.
Prefix a term with `!` to exclude matching messages. For example, `bounds, null` pauses on messages
containing either word, `!libdreamluau, !rust_g` excludes messages containing either name, and
`list, !bounds` matches messages containing `list` unless they also contain `bounds`.

Runtime errors that do not trigger a pause are still printed to the Debug Console with clickable
links to the source file and line.

### Variables, hover, and evaluation

The Variables view displays **Locals** (including `.`), **Arguments** (including `src` and `usr`),
and **Globals** (loaded when expanded, with BYOND's built-in constants grouped under `(BYOND built-ins)`).
Objects expand to their variables (including built-in fields on `/vector`, `/pixloc`, `/callee`, and
`/savefile`), pointers expand to their target (`*`), and lists and alists expand to their entries on
demand. Hover over a variable to see its DM type.

- **Set Value** evaluates a DM expression in the selected frame and assigns the result (not
  supported on list keys, built-in constants, or pointer targets).
- **Copy as Expression** and **Add to Watch** generate an evaluatable path such as
  `mob.contents[3].name`.
- **Format > Hex** displays integer values in hexadecimal.

The Debug Console, Watch expressions, and breakpoint conditions accept DM expressions such as
`health / maxHealth` or `get_dist(src, usr) < 3`. When paused, name lookup checks locals and
arguments, then variables of `src`, then globals. A bare call `f()` calls `src.f()` if `src` has
that proc, or a global proc otherwise. While the world is running, only globals are in scope.

`/obj/item::force` reads the initial value of a type's variable. In launched worlds,
`auxtools_kept()` returns a persistent list for keeping values across evaluations until the world
reboots (for example, `auxtools_kept()["m"] = m` followed later by `auxtools_kept()["m"]`).

Expressions can mutate variables and call procs. Because watch expressions re-run at every pause,
side effects in a watch expression change world state.

The Debug Console also accepts multi-statement proc bodies with local declarations, loops, and
`return`:

```dm
var/n = 0
for (var/mob/M in world)
    n++
n
```

A body returns its `return` value, its trailing expression, or `.` when it ends with a statement
such as an assignment or increment, and can modify variables in the selected frame. Compiler
warnings and runtime errors in called procs appear alongside the result. Code that sleeps, or that
BYOND moves to the background via `world.loop_checks` or `set background = 1`, aborts synchronous
evaluation (use `auxtools eval --spawn --wait` in the terminal for code that sleeps).

Your project's `#define` macros become available in evaluations and logpoints once hot reload has
parsed the project, and update automatically on reload. Without a parsed project (including sessions
with `"hotReload": false`), source `#define`s are unavailable.

Evaluation folds constants using the same compiler as hot reload, with a few differences from
DreamMaker:

- `var/const` variables are read from the running world at runtime, so expressions like `~-CONST` or
  `-(0 * CONST)` can differ from DreamMaker's compile-time folded value.
- `x::name` resolves against the runtime type of `x` rather than its declared type, returning `null`
  when `x` is `null`.
- `..()`, `.()`, `__FILE__`, `__LINE__`, `__PROC__`, `__TYPE__`, and prefabs (`/type{var = value}`)
  are not supported.

Hovering over source code while paused reads a single expression in the selected frame without
executing code: identifiers, field accesses, and list indexing (`src.items[i]`) are resolved safely,
while proc calls, operator overloads, and custom `operator[]` are skipped. The Debug Console
provides completions (triggered by `.` and `:`) for visible locals, arguments, `src` members,
globals, global procs, object members (reading the receiver safely without executing code), and `#`
console commands.

Enter `#dis` (or `#disassemble`) in the Debug Console to print the bytecode of the selected frame's
proc, `#dis /proc/name [override_id]` for another proc, or `#help` for console command help.

### Disassembly

While paused, right-click a stack frame and choose **Open Disassembly View** to inspect dmasm
instructions, hexadecimal bytecode words, source lines, and the current instruction pointer. Frames
still executing pre-reload bytecode are labeled `(old code)` and omit source lines.

Click beside an instruction address to set an instruction breakpoint. The address must be the start
of an instruction in the proc's current bytecode (pre-reload bytecode cannot take new breakpoints).
Instruction breakpoints support conditions and hit counts and persist across world reboots.

Addresses have the form `0xPPPPPPPPCCCCOOOO`: a 32-bit proc ID, a 16-bit code ID (`0` for the
proc's current bytecode, or nonzero for pre-reload bytecode until reboot), and a 16-bit word
offset. For example, `0x0000002A00000013` is offset `0x13` in proc `0x2A`'s current bytecode.

When the Disassembly view has focus, **Step Over** and **Step Into** step by single bytecode
instruction instead of source line, with **Step Over** running called procs to completion. **Step
Out** always runs until the frame returns to its caller.

## Instruction profiling

Run **auxtools: Profile Instructions** (or choose **Profile Instructions** from the status bar world
actions menu) to start profiling. Click **Profiling** in the status bar (or run the command again)
to stop profiling and open the **Instruction timing** report beside your editor. The world keeps
running while profiling, at reduced speed.

The left pane lists up to 300 procs ranked by self time (with a name filter), and the top 40 procs
include full instruction-by-source-line listings. Use the toolbar to switch **Time** between `total`
and `per hit`, toggle **Include calls** (adds callee time bars and a call-time column), and toggle
**Show unexecuted instructions**. Click a `file:line` header to jump to that source line in VS Code,
or click a `LAB_...` jump target in operands to jump to that offset (`Alt+Left` returns).

The profiler excludes sleeping and engine work between DM runs, and includes calls from the Debug
Console. See [Instruction profiling](https://github.com/auxtools/auxtools/blob/master/docs/profiling.md#instruction-profiling)
for how timings and hook overhead are calculated and for CLI profiling (`auxtools profile`).

## Hot reload (experimental)

When you save a source file, the launcher compiles and applies supported changes directly to the
running world, including under **Run Without Debugging** (`Ctrl+F5`). The status bar shows hot
reload progress and state (`Reading source`, `Hot reload`, `Reloading`, `Reboot needed`, `Rebuild
needed`, `Hot reload failed`, or `Hot reload off`). Click it to open the world actions menu. Each
reload is also logged in the Debug Console.

At startup, the launcher warns if any source or resource files are newer than the `.dmb` (and offers
**Restart**), since the build may be missing those edits.

When an edit cannot be applied live, your file is still saved, but hot reload holds that edit and
any later code edits until the conflict is resolved. The Problems panel marks the offending lines,
and a notification offers the next action:

- **Reboot required:** You changed the initializer of an existing global or static variable (or
  switched SS13 Build Flags in a way that changes compiled code). Choose **Reboot** to call
  `world.Reboot()` without running `preLaunchTask`.
- **Rebuild required:** The edit is unsupported, fails to compile, or changes maps or world/client
  settings stored in the `.dmb`. Choose **Restart** to run `preLaunchTask` and restart
  (configure a build task or rebuild manually, because the launcher does not compile `.dmb` files).

Fixing or reverting the refused edit allows pending edits to reload.

If earlier steps of a reload succeed before a later step fails, the reload is **partially applied**,
leaving the world running a mix of old and new code. The status bar displays **Hot reload failed**
and the Debug Console lists every failed step. Rebuild the `.dmb` and restart the world to load all
changes cleanly.

After a `world.Reboot()`, the launcher restores active reloads before world code runs, except
reloads that reference a resource missing from the build, which wait until `world/New` (until then,
map-created atoms run code from the build).

Hot reload supports:

- Proc bodies
- Added or removed procs, verbs, world procs, and global procs (including inserting or removing
  definitions between existing overrides)
- New types, including mobs
- New variables and static locals
- Variable types, default values, and `static`, `const`, and `tmp` qualifiers
- Proc parameters, including `as` input types and `in` lists
- Prefabs
- `set` statements
- Parent types (`parent_type`)

These changes require a rebuild:

- Changing a proc's declaration (such as turning a proc into a verb or adding `final`) or reordering
  a proc's definitions
- Changing built-in defaults that hot reload does not support (such as `see_infrared`)
- Removing an override of a built-in variable on a built-in type itself (such as `/mob/layer`),
  because the running world no longer has BYOND's original default
- Changing maps, world settings, or client settings stored in the `.dmb` (such as
  `client.control_freak`)

Runtime behavior and caveats:

- **In-flight calls and existing instances:** Calls already running when you save finish on the old
  bytecode. Existing objects keep their verbs and built-in defaults while new objects receive
  updated computed defaults. A newly added computed variable is `null` on existing objects and in
  `initial()` until the world restarts.
- **Field caching:** Reloaded and evaluated code caches fields the same way DreamMaker does: after
  `a.b.f()`, a later `a.b.x` in the same flow can still read the old object even if `f()` replaced
  `a.b`. The compiler does not warn for this pattern, so read the field into a local variable if a
  call can replace it. Other compiler differences and copied DreamMaker quirks are reported as
  warnings.

### Icons, sounds, and other files

When you save an icon, sound, or other resource referenced by the code, the launcher reloads it. It
searches the project directory first and then `FILE_DIR` directories in order, matching DreamMaker.
Existing appearances, overlays, images, and mouse pointers redraw with the updated file, and
recompiled code and defaults use the new resource.

Resource reload has several limits:

- **Stored values keep the old resource:** Variables, lists, `/icon` and `/sound` datums, mutable
  appearances, and objects built later from those values continue referencing the old resource.
- **Byte-identical files share a resource ID:** If two files had the same bytes in the build, the
  launcher cannot replace one without affecting the other and warns you to rebuild.
- **Post-reboot deferral:** After a reboot, resource edits (and code that needs them) wait until
  `world/New`, so map-created atoms use resources from the build until then. Resources modified
  before launch are reported as outdated and reload only when saved again.
- **`FILE_DIR` paths:** A reloaded resource loaded from `FILE_DIR` has a path relative to the world
  directory (for example, `"[icon]"` and `browse_rsc()` produce `icons/lamp.dmi` rather than
  `lamp.dmi` as written in source).
- **BYOND build limit:** Because resource reload updates BYOND's appearance tree directly, auxtools
  disables it on builds newer than `516.1689` while the rest of the debugger still attempts to
  start.
- **Generated assets and caches:** Icons created at runtime with `icon()` keep their own copy of the
  data. Caches built during global initialization run before resource edits are restored on reboot
  and keep the old resources. Files sent with `browse_rsc()` or an asset cache reach clients only
  when resent. Regenerate those assets or rebuild the `.dmb`.

The launcher also watches and reloads referenced resource files that the world writes while running
(such as generated map preview icons).

### Skins

Saving the last `.dmf` file included by the `.dme` updates the skin sent to clients on login, and
the update persists across reboots. Connected clients must reconnect to load the new interface using
the reload notification or **auxtools: Reconnect Clients**. If DreamSeeker hosts the world, the
world reboots instead because a hosted client cannot reconnect independently. Skins that reference
new images missing from the build require a rebuild.

### Client scripts

Edits to inline `client/script` text apply on login and persist across reboots. Connected clients
must reconnect via the reload notification or **auxtools: Reconnect Clients** (or reboot when hosted
in DreamSeeker). Scripts stored in resource files or included `.dms` files require a rebuild.

## Commands and settings

### Commands

| Command | Availability | Description |
| --- | --- | --- |
| **auxtools: Debug in DreamDaemon** | Always | Launch a compiled `.dmb` in console DreamDaemon. |
| **auxtools: Attach to Running World** | Always | Attach to a local world started with the auxtools launcher. |
| **auxtools: Join with DreamSeeker** | Active session | Open DreamSeeker on the world (Windows) or copy its `byond://` address (Linux). |
| **auxtools: Join with DreamSeeker as a New Guest** | Active session | Assign a random key to the next guest login, then open DreamSeeker on the world (Windows) or copy its `byond://` address (Linux). |
| **auxtools: Reconnect Clients** | Active session | Ask connected clients to reconnect (or reboot a DreamSeeker-hosted world). |
| **auxtools: Reboot World** | Active session | Reboot the world in place with `world.Reboot()` without running `preLaunchTask`. |
| **auxtools: Profile Instructions** | Active session | Start or stop instruction profiling and open the report. |
| **auxtools: Show World Actions** | Active session | Open the status bar QuickPick menu (join, reconnect, restart, reboot, profile, stop, and **Show Problems** when a reload is blocked). |

`auxtools.pickWorld` is also registered for use as `"${command:auxtools.pickWorld}"` in
`.vscode/launch.json`.

### Settings

| Setting | Default | Description |
| --- | --- | --- |
| `auxtools.byondPath` | `""` | Path to the BYOND installation directory containing `bin`. Leave empty to search `BYOND_SYSTEM`, the Windows registry and `Program Files` (Windows), or `~/byond`, `/usr/local/byond`, and `/usr/share/byond` (Linux). |
| `auxtools.launcherPath` | `""` | Path to a custom `auxtools` launcher binary. Leave empty to use the bundled launcher. |
| `auxtools.debugLibrary` | `""` | Path to a custom debug library (`debug_server.dll` on Windows, `libdebug_server.so` on Linux). Leave empty to use the bundled library. |
| `auxtools.addToTerminalPath` | `true` | Prepend the launcher directory to `PATH` in VS Code terminals so CLI commands such as `auxtools log` and `auxtools check` work there. Open new terminals after changing this setting. |
| `auxtools.autoJoin` | `true` | Automatically open DreamSeeker when launching a world in DreamDaemon on Windows. |
| `auxtools.closeDreamSeeker` | `true` | Close DreamSeeker clients opened by the extension when their world process exits. |
| `auxtools.notifications` | All `true` | Toggle notification categories individually: `reconnect` (skin/script reload prompts), `join` (join or copy-address prompts when not auto-joined), `rebuild` (reboot/rebuild warnings), and `outdated` (files newer than the `.dmb` at startup). |

## License

GPL-3.0. See [LICENSE](LICENSE). The packaged extension includes the `auxtools` launcher and the
debug library.
