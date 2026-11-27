# Matching lines between old and new code

To move line breakpoints and keep line numbers accurate when only lines moved, a hot reload pairs
the lines of a proc's old code with the lines of its new code. Because an edit elsewhere in the
proc renumbers jump targets and local slots, raw bytecode words cannot be compared directly.

## Reducing each line's code

`line_code` reduces each line to comparable words:

- Locals are compared by name, and jump operands are blanked.
- Lines from another file are kept as `None`.
- The proc's final `End` is left out because it belongs to whichever line is last.
- An unconditional jump (`Jmp` or `JmpLoop`) to another line, such as an `if`'s jump over its
  `else` or a loop's jump back, ends a block and belongs to whichever statement ends the block. The
  rest of a line from such a jump is left out unless the jump starts the line. A line that consists
  only of a jump, such as `break`, keeps it.
- The `End` that ends a `spawn` block's code is left out in the same way.

## Pairing lines

`kept_lines` pairs the common prefix and suffix, then finds the longest ordered match of the lines
between them when that table has at most 4,000,000 cells. Because the ordered match keeps only one
of two lines that swapped places, a second pass pairs any unpaired line whose code is unique in
both versions, wherever it moved.

## Placing a breakpoint inside a matched line

`same_place` moves a breakpoint to the same instruction inside the matched line when the code from
the line's start up to that instruction is the same. A jump in the old code must go to where the
new code places its old target. When code is added after a jump to the proc's final `End`, it can
end up inside the jump's block or after it, so a jump to the old final `End` may match either.
Otherwise, the breakpoint goes to the line's start.

## Applies to

- `debug_server/src/hot_reload.rs`: `line_code`, `kept_lines`, `unique_lines`, `named_words`,
  `same_place`, `jump_target`
