# Compiling built-in proc calls to match DreamMaker's arguments

Each built-in proc compiles to its own instruction with arguments pushed in a fixed layout. What
DreamMaker pushes often differs from the documented signature: it pushes placeholders for omitted
parameters, drops certain arguments, and accepts argument counts the proc never uses.

## The built-in table

- When a call omits a parameter, DreamMaker usually pushes `null` and lets the instruction apply
  the real default. `BUILTINS` records the value DreamMaker pushes, which is why many defaults in
  the table are `null` rather than the documented default.
- `BUILTINS` is sorted by the bytes of each name, placing uppercase names before lowercase ones.
  Because `find` performs a binary search and nothing checks the sort order, an out-of-order entry
  is silently missed, so new entries must be inserted in byte order by hand.
- DreamMaker accepts any number of arguments to `regex()` and passes the count to the instruction
  even though `regex()` uses at most two. The table matches this by compiling `regex()` as a
  counted call with no upper limit.

## Special calls

- Outside `<<`, `browse()`, `browse_rsc()`, `output()`, and `load_resource()` return their first
  argument, while DreamMaker evaluates and drops the remaining arguments.
- DreamMaker passes `gradient()`'s arguments as a list. When any argument is named (such as `index`
  or `space`), it builds an associative list and keys the unnamed arguments by their position.
- In `animate()`, an unnamed argument after the object stops the animation and is not evaluated by
  DreamMaker, whereas a named argument creates an animation step. Thus `animate(obj, x)` with only
  unnamed arguments compiles to `NullAnimate` on the object alone. Evaluating those dropped
  arguments would run code that DreamMaker does not execute.

## Applies to

- `dmasm/compiler/src/compiler/builtins.rs`: `Param`, `find`, `BUILTINS`
- `dmasm/compiler/src/compiler/builtin_procs.rs`: `browse`, `gradient`, and `animate` arms of
  `emit_code`
