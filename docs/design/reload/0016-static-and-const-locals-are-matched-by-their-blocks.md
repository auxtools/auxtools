# Matching `static` and `const` locals by enclosing blocks

A reloaded proc preserves the values of its `static` and `const` local variables. Because
recompiled code renumbers its locals, each new local must be paired with the old local whose slot
it inherits, even when locals of the same name are declared in multiple blocks. Both the launcher
(when matching static slots) and
[the global init proc](0004-a-reboot-after-a-reload-runs-dmasms-global-init-proc.md) (when
renumbering old statics) match locals this way.

Each local records its enclosing blocks from outermost to innermost along with the code that heads
each block, as well as those blocks' numbers in source order. A `for` loop's head counts as one
block and its body as another block inside it. A local is visible wherever its block numbers are a
prefix of the current location's block numbers.

A new local is paired with an old local of the same name by its enclosing blocks. Matching tries
first by the blocks together with their heading code, and then by block kinds alone. Locals of the
same name in blocks of the same kinds that cannot be paired one-to-one are left unpaired and
returned as `Unmatched`, rather than guessed.

## Applies to

- `dmasm/reload/src/reload/locals.rs`: `Declared`, `Step`, `declared`, `visible_at`, `matched`
