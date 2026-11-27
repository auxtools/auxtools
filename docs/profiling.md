# Instruction profiling

auxtools includes an instruction profiler that measures DM execution time by proc, source line, and
bytecode instruction. Run it from the command line with `auxtools profile` or from VS Code (see
[Instruction profiling](../editors/vscode/README.md#instruction-profiling) in the extension README,
and [README.md](../README.md) for installation).

## Profiling from the command line

```sh
auxtools profile --for 10                       # top slowest procs and source lines
auxtools profile /mob/living/proc/Life --for 10  # instruction breakdown for one proc
auxtools profile --for 10 --html profile.html   # standalone interactive HTML report
```

Without `--for`, profiling runs until you press `Ctrl+C`. Omit proc paths to rank the top `--top N`
(default `40`) slowest procs and lines, or pass one or more `/PATH[#N]` proc paths to print
instruction-level listings grouped by source line (hit count, share of self time, nanoseconds per
hit, and callee time per hit). Pass `--json` for structured output or `--html FILE` for a
self-contained interactive report.

## How timings are measured

An instruction's self time runs until the next instruction begins, which includes BYOND's internal
work for that instruction (such as copying a list or freeing locals). For a call instruction, self
time covers only entering the proc, and execution inside the callee is reported separately. Calls
triggered by `auxtools eval` and the Debug Console are included, whereas sleeping and engine work
between DM runs are excluded. If a called proc sleeps, its caller's callee time omits that call.

## Overhead and limits

The profiler estimates its per-instruction hook overhead from the lowest average duration among
instructions with enough timed samples and subtracts that baseline from each instruction's self and
callee times. Timings are therefore relative to that baseline, and instructions with very few hits
can still reflect single slow runs (such as the first run after a sleep).

Profiling slows execution while active, and only one profile can run in a world at a time. A profile
ends if its owning client disconnects or the world reboots. After a hot reload, old and new bytecode
are recorded separately.
