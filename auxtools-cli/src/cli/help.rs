use clap::error::{ContextKind, ContextValue, ErrorKind};

use super::{
	args::{self, Scope, Unparsed},
	Failure, Result
};

pub struct Page {
	pub path: &'static [&'static str],
	pub brief: &'static str,
	pub summary: &'static str,
	pub usage: &'static str,
	pub arguments: &'static str,
	pub sections: &'static str
}

const STEP_SECTIONS: &str = "Behavior:
  Requires a paused world. Resumes execution and waits until the world pauses
  again or the step completes. If the step pauses the world, it stays paused
  after this command exits. If the last frame returns, execution continues.

  On timeout, this command stops waiting and exits with code 2 without
  pausing or resuming the world. A timeout does not mean the step completed.

Output:
  Reports the execution fields of status: pid, paused, reason, message,
  caught, changes, stack, and instruction (populated only when stepping by
  instruction).

  On timeout, prints the last state the world reported if it answers within
  2 seconds. An absent pause does not guarantee the world is running.

Exit codes:
  0  The world paused, or the step completed.
  1  The step failed.
  2  The step timed out, or the world did not respond in time.
  3  The world exited or rebooted before the step completed.";

pub const PAGES: &[Page] = &[
	Page {
		path: &["launch"],
		brief: "Start a world with debugging and hot reload enabled",
		summary: "Start a compiled world with the debug library loaded.",
		usage: "auxtools launch DMB [OPTIONS] [-- ARGS...]",
		arguments: "DMB   Path to a compiled .dmb world.
ARGS  Arguments for the BYOND host program, replacing the defaults
      (-trusted, plus -close for dreamdaemon-gui on Windows).",
		sections: "Behavior:
  In the foreground, prints launcher and world output until the world exits.
  Press Ctrl+C once for a graceful shutdown, or twice to terminate the
  process.

  The launcher neither builds the .dmb nor pauses execution, and other
  auxtools commands or editors can attach while it runs. Use 'auxtools reboot'
  to load a rebuilt .dmb.

  If the debug library cannot start (for example, on an unsupported BYOND
  build), the launcher reports the reason and the world continues running
  without debugging until you stop it.

Output:
  In detached mode, prints the world PID, the launcher PID, and the launcher
  log file. JSON contains pid, launcher, log, and unavailable (the reason the
  debug library could not start, or null).

Exit codes:
  0  Detached world started, or foreground world exited with code 0.
  1  Launch failed.
  2  Detached startup did not complete within 120 seconds, so the world was
     stopped.
  4  Foreground world exited with a nonzero code or was terminated."
	},
	Page {
		path: &["list"],
		brief: "List local worlds with the debug library loaded",
		summary: "List local worlds with the debug library loaded.",
		usage: "auxtools list [--json]",
		arguments: "",
		sections: "Output:
  Text output prints a table of worlds, followed by any startup error lines.
  JSON output is an array of world records containing pid, program, dmb, args,
  cwd, protocol, compatible, responsive, and unavailable (empty if none are
  found).

  compatible is false when a world uses a different debug protocol. Attach
  using an auxtools CLI with the same protocol. Incompatible worlds are not
  queried, so their responsive field is null.

  responsive is false when a world does not answer within 5 seconds, leaving
  dmb, args, and cwd unknown. unavailable holds the reason the debug library
  could not start, or null. Such worlds can only be stopped."
	},
	Page {
		path: &["status"],
		brief: "Show execution state, uptime, and hot reload status",
		summary: "Show execution state, uptime, and the latest hot reload result.",
		usage: "auxtools [--pid PID] status [--json]",
		arguments: "",
		sections: "Output:
  Shows the execution state, pause details (including up to 20 stack frames)
  when paused, uptime, world.time, load count, and the latest hot reload
  result.

  JSON contains pid, paused, reason, message, caught, changes, stack,
  instruction, uptime_ms, world_time, loads, and last_reload. uptime_ms is the
  time since the latest world load, world_time is in deciseconds, loads counts
  world loads in this process (1 initially, plus 1 per reboot), and
  last_reload is a reload journal entry or null.

  When paused, reason is \"breakpoint\", \"step\", \"pause\", \"runtime\", or
  \"change\". A runtime error sets message and caught. A data breakpoint sets
  changes (entries with name, old, and new). instruction is always null here
  (it is set only by step-over --instruction and step-in --instruction).
  Inapplicable fields are null, or empty arrays for changes and stack. A
  running world reports paused=false with no pause details."
	},
	Page {
		path: &["stop"],
		brief: "Request graceful shutdown and wait for exit",
		summary: "Request graceful shutdown and wait for the world process to exit.",
		usage: "auxtools [--pid PID] stop [--timeout SECONDS] [--kill [--kill-timeout SECONDS]] [--json]",
		arguments: "",
		sections: "Behavior:
  Calls world.Del(). A world started by launch also shuts down its launcher.
  If world.Del() returns without ending the world, the debugger still shuts
  down. In a launched world, a runtime error in world.Del() leaves the debugger
  running, and stop reports the failure.
  Pass --kill with the first stop command. After world.Del() runs, the
  debugger may be gone, and a second stop may not reach the world.
  With --kill, the command terminates the process when --timeout passes, or at
  once when world.Del() fails. If the debug library could not start,
  world.Del() cannot run, and the command terminates the process at once.
  After termination, the process must exit within --kill-timeout.

Output:
  JSON contains pid, stopped, and killed (true when the process was forcibly
  terminated).

Exit codes:
  0  World process exited.
  1  Shutdown or forced termination failed.
  2  Graceful shutdown timed out without --kill, or forced termination timed
     out."
	},
	Page {
		path: &["reboot"],
		brief: "Reboot a world and wait for it to start running",
		summary: "Reboot the world and wait for it to start running again.",
		usage: "auxtools [--pid PID] reboot [--timeout SECONDS] [--json]",
		arguments: "",
		sections: "Behavior:
  Calls world.Reboot() to reload the .dmb, closing existing debug connections.
  In a world started by launch with hot reload, the launcher restores active
  reloads at startup. Reloads that need a resource absent from the build wait
  until world/New along with any later queued reloads.

  Returns once the new world starts running. To wait for a reboot without
  triggering one, use wait reboot.

Output:
  JSON contains pid and rebooted.

Exit codes:
  0  World rebooted and started running.
  1  Reboot request failed.
  2  Reboot timed out.
  3  World process exited instead of rebooting."
	},
	Page {
		path: &["reconnect"],
		brief: "Reconnect clients to load a changed skin or script",
		summary: "Reconnect clients to load the current skin and client script.",
		usage: "auxtools [--pid PID] reconnect [--timeout SECONDS] [--json]",
		arguments: "",
		sections: "Behavior:
  Hot reload sends an updated .dmf skin or client script only when clients log
  in, so already-connected clients keep the previous version. This command
  asks each connected client to reconnect (equivalent to DreamSeeker's
  .reconnect), causing each client to log out and back in.

  When DreamSeeker hosts the world, its window cannot reconnect without
  restarting the world, so this command reboots the world instead and waits
  for it to start running.

Output:
  JSON contains pid and either reconnected (the number of clients asked to
  reconnect) or rebooted (when DreamSeeker hosts the world).

Exit codes:
  0  Clients were asked to reconnect, or the world rebooted and started.
  1  Request failed.
  2  Reboot timed out, or the world did not respond in time.
  3  World process exited or connection ended."
	},
	Page {
		path: &["log"],
		brief: "Read or follow world journal events",
		summary: "Read or follow the world's event journal.",
		usage: "auxtools [--pid PID] log [OPTIONS]",
		arguments: "",
		sections: "Behavior:
  Shows all categories by default, or only the categories selected by flags.
  Entries appear oldest first, with entry numbers continuing across reboots.
  The journal retains only the latest 4,000 entries.

  --follow reconnects automatically across reboots and exits with code 0 when
  the world exits. Runtime errors are recorded without pausing execution.

  World output is captured only for worlds started by the launcher using
  dreamdaemon (dd.exe on Windows), because dreamseeker and dreamdaemon-gui do
  not pass their output to the launcher.

Output:
  Each JSON Lines entry contains number, time (Unix milliseconds), world_time
  (deciseconds), kind, and kind-specific fields. kind is \"runtime\",
  \"reload\", \"output\", \"world\", or \"spawned\".

Exit codes:
  0  Entries were printed, or with --follow, the world exited.
  1  Command failed.
  2  World did not respond in time."
	},
	Page {
		path: &["stack"],
		brief: "Show paused stack frames",
		summary: "Show stack frames for a paused world, innermost first.",
		usage: "auxtools [--pid PID] stack [--all] [--json]",
		arguments: "",
		sections: "Behavior:
  Requires a paused world.

Output:
  JSON is an array of frames, each with proc, file, line, offset, and
  old_code. With --all, JSON is an array of stacks, each with id, name,
  sleeping, and frames."
	},
	Page {
		path: &["vars"],
		brief: "Show frame arguments, locals, and optional globals",
		summary: "Show arguments and local variables for a paused stack frame.",
		usage: "auxtools [--pid PID] vars [--frame N] [--globals] [--json [--depth N]]",
		arguments: "",
		sections: "Behavior:
  Requires a paused world.

Output:
  Text output truncates values longer than 250 characters to their first 200
  characters and shows the full length. Use --json or eval for full values.

  JSON contains arguments, locals, and (with --globals) globals, encoded as in
  eval. Each variable consumes one level of --depth, so at the default depth
  lists expand while object variables do not."
	},
	Page {
		path: &["eval"],
		brief: "Evaluate a DM expression or proc body",
		summary: "Evaluate a DM expression in the world or a paused stack frame.",
		usage: "auxtools [--pid PID] eval EXPRESSION [OPTIONS]
auxtools [--pid PID] eval --file FILE [OPTIONS]
auxtools [--pid PID] eval - [OPTIONS]",
		arguments: "",
		sections: "Behavior:
  Provide exactly one input: EXPRESSION, --file FILE, or '-' to read stdin.
  Put '--' before an expression starting with '-' unless it is a number.

  In a paused world, expressions can access the selected frame's variables and
  src. In a running world, they can access globals and call procs. Separate
  multiple statements with newlines or ';' to form a proc body, which returns
  its explicit return value or last expression (or '.', if the final statement
  is an assignment, increment, or other non-value statement).

  If launch started the world with hot reload, expressions can use the
  project's #define macros. Evaluation can mutate world state through
  assignments and proc calls. An error in the expression exits with code 1 and
  prints the location and call stack when known. A runtime error inside a
  called proc terminates only that proc (returning null to its caller as in
  DM), prints to stderr, and lets evaluation continue.

  Direct evaluation cannot sleep or move to the background (such as via
  world.loop_checks or set background = 1). Pass --spawn to run such code on a
  launched world. Spawned code runs outside any frame (use --src to set src),
  pauses at breakpoints, runs after resuming if the world is paused, and logs
  its result to the journal (readable via log --spawned). In a launched world,
  auxtools_kept() persists values between evaluations until reboot (for
  example, auxtools_kept()[\"m\"] = m).

Output:
  JSON emits numbers, text, and null directly. Non-finite numbers become
  {number: \"inf\"}, {number: \"-inf\"}, or {number: \"nan\"}. Lists without
  associated values become arrays, lists whose keys are all distinct strings
  become objects, and all other lists become arrays of {key, value} pairs.
  Objects are {ref, type, name} (plus vars when expanded), type paths are
  {path}, and other values are {kind, text}.

  --depth controls expansion depth for direct evaluation: lists expand at
  depth 1+, object variables at depth 2+, and each nesting level consumes one
  depth level. Unexpanded lists render as {list: true, len}, and unexpanded
  objects omit vars. With --spawn and no --wait, outputs the spawn number
  (JSON: {spawn}). With --spawn --wait --json, results always render at depth
  1 regardless of --depth.

Exit codes:
  0  Evaluation completed, or the code was spawned.
  1  Evaluation failed, or with --wait, the spawned code failed.
  2  --wait timed out, or the world did not respond in time (spawned code
     continues running).
  3  World exited or rebooted before evaluation or --wait finished."
	},
	Page {
		path: &["proc"],
		brief: "Show a proc's location, definitions, and reload status",
		summary: "Show a proc's source location, definition count, and hot reload status.",
		usage: "auxtools [--pid PID] proc /PATH[#N] [--json]",
		arguments: "/PATH  Proc path, with or without /proc/ (for example, /mob/proc/attack
       or /mob/attack). For unnamed procs, use the stack trace name:
       (global init), (in list), or (var initializer).
#N     Zero-based definition index. [default: 0]",
		sections: "Output:
  JSON contains proc, file, line, definitions, and reloaded (null, \"lines\",
  \"code\", or \"added\")."
	},
	Page {
		path: &["disassemble"],
		brief: "Disassemble a proc's current bytecode",
		summary: "Disassemble a proc's current bytecode.",
		usage: "auxtools [--pid PID] disassemble /PATH[#N] [--json]",
		arguments: "/PATH  Proc path, with or without /proc/, or an unnamed proc name such as
       (global init).
#N     Zero-based definition index. [default: 0]",
		sections: "Output:
  Lists each instruction with its bytecode offset, text, and source line,
  reflecting any hot-reloaded bytecode.

  JSON contains proc, file, and instructions (entries with offset, text, and
  line)."
	},
	Page {
		path: &["pause"],
		brief: "Pause execution and report the current location",
		summary: "Pause execution and report the current location.",
		usage: "auxtools [--pid PID] pause [--hold-max SECONDS] [--json]",
		arguments: "",
		sections: "Behavior:
  Keeps the world paused after this command exits. Use continue to resume, or
  pass --hold-max to resume automatically after SECONDS with no debugger
  connected. If the world is already paused, keeps the pause and reports it.

Output:
  Reports the execution fields of status: pid, paused, reason, message,
  caught, changes, stack, and instruction."
	},
	Page {
		path: &["continue"],
		brief: "Resume a paused world",
		summary: "Resume execution of a paused world.",
		usage: "auxtools [--pid PID] continue [--json]",
		arguments: "",
		sections: "Behavior:
  Requires a paused world. Returns once execution resumes without waiting for
  another pause.

Output:
  JSON contains pid and resumed."
	},
	Page {
		path: &["step-over"],
		brief: "Advance to the next line, stepping over proc calls",
		summary: "Advance to the next source line, stepping over proc calls.",
		usage: "auxtools [--pid PID] step-over [--instruction] [--timeout SECONDS] [--hold-max SECONDS] [--json]",
		arguments: "",
		sections: STEP_SECTIONS
	},
	Page {
		path: &["step-in"],
		brief: "Advance to the next line, stepping into proc calls",
		summary: "Advance to the next source line, stepping into proc calls.",
		usage: "auxtools [--pid PID] step-in [--instruction] [--timeout SECONDS] [--hold-max SECONDS] [--json]",
		arguments: "",
		sections: STEP_SECTIONS
	},
	Page {
		path: &["step-out"],
		brief: "Run until the current proc returns",
		summary: "Run until the current proc returns to its caller.",
		usage: "auxtools [--pid PID] step-out [--timeout SECONDS] [--hold-max SECONDS] [--json]",
		arguments: "",
		sections: STEP_SECTIONS
	},
	Page {
		path: &["wait"],
		brief: "Wait for a pause, hot reload, or reboot",
		summary: "Wait for a pause, hot reload, or reboot.",
		usage: "auxtools [--pid PID] wait pause [OPTIONS]
auxtools [--pid PID] wait reload [OPTIONS]
auxtools [--pid PID] wait reboot [OPTIONS]",
		arguments: "",
		sections: "Commands:
  pause   Wait for any pause, or for breakpoints, changes, or runtime errors
  reload  Wait for a hot reload attempt and report its result
  reboot  Wait for a reboot and for the debug server to be ready

Specify pause, reload, or reboot. 'auxtools wait' alone prints this help and
exits with code 1. Run 'auxtools wait COMMAND --help' for options."
	},
	Page {
		path: &["wait", "pause"],
		brief: "Wait for any pause, or for breakpoints, changes, or runtime errors",
		summary: "Wait for execution to pause and report the location and reason.",
		usage: "auxtools [--pid PID] wait pause [OPTIONS]",
		arguments: "",
		sections: "Behavior:
  Without --break, --change, or --runtimes (including flags that imply it),
  waits for any pause. With one or more of them, waits only for pauses they
  cause and ignores others.

  If the world is already paused and --change is not given, returns
  immediately (after validating and placing any --break locations). With
  --change on a paused world, resolves data breakpoints in the selected frame,
  resumes execution, and waits. Data breakpoints support object variables,
  list entries, globals, and proc variables. All temporary breakpoints and
  data breakpoints are removed on exit.

  Invalid breakpoint locations fail before waiting begins, and resolved
  breakpoint locations are printed to stderr and included in JSON output. By
  default, the world stays paused after this command exits. Pass --no-hold to
  skip holding the pause (another debugger or an existing hold can still keep
  the world paused).

  --if does not filter runtime errors or pauses from other clients.
  --runtime-filter matches messages by comma-separated substrings: 'bounds,
  null' matches either, and '!libdreamluau' excludes matching messages.

Input:
  For EXPR and CONDITION, @FILE reads the text from FILE. Write @@ for text
  starting with @.

Output:
  Reports the execution fields of status plus already (true if already
  paused), breakpoints (requested and resolved locations), and watching
  (data breakpoint targets and initial values).

Exit codes:
  0  World paused, or was already paused.
  1  Setup or command failed.
  2  Wait timed out, or the world did not respond in time.
  3  World exited or rebooted before a pause was reported.

Examples:
  auxtools --pid 1234 wait pause --break code/combat.dm:42 --timeout 30 --json
  auxtools --pid 1234 wait pause --change src.health --if \"value < old\" --json
  auxtools --pid 1234 wait pause --runtimes --hold-max 60 --json
  auxtools --pid 1234 wait pause --runtime-filter '!libdreamluau' --json"
	},
	Page {
		path: &["wait", "reload"],
		brief: "Wait for a hot reload attempt and report its result",
		summary: "Wait for a hot reload attempt and report whether it was applied.",
		usage: "auxtools [--pid PID] wait reload [--since N] [--timeout SECONDS] [--json]",
		arguments: "",
		sections: "Behavior:
  Waits for the next hot reload attempt without triggering a reload or pausing
  the world. Start this command before saving source files, or pass --since N
  with an earlier journal entry number to match a reload that has already
  finished (while that entry remains in the journal). Reconnects across
  reboots.

Output:
  One reload journal entry, with the same fields as log --reloads.

Exit codes:
  0  Reload was applied.
  1  Reload was rejected or partially applied, or the command failed.
  2  Wait timed out, or the world did not respond in time.
  3  World exited before the reload result arrived."
	},
	Page {
		path: &["wait", "reboot"],
		brief: "Wait for a reboot and for the debug server to be ready",
		summary: "Wait for the world to reboot and for its debug server to be ready.",
		usage: "auxtools [--pid PID] wait reboot [--since N] [--timeout SECONDS] [--json]",
		arguments: "",
		sections: "Behavior:
  Waits for a reboot without starting one or pausing the world. After the
  reboot, reconnects and returns once the new world starts running (with
  startup hot reload changes restored). With --since N, a world-load entry
  already in the journal after N ends the wait immediately.

Output:
  One world-load journal entry, with the same fields as log --world.

Exit codes:
  0  World loaded and its debug server responded.
  1  Command failed.
  2  Wait timed out, or the world did not respond in time.
  3  World exited before the reboot completed."
	},
	Page {
		path: &["trace"],
		brief: "Log or count breakpoint hits without pausing",
		summary: "Log or count breakpoint hits without pausing execution.",
		usage: "auxtools [--pid PID] trace LOCATION... [OPTIONS]",
		arguments: "LOCATION  One or more FILE:LINE or /proc/path[#N] locations.
          #N selects a zero-based definition index. [default: 0]",
		sections: "Behavior:
  Prints the location and caller of each hit by default. With --by, suppresses
  per-hit lines and prints counts at exit, highest first (--by and --log
  cannot be used together). With both --for and --count, stops at whichever
  limit is reached first.

  If --if fails with an error, the hit is skipped, counted as skipped, and the
  error is printed once. If --by EXPR fails, the hit is counted under the
  error text. Temporary breakpoints are removed on exit.

Input:
  For MESSAGE, CONDITION, and EXPR, @FILE reads the text from FILE. Write @@
  for text starting with @.

Exit codes:
  0  Duration or hit limit reached, or interrupted with Ctrl+C.
  1  Setup or command failed.
  2  World did not respond in time.
  3  World exited or rebooted."
	},
	Page {
		path: &["profile"],
		brief: "Profile instruction execution time",
		summary: "Measure execution time by proc, source line, and instruction.",
		usage: "auxtools [--pid PID] profile [/PATH[#N]...] [OPTIONS]",
		arguments: "/PATH  One or more procs to measure, with or without /proc/.
       [default: every proc]
#N     Zero-based definition index. [default: 0]",
		sections: "Behavior:
  Profiles until --for expires or you press Ctrl+C, then prints the report.
  Profiling slows execution, and only one profile can run in a world at a
  time.

  Includes code run by eval, while excluding sleeping time and BYOND work
  between DM runs. Each instruction's self time lasts until the next
  instruction begins. Time in called procs is tracked separately and excludes
  calls that sleep. Profiler overhead is estimated from the cheapest sampled
  instruction's average time and subtracted.

Output:
  Without proc paths, lists the slowest procs and source lines (up to --top of
  each). With proc paths, prints the proc table followed by per-instruction
  listings grouped by source line, showing offset, hits, share of proc self
  time, nanoseconds per hit, and average call time per hit.

  --html also writes an interactive HTML report. --json emits one JSON object
  with seconds, ended (\"time\" or \"interrupted\"), overhead_ns,
  instructions, ns, and procs. Each proc has proc, override, old_code, file,
  line, calls, instructions, ns, callee_ns, and listing (entries with offset,
  text, line, source, hits, ns, and callee_ns). Without proc paths, listing is
  null for procs outside --top.

Exit codes:
  0  Duration reached, or interrupted with Ctrl+C.
  1  Setup or command failed.
  2  World did not respond in time.
  3  World exited or rebooted."
	},
	Page {
		path: &["check"],
		brief: "Check project source for known BYOND bugs",
		summary: "Report known BYOND bugs detected in project source.",
		usage: "auxtools check [DME] [OPTIONS]",
		arguments: "DME  Path to the .dme project file.
     [default: the only .dme in the current directory]",
		sections: "Behavior:
  Compiles every proc and reports known bugs affecting the selected BYOND
  build, without writing a .dmb or requiring BYOND to be installed.

  Parse errors and procs that fail to compile are reported as not checked
  without changing the exit code. Text output lists them on stderr and JSON
  output includes them.

Output:
  Lists each issue with its file, optional line number, proc, and message
  (paths relative to the project directory), followed by a summary of issues
  and checked procs (printed to stderr when issues are found).

  JSON contains byond, procs (checked count), found, not_checked, and
  parse_errors. found entries have id (stable across wording changes), file,
  line, proc, override, and text. not_checked entries have file, proc,
  override, and error. parse_errors entries are FILE:LINE: TEXT strings.

Exit codes:
  0  No known bugs found.
  1  Project could not be read.
  5  Known BYOND bugs found."
	},
	Page {
		path: &["dap"],
		brief: "Serve the Debug Adapter Protocol over stdio",
		summary: "Serve the Debug Adapter Protocol over stdin and stdout.",
		usage: "auxtools dap",
		arguments: "",
		sections: "Behavior:
  Handles editor requests to launch or attach to worlds, debug them, and
  hot-reload source changes. Writes only DAP messages to stdout."
	},
	Page {
		path: &["help"],
		brief: "Show command help",
		summary: "Show top-level or command-specific help.",
		usage: "auxtools help [COMMAND [SUBCOMMAND]]",
		arguments: "",
		sections: "Equivalent forms:
  auxtools --help
  auxtools COMMAND --help
  auxtools COMMAND SUBCOMMAND --help"
	},
	Page {
		path: &["version"],
		brief: "Show CLI and debug protocol versions",
		summary: "Show CLI and debug protocol versions.",
		usage: "auxtools version [--json]",
		arguments: "",
		sections: "Output:
  JSON contains version and protocol.

Aliases: --version, -V"
	}
];

const MORE_ROWS: &[(&str, &str, &str, &str)] = &[("trace", "by", "--by caller", "Count hits by calling proc and source line.")];

pub fn overview() -> String {
	let mut text = String::from(
		"auxtools - Launch, inspect, and debug BYOND worlds

Usage:
  auxtools [--pid PID] COMMAND [ARGS] [OPTIONS]
  auxtools help [COMMAND [SUBCOMMAND]]
"
	);
	let width = args::GROUPS
		.iter()
		.flat_map(|(_, commands)| commands.iter())
		.map(|(name, _)| name.len())
		.max()
		.unwrap_or(0);
	for (group, commands) in args::GROUPS {
		text += &format!("\n{}:\n", group);
		for (name, _) in *commands {
			let brief = page(&[name]).map_or("", |page| page.brief);
			text += &format!("  {:width$}  {}\n", name, brief);
		}
	}
	text += &format!(
		"
World selection:
  --pid PID    Select a world process. When omitted, uses the only compatible
               world. Required when multiple compatible worlds are running.
               Run 'auxtools list' to find process IDs.

Responses:
  --response-timeout SECONDS
               {response_timeout}
               Exits with code 2 if the world does not respond in time. Does
               not limit event waits. Use --timeout or --for instead
               (log --follow waits indefinitely).

Output:
  --json       Emit JSON instead of text where supported. log and trace emit
               JSON Lines. Other commands emit a single JSON value. See each
               command's help for its fields.

Pauses:
  pause, step-over, step-in, step-out, and wait pause keep the world paused
  after they exit. Use continue to resume. --hold-max SECONDS resumes the
  world after SECONDS with no debugger connected.
  [default: AUXTOOLS_HOLD_MAX, or unlimited if unset]

Help:
  auxtools COMMAND --help
  auxtools help COMMAND

Exit codes:
  0  Command completed successfully.
  1  Command failed.
  2  Operation timed out.
  3  World connection ended before the operation completed.
  4  Foreground launch: world exited with a nonzero code or was terminated.
  5  check: known BYOND bugs found.

log --follow, wait reload, and wait reboot reconnect across reboots.
",
		response_timeout = args::response_timeout_help()
	);
	text
}

pub fn page(path: &[&str]) -> Option<&'static Page> {
	PAGES.iter().find(|page| page.path == path)
}

fn indented(lines: &str) -> String {
	lines.lines().map(|line| format!("  {}\n", line)).collect()
}

fn spec(arg: &clap::Arg) -> String {
	let mut names: Vec<String> = arg.get_short().iter().map(|short| format!("-{}", short)).collect();
	names.extend(arg.get_long().iter().map(|long| format!("--{}", long)));
	let mut text = names.join(", ");
	if arg.get_action().takes_values() {
		for name in arg.get_value_names().unwrap_or_default() {
			text += &format!(" {}", name);
		}
	}
	text
}

fn option_rows(command: &clap::Command, path: &[&str]) -> Vec<(String, String)> {
	let row = |arg: &clap::Arg| (spec(arg), arg.get_help().map(ToString::to_string).unwrap_or_default());
	let shared = |id: &str| command.get_arguments().find(|arg| arg.get_id() == id).map(row);
	let mut rows = vec![];
	let mut common_json = false;
	for arg in command.get_arguments() {
		let id = arg.get_id().as_str();
		if arg.is_positional() || arg.is_hide_set() || matches!(id, "help" | "pid" | "response_timeout") {
			continue;
		}
		if id == "json" && arg.get_help().is_some_and(|help| help.to_string() == args::JSON) {
			common_json = true;
			continue;
		}
		rows.push(row(arg));
		for (_, _, more, help) in MORE_ROWS.iter().filter(|(name, after, ..)| path == [*name] && *after == id) {
			rows.push(((*more).to_owned(), (*help).to_owned()));
		}
	}
	let scope = path.first().and_then(|name| args::scope_of(name));
	if matches!(scope, Some(Scope::World | Scope::WorldExit)) {
		rows.extend(shared("pid"));
	}
	if scope == Some(Scope::World) {
		rows.extend(shared("response_timeout"));
	}
	if common_json {
		rows.extend(shared("json"));
	}
	rows.push(("-h, --help".to_owned(), "Show help for this command.".to_owned()));
	rows
}

pub fn render(page: &Page, command: &clap::Command) -> String {
	let mut text = format!("{}\n\nUsage:\n{}", page.summary, indented(page.usage));
	if !page.arguments.is_empty() {
		text += &format!("\nArguments:\n{}", indented(page.arguments));
	}
	if !command.has_subcommands() {
		let rows = option_rows(command, page.path);
		let width = rows.iter().map(|(spec, _)| spec.len()).max().unwrap_or(0);
		text += "\nOptions:\n";
		for (spec, help) in rows {
			let mut lines = help.lines();
			text += &format!("  {:width$}  {}\n", spec, lines.next().unwrap_or_default());
			for line in lines {
				text += &format!("  {:width$}  {}\n", "", line);
			}
		}
	}
	if !page.sections.is_empty() {
		text += &format!("\n{}\n", page.sections);
	}
	text
}

pub fn attach(command: clap::Command) -> clap::Command {
	fn attached(mut command: clap::Command, path: &[&str]) -> clap::Command {
		let names: Vec<String> = command.get_subcommands().map(|sub| sub.get_name().to_owned()).collect();
		for name in &names {
			let mut below = path.to_vec();
			below.push(name);
			command = command.mut_subcommand(name, |sub| attached(sub, &below));
		}
		match page(path) {
			Some(page) => {
				let text = render(page, &command);
				command.about(page.brief).override_help(text)
			}
			None if path.is_empty() => command.override_help(overview()),
			None => command
		}
	}
	let mut command = command;
	command.build();
	attached(command, &[])
}

pub fn show(path: &[String]) -> Result<()> {
	if path.is_empty() {
		say_part!("{}", overview());
		return Ok(());
	}
	let root = args::command();
	let mut command = &root;
	for (i, name) in path.iter().enumerate() {
		command = command
			.find_subcommand(name)
			.ok_or_else(|| Failure(1, unknown_command(&path[..i].iter().map(String::as_str).collect::<Vec<_>>(), name)))?;
	}
	let names: Vec<&str> = path.iter().map(String::as_str).collect();
	match page(&names) {
		Some(page) => say_part!("{}", render(page, command)),
		None => return Err(Failure(1, format!("no help page for '{}'.", path.join(" "))))
	}
	Ok(())
}

fn unknown_command(parents: &[&str], name: &str) -> String {
	match parents {
		[] => format!("unknown command '{}'. Run 'auxtools --help' for available commands.", name),
		_ => format!(
			"unknown {} command '{}'. Run 'auxtools {} --help' for available commands.",
			parents.join(" "),
			name,
			parents.join(" ")
		)
	}
}

fn command_path<'a>(root: &'a clap::Command, args: &[String]) -> Vec<&'a str> {
	let mut path = vec![];
	let mut command = root;
	let mut args = args.iter();
	while let Some(arg) = args.next() {
		if arg == "--pid" {
			args.next();
			continue;
		}
		if arg.starts_with('-') {
			continue;
		}
		match command.find_subcommand(arg) {
			Some(sub) => command = sub,
			None => break
		}
		path.push(command.get_name());
		if !command.has_subcommands() {
			break;
		}
	}
	path
}

pub fn unparsed(error: &clap::Error, args: &[String], root: &clap::Command) -> Unparsed {
	if matches!(error.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand) {
		return Unparsed::Help(error.render().to_string());
	}
	let path = command_path(root, args);
	let text = |kind: ContextKind| match error.get(kind) {
		Some(ContextValue::String(text)) => Some(text.clone()),
		Some(ContextValue::Strings(texts)) => Some(texts.join(", ")),
		Some(other) => Some(other.to_string()),
		None => None
	};
	let option = |spec: String| spec.split([' ', '=']).next().unwrap_or_default().to_owned();
	let value_name = |spec: &str| spec.split_once(' ').map(|(_, value)| value.trim_matches(['<', '>', '.']).to_owned());
	let usage = match path.as_slice() {
		[] => "Run 'auxtools --help' for usage.".to_owned(),
		_ => format!("Run 'auxtools {} --help' for usage.", path.join(" "))
	};
	let message = match error.kind() {
		ErrorKind::InvalidSubcommand => return Unparsed::Usage(unknown_command(&path, &text(ContextKind::InvalidSubcommand).unwrap_or_default())),
		ErrorKind::UnknownArgument => {
			let argument = text(ContextKind::InvalidArg).unwrap_or_default();
			match path.as_slice() {
				["wait"] => format!(
					"unexpected argument '{}' before a wait command. Specify pause, reload, or reboot first.",
					argument
				),
				_ => format!("unexpected argument '{}'.", argument)
			}
		}
		ErrorKind::InvalidValue | ErrorKind::ValueValidation | ErrorKind::NoEquals | ErrorKind::TooFewValues | ErrorKind::WrongNumberOfValues => {
			let spec = text(ContextKind::InvalidArg).unwrap_or_default();
			let value = text(ContextKind::InvalidValue).unwrap_or_default();
			let why = std::error::Error::source(error).map(ToString::to_string);
			match (value.is_empty(), why) {
				(true, _) => match value_name(&spec) {
					Some(name) => format!("{} requires a value: {}.", option(spec), name),
					None => format!("{} requires a value.", option(spec))
				},
				(false, Some(why)) => format!("invalid value '{}' for {}: {}.", value, option(spec), why),
				(false, None) => format!("invalid value '{}' for {}.", value, option(spec))
			}
		}
		ErrorKind::MissingRequiredArgument => {
			let missing = text(ContextKind::InvalidArg).unwrap_or_default();
			format!("missing required argument: {}.", missing.replace(['<', '>'], "").replace("...", ""))
		}
		ErrorKind::ArgumentConflict => {
			let first = option(text(ContextKind::InvalidArg).unwrap_or_default());
			match text(ContextKind::PriorArg).map(option) {
				Some(second) if second != first => format!("{} and {} cannot be used together.", second, first),
				_ => format!("{} cannot be used more than once.", first)
			}
		}
		_ => {
			let rendered = error.render().to_string();
			let line = rendered.lines().next().unwrap_or_default();
			format!("{}.", line.trim_start_matches("error: ").trim_end_matches('.'))
		}
	};
	Unparsed::Usage(format!("{} {}", message, usage))
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use auxtools_cli::{byond::Server, local::PROBE_PATIENCE};

	use super::{command_path, overview, page, render, spec, PAGES};
	use crate::cli::{
		args::{self, Scope},
		check, hold, json, launch, HUNG_UP, TIMED_OUT
	};

	fn commands() -> Vec<(Vec<String>, clap::Command)> {
		fn under(command: &clap::Command, path: &[String], found: &mut Vec<(Vec<String>, clap::Command)>) {
			for sub in command.get_subcommands() {
				let mut path = path.to_vec();
				path.push(sub.get_name().to_owned());
				found.push((path.clone(), sub.clone()));
				under(sub, &path, found);
			}
		}
		let mut found = vec![];
		under(&args::command(), &[], &mut found);
		found
	}

	fn names(path: &[String]) -> Vec<&str> {
		path.iter().map(String::as_str).collect()
	}

	fn options_named(text: &str) -> Vec<(&str, &str)> {
		let mut found = vec![];
		let mut at = 0;
		while let Some(offset) = text[at..].find('-') {
			let start = at + offset;
			let rest = &text[start..];
			let hyphens = if rest.starts_with("--") { 2 } else { 1 };
			let name = rest[hyphens..]
				.split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
				.next()
				.unwrap_or_default()
				.trim_end_matches('-');
			at = start + hyphens + name.len();
			if text[..start].ends_with(|c: char| c.is_alphanumeric() || c == '-') {
				continue;
			}
			let long = hyphens == 2 && name.starts_with(|c: char| c.is_ascii_lowercase());
			let short = hyphens == 1 && name.len() == 1 && name.starts_with(|c: char| c.is_ascii_alphabetic());
			if long || short {
				let before = text[..start].split_whitespace().next_back().unwrap_or_default();
				found.push((before.trim_matches(|c: char| !c.is_ascii_alphanumeric()), &rest[..hyphens + name.len()]));
			}
		}
		found
	}

	fn takes(command: &clap::Command, option: &str) -> bool {
		command.get_arguments().any(|arg| {
			arg.get_long().is_some_and(|long| format!("--{}", long) == option) || arg.get_short().is_some_and(|short| format!("-{}", short) == option)
		})
	}

	fn defaults_stated(text: &str) -> Vec<&str> {
		text.split("[default: ")
			.skip(1)
			.map(|rest| rest.split(']').next().unwrap_or_default())
			.collect()
	}

	fn default_in_code(path: &[&str], id: &str) -> Option<String> {
		let seconds = |default: Duration| default.as_secs_f64().to_string();
		Some(match (path, id) {
			(_, "response_timeout") => seconds(args::DEFAULT_RESPONSE_TIMEOUT),
			(["stop"], "timeout") => seconds(args::DEFAULT_STOP_TIMEOUT),
			(["stop"], "kill_timeout") => seconds(args::DEFAULT_KILL_TIMEOUT),
			(["reboot" | "reconnect"], "timeout") => seconds(args::DEFAULT_REBOOT_TIMEOUT),
			(["vars" | "eval"], "depth") => json::DEFAULT_DEPTH.to_string(),
			(["profile"], "top") => args::DEFAULT_TOP.to_string(),
			(["check"], "byond_version") => {
				let latest = dmasm::ByondVersion::LATEST;
				format!("{}.{}", latest.major, latest.build)
			}
			(_, "hold_max") => hold::HOLD_MAX_VAR.to_owned(),
			(["vars" | "eval"] | ["wait", "pause"], "frame") | (["wait", "pause"], "breaks") | (_, "#N") => "0".to_owned(),
			_ => return None
		})
	}

	#[test]
	fn every_command_has_a_page_and_every_page_a_command() {
		let commands = commands();
		for (path, _) in &commands {
			let names: Vec<&str> = path.iter().map(String::as_str).collect();
			assert!(page(&names).is_some(), "{} has no page", path.join(" "));
		}
		assert_eq!(commands.len(), PAGES.len(), "a page is of no command");
		for gone in ["next", "step", "out", "dis", "--list", "run"] {
			assert!(page(&[gone]).is_none(), "{}", gone);
		}
	}

	#[test]
	fn the_first_page_lists_every_command_once() {
		let help = overview();
		let root = args::command();
		let width = root.get_subcommands().map(|command| command.get_name().len()).max().unwrap();
		for command in root.get_subcommands() {
			let name = command.get_name();
			let page = page(&[name]).unwrap();
			assert_eq!(
				help.matches(&format!("\n  {:width$}  {}\n", name, page.brief)).count(),
				1,
				"{} isn't in the help once",
				name
			);
			assert!(!page.brief.ends_with('.'), "{}", page.brief);
		}
	}

	#[test]
	fn a_command_with_commands_lists_each_as_its_page_does() {
		for (path, command) in commands() {
			let mut names = names(&path);
			let text = render(page(&names).unwrap(), &command);
			let width = command.get_subcommands().map(|sub| sub.get_name().len()).max().unwrap_or(0);
			for sub in command.get_subcommands() {
				names.push(sub.get_name());
				let brief = page(&names).unwrap().brief;
				names.pop();
				assert!(
					text.contains(&format!("\n  {:width$}  {}\n", sub.get_name(), brief)),
					"{} doesn't list {}: {}",
					path.join(" "),
					sub.get_name(),
					text
				);
			}
		}
	}

	#[test]
	fn a_page_starts_with_what_the_command_does_and_its_usage() {
		for (path, command) in commands() {
			let names: Vec<&str> = path.iter().map(String::as_str).collect();
			let page = page(&names).unwrap();
			let text = render(page, &command);
			assert!(text.starts_with(&format!("{}\n\nUsage:\n  auxtools ", page.summary)), "{}", text);
			assert!(page.summary.ends_with('.'), "{}", page.summary);
			for usage in page.usage.lines() {
				assert!(usage.contains(&format!(" {}", path.join(" "))), "{}", usage);
				let selects = args::scope_of(names[0]) != Some(Scope::Local);
				assert_eq!(usage.starts_with("auxtools [--pid PID] "), selects, "{}", usage);
			}
			if command.has_subcommands() {
				assert!(!text.contains("\nOptions:\n"), "{}", text);
				continue;
			}
			assert!(text.contains("  -h, --help"), "{}", text);
			assert_eq!(text.contains("  --pid PID "), args::scope_of(names[0]) != Some(Scope::Local), "{}", text);
			assert!(!text.contains("--detached"), "{}", text);
		}
	}

	#[test]
	fn every_option_is_in_its_commands_page_and_says_what_it_does() {
		for (path, command) in commands() {
			if command.has_subcommands() {
				continue;
			}
			let names = names(&path);
			let text = render(page(&names).unwrap(), &command);
			let scope = args::scope_of(names[0]);
			for arg in command.get_arguments().filter(|arg| !arg.is_positional()) {
				let id = arg.get_id().as_str();
				let spec = spec(arg);
				let listed = match id {
					"pid" => scope != Some(Scope::Local),
					"response_timeout" => scope == Some(Scope::World),
					_ => !arg.is_hide_set()
				};
				let row = format!("  {}  ", spec);
				assert_eq!(
					text.lines().filter(|line| line.starts_with(&row)).count(),
					usize::from(listed),
					"{} {}: {}",
					path.join(" "),
					spec,
					text
				);
				if !listed || id == "help" {
					continue;
				}
				let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
				for line in help.lines() {
					assert!(text.contains(&format!("  {}\n", line)), "{} {}: {:?}", path.join(" "), spec, line);
				}
				assert!(
					help.chars().next().is_some_and(char::is_uppercase),
					"{} {}: {:?}",
					path.join(" "),
					spec,
					help
				);
				assert!(help.ends_with('.') || help.ends_with(']'), "{} {}: {:?}", path.join(" "), spec, help);
			}
		}
	}

	#[test]
	fn a_usage_line_names_every_option_or_says_options() {
		for (path, command) in commands() {
			let page = page(&names(&path)).unwrap();
			if command.has_subcommands() || page.usage.contains("[OPTIONS]") {
				continue;
			}
			let own = command
				.get_arguments()
				.filter(|arg| !arg.is_positional() && !arg.is_hide_set() && !matches!(arg.get_id().as_str(), "help" | "pid" | "response_timeout"));
			for arg in own {
				let spec = spec(arg);
				assert!(
					page.usage.contains(&format!("[{}]", spec)) || page.usage.contains(&format!("[{} [", spec)),
					"the usage of {} leaves out {}: {}",
					path.join(" "),
					spec,
					page.usage
				);
			}
		}
	}

	#[test]
	fn a_page_names_no_option_that_doesnt_exist() {
		let root = args::command();
		let commands = commands();
		for (path, command) in &commands {
			let text = render(page(&names(path)).unwrap(), command);
			for (before, option) in options_named(&text) {
				let others = root.find_subcommand(before).is_some_and(|other| takes(other, option));
				assert!(
					takes(command, option) || takes(&root, option) || others,
					"the page of {} names {}, which isn't an option of it: {}",
					path.join(" "),
					option,
					text
				);
			}
		}
		let first = overview();
		for (_, option) in options_named(&first) {
			assert!(
				takes(&root, option) || commands.iter().any(|(_, command)| takes(command, option)),
				"the first page names {}, which isn't an option",
				option
			);
		}
		assert_eq!(options_named("Put '--' before -x (--wait), log --follow; step-over -trusted Ctrl-C"), [
			("before", "-x"),
			("", "--wait"),
			("log", "--follow")
		]);
	}

	#[test]
	fn a_default_the_help_states_is_the_one_the_code_uses() {
		let check = |path: &[&str], id: &str, stated: &str| match default_in_code(path, id) {
			Some(value) => assert!(
				stated
					.strip_prefix(value.as_str())
					.is_some_and(|rest| rest.is_empty() || rest.starts_with([',', ' '])),
				"{} {} says [default: {}], and the code uses {}",
				path.join(" "),
				id,
				stated,
				value
			),
			None => assert!(
				!stated.starts_with(|c: char| c.is_ascii_digit()),
				"{} {} says [default: {}]: name the value the code uses in `default_in_code`",
				path.join(" "),
				id,
				stated
			)
		};
		for (path, command) in commands() {
			let names = names(&path);
			let page = page(&names).unwrap();
			for arg in command.get_arguments() {
				let id = arg.get_id().as_str();
				let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
				let stated = defaults_stated(&help);
				if arg.get_action().takes_values() && !arg.get_default_values().is_empty() {
					let given: Vec<_> = arg.get_default_values().iter().map(|value| value.to_string_lossy()).collect();
					assert_eq!(stated, given, "{} {}", path.join(" "), id);
				}
				for stated in stated {
					if names == ["launch"] && id == "server" {
						let server: Server = serde_json::from_value(serde_json::json!(stated)).expect("a server by its name");
						assert_eq!(server, Server::default_for_platform());
						continue;
					}
					check(&names, id, stated);
				}
			}
			for line in page.arguments.lines().chain(page.sections.lines()) {
				let id = if line.trim_start().starts_with("#N ") { "#N" } else { "" };
				for stated in defaults_stated(line) {
					check(&names, id, stated);
				}
			}
		}
		let first = overview();
		assert!(
			first.contains(&format!("\n               {}\n", args::response_timeout_help())),
			"{}",
			first
		);
		assert_eq!(defaults_stated(&first).len(), 2, "{}", first);
		assert!(
			first.contains(&format!("[default: {}, or unlimited if unset]", hold::HOLD_MAX_VAR)),
			"{}",
			first
		);
	}

	#[test]
	fn the_times_and_exit_codes_in_the_prose_are_the_commands() {
		let sections = |name: &str| page(&[name]).unwrap().sections;
		let within = |time: Duration| format!("within {} seconds", time.as_secs());
		assert!(sections("launch").contains(&within(launch::COMING_UP)), "{}", sections("launch"));
		assert!(sections("list").contains(&within(PROBE_PATIENCE)), "{}", sections("list"));

		fn codes(text: &str) -> Vec<i32> {
			let listed = text.split("Exit codes:\n").nth(1).unwrap_or_default();
			listed
				.lines()
				.take_while(|line| !line.is_empty())
				.filter_map(|line| line.strip_prefix("  ")?.split("  ").next()?.parse().ok())
				.collect()
		}
		let first = codes(&overview());
		assert_eq!(first, [0, 1, TIMED_OUT, HUNG_UP, launch::WORLD_FAILED, check::FOUND]);
		for page in PAGES {
			for code in codes(page.sections) {
				assert!(first.contains(&code), "{}: {}", page.path.join(" "), code);
				assert!(
					code != launch::WORLD_FAILED || page.path == ["launch"],
					"{}: {}",
					page.path.join(" "),
					code
				);
				assert!(code != check::FOUND || page.path == ["check"], "{}: {}", page.path.join(" "), code);
			}
		}
	}

	#[test]
	fn no_line_of_the_help_is_wider_than_a_terminal() {
		let pages = commands().into_iter().map(|(path, command)| {
			let names: Vec<&str> = path.iter().map(String::as_str).collect();
			render(page(&names).unwrap(), &command)
		});
		for text in std::iter::once(overview()).chain(pages) {
			for line in text.lines() {
				assert!(line.chars().count() <= 100, "{} characters: {}", line.chars().count(), line);
				assert_eq!(line, line.trim_end(), "a line ends in spaces: {:?}", line);
			}
			assert!(
				text.ends_with('\n') && !text.ends_with("\n\n"),
				"{:?}",
				&text[text.len().saturating_sub(40)..]
			);
		}
	}

	#[test]
	fn the_command_an_error_is_about_is_found_among_the_arguments() {
		let root = args::command();
		let path = |args: &[&str]| command_path(&root, &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<String>>());
		assert_eq!(path(&["--pid", "12", "wait", "pause", "--break", "x"]), ["wait", "pause"]);
		assert_eq!(path(&["wait", "--break", "x"]), ["wait"]);
		assert_eq!(path(&["eval", "wait", "pause"]), ["eval"]);
		assert_eq!(path(&["frobnicate", "stop"]), Vec::<&str>::new());
		assert_eq!(path(&["--json"]), Vec::<&str>::new());
	}

	#[test]
	fn options_are_listed_in_a_column() {
		let commands = commands();
		let (_, stop) = commands.iter().find(|(path, _)| path == &["stop".to_owned()]).unwrap();
		let text = render(page(&["stop"]).unwrap(), stop);
		assert!(
			text.contains(
				"\nOptions:
  --timeout SECONDS       Wait up to SECONDS for graceful shutdown. [default: 10]
  --kill                  Terminate the process if shutdown fails or times out.
  --kill-timeout SECONDS  Wait up to SECONDS for exit after termination.
                          [default: 5] Requires --kill.
  --pid PID               Select the world process by PID.
  --json                  Emit one JSON value on stdout.
  -h, --help              Show help for this command.
"
			),
			"{}",
			text
		);
		let (_, eval) = commands.iter().find(|(path, _)| path == &["eval".to_owned()]).unwrap();
		assert!(render(page(&["eval"]).unwrap(), eval).contains("\n  -f, --file FILE             Read the expression or statements from FILE.\n"));
		let (_, launch) = commands.iter().find(|(path, _)| path == &["launch".to_owned()]).unwrap();
		assert!(
			render(page(&["launch"]).unwrap(), launch).contains("\n  -D NAME[=VALUE]   Define a preprocessor macro for hot reload. Repeatable.\n")
		);
		let (_, trace) = commands.iter().find(|(path, _)| path == &["trace".to_owned()]).unwrap();
		assert!(render(page(&["trace"]).unwrap(), trace).contains(
			"\n  --by EXPR                   Count hits by EXPR and print counts on exit.\n                              Cannot be used with \
			 --log.\n  --by caller                 Count hits by calling proc and source line.\n"
		));
	}
}
