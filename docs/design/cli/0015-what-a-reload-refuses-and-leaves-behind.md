# Refused reloads, partial failures, and removed definitions

A hot reload applies a multi-step plan to a running world, where some edits cannot be applied safely
and some steps can fail after earlier steps have already changed the world. Because the source
worker compares parses of the source rather than inspecting the running world, auxtools separates
refused reloads from partial failures and defers certain checks to the world itself.

## Refused reloads and partial failures

- **Refused vs. applied:** A refused reload leaves the world unchanged, except for unused table
  entries. An applied reload can still contain failed steps, listed in `HotReloadResult::failures`.
  Those are steps that failed after other steps modified the world, leaving the world running a
  partially applied reload.
- **Definitions hidden by parse errors:** A parse error hides the definitions that follow it. When
  the error is fixed, those types, procs, and variables appear in the new parse as if they were new,
  even though DreamMaker may already have compiled them into the built world. Additions of types,
  procs, or variables that appear when a parse error is fixed are refused, because adding them again
  could create overrides of definitions the world already has.

## Removed variables and global procs

- **Removed variables:** Whether a variable removed from source still exists in the running world
  can be known only by the world itself. The source worker does not warn about removed variables and
  leaves the warning to the world's response.
- **Removed global procs:** BYOND finds global procs by name, whereas code that already holds a
  direct reference (such as `call(/proc/name)()`) keeps running the definition it refers to. A global
  proc removed by a reload can no longer be found by name, but existing direct references continue
  running its old code until the world restarts. The debug adapter reports such a proc as removed
  from source but still reachable by direct reference, rather than deleted. A breakpoint placed in
  such a proc is listed with `PlacedBreakpoint::unlinked` set, and its `at` field can name the proc
  that replaced it.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::HotReload`, `HotReloadResult`,
  `PlacedBreakpoint::unlinked`
- `auxtools-cli/src/reload.rs`: `refuse_hidden`
- `auxtools-cli/src/code.rs`: removed-variable filter in the reload path
- `auxtools-cli/src/adapter.rs`: `Adapter::reload`
