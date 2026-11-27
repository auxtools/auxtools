# DreamMaker line records for no-code lines and read-ahead lines

DreamMaker emits a line instruction for each body line of a proc, including several kinds of lines
that contain no executable code:

- A variable declared without a value, such as `var/unset`.
- A `set` statement, such as `set background = TRUE`.
- An empty macro, only when it is the first body line (as with `RETURN_TYPE(/list)`).

Because the next line's code directly follows such a line record, a breakpoint on a no-code line
stays on that line, stops where the next line's code starts, and reports the stop on the no-code
line itself.

DreamMaker also emits a read-ahead line record for `var/text = "[n]"`, recording the next line's
number before evaluating the initializer. A breakpoint on the following line
(`var/size = length(text)`) therefore stops while `text` is still unset.

## Debugger and hot reload coverage

The debugger keeps breakpoints on these lines, and the reload compiler reproduces DreamMaker's line
records, including the read-ahead behavior. Giving these lines no record during a reload, or
recording the read-ahead line after the initializer, would move breakpoints off lines present in the
built code or stop execution at a different place than before.

Two test procs cover these behaviors:

- `sub_declares` (`sub/tool.dm`): verifies that a breakpoint on `var/unset` is verified on its own
  line, stops there, and continues to the next two lines with Step Over.
- `nothing_to_run` (`stepping.dm`): contains an empty macro, a bare variable declaration, a `set`
  statement, and a read-ahead line. Breakpoints on all four stay on their lines before and after a
  reload, and a breakpoint on the read-ahead line stops at the same line with `text` in the same
  state after the reload as before it.

## Applies to

- `tests/dap_test/src/groups/breakpoints.rs`: no-code breakpoint check
- `tests/dap_test/src/groups/reloading.rs`: `nothing_to_run`, `read_ahead_stop`
- `tests/dap_test_host/stepping.dm`: `nothing_to_run`
- `tests/dap_test_host/sub/tool.dm`: `sub_declares`
