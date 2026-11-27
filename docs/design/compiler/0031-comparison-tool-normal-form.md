# Normalizing bytecode in the DreamMaker comparison tool

The comparison tool compiles each test case with both DreamMaker and dmasm and compares the output
proc by proc. Some differences are deliberate and do not change behavior, so a word-for-word check
would report noise, while ignoring too much would hide real compiler bugs. If a proc's words are
already identical, it counts as an exact match. Otherwise, the tool disassembles both sides and
rewrites deliberate differences into a single normal form before comparing.

## Normalization rules

Before comparison, `normalize` applies four rewrites:

- `DbgFile` and `DbgLine` instructions are dropped.
- DreamMaker omits a cache load when the cache already holds the base. The tool expands omitted
  cache loads on both sides using
  [DreamMaker's cache tracking rules](0001-track-the-cache-as-dreammaker-does.md) for what the cache
  holds at a label. For each instruction, it also records how many cache-load steps it expanded, so
  differences in cache reuse are still reported.
- `static_proc(P)` and `static_verb(P)` are rewritten as calls by name. DreamMaker emits the static
  form only when the callee is already compiled, which depends on parse order, and both forms look
  up the proc by name.
- Each global variable is named with its slot, because multiple globals can share a name (see
  [proc statics and global slots](0005-proc-statics-and-proc-path.md)).

Because the disassembler needs a name for every value and stops if an environment answers "unknown",
`value_to_string_data` never returns "unknown". Any value it cannot name is rendered as
`<what#number>`, which compares unequal to every real name.

Any rewrite added to `normalize` suppresses an entire class of differences across all cases, so it
must cover only behavior-preserving differences. Cases that intentionally differ for other reasons
are listed as known differences, and the tool reports any listed case whose code matches after all.

## Building cases and the fuzz harness

The comparison tool builds each case through a `world.dme` that includes `case.dm` (see
[DreamMaker's debug file naming](0009-debug-file-names.md)). For dmasm, it parses the case through a
separate `dmasm.dme` that undefines `SPACEMAN_DMM`, sets `DM_VERSION` and `DM_BUILD` to match the
DreamMaker build, and then includes `case.dm`. Including the file instead of prepending the defines
to its text keeps the case's line numbers identical to DreamMaker's build.

In the fuzz harness, an `alist`'s iteration order follows the string IDs of its keys. Because a
reloaded world and a fresh build assign different string IDs, `fz_show` sorts an `alist`'s parts
before printing them for comparison.

## Applies to

- `dmasm/compiler/examples/compare_dreammaker/compare.rs`: `judge`, `normalize`, `canonical`
- `dmasm/compiler/examples/compare_dreammaker/env.rs`: `value_to_string_data`
- `dmasm/compiler/examples/compare_dreammaker/main.rs`: `run_case`
- `dmasm/compiler/tests/dreammaker/fuzz-harness.dm`: `fz_show`
