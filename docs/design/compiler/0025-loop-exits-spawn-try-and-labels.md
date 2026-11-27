# Loop exits across `spawn`, `try`, and labels

`break` and `continue` leave or restart a loop and can cross a `try` block, a `spawn` block, or a
label along the way. DreamMaker compiles each case with its own rules, and BYOND has known bugs in
some of them that dmasm warns about.

## `spawn` blocks

A `spawn` block is compiled inline: `Spawn` jumps over the block, and the block ends with `End`. The
new thread runs the block. For compilation, the spawned code remains inside its enclosing loops, so
a `break` or `continue` inside it compiles as a jump into the enclosing loop, and a `continue` in
the new thread keeps running that loop.

At run time, however, the spawned code executes later, after the surrounding loops in the original
thread have moved on. The walk that warns about known BYOND bugs (`Walk::block`) therefore clears
its loop stack while visiting a `spawn` block, so it does not warn about a `continue` to a labeled
loop or nested list loops across `try` when the loop is outside the `spawn` and the statement is
inside it. These two views of `spawn` do not conflict: one describes compilation and the other
describes execution timing. Whether BYOND's loop bugs also happen in a spawned thread is unknown,
so because the bug walk stays silent there, a real case inside a `spawn` goes unwarned.

## `try` blocks and labels

- Inside a `try` block, DreamMaker exits a loop with the instructions that end the `try`: `Catch`
  for `break` and `TryJmp` for `continue`, instead of `Jmp` and `JmpLoop`. This holds regardless of
  where the loop sits relative to the `try`.
- A label on a loop gives the loop that name, but `break name` leaves the labeled block itself and
  drops the loop's iterator on the way out.

## Empty `for` initializers

DreamMaker drops an empty initializer before a semicolon in a `for` loop, so `for (; a < 5; a++)`
uses `a < 5` as the initializer and `a++` as the test. The compiler reproduces this behavior.

## Applies to

- `dmasm/compiler/src/compiler/statements.rs`: `Break`, `Continue`, `Spawn`, and `Label` arms of
  `emit_statement_inner`
- `dmasm/compiler/src/compiler/known_bugs.rs`: `Walk::block`
- `dmasm/compiler/tests/compile_proc.rs`: `a_for_with_an_empty_first_part_is_read_past_it`
