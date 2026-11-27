# Deferring breakpoints sent on save until hot reload

When a file is saved, VS Code resends the file's breakpoints with `sourceModified`, using the saved
file's line numbers. Those requests usually arrive before the world has the reloaded code.

Rather than mapping those breakpoints onto the old lines, the debug adapter answers them as
unverified with the message "pending hot reload", and places them once the reload is applied.
`dap_test` exercises this behavior by saving an edit that moves `middle_2` down two lines, sending
the breakpoint with `sourceModified` before the reload, checking the pending response, and then
verifying that the breakpoint moves to the new line and pauses there.

## Applies to

- `auxtools-cli/src/adapter.rs`: `sourceModified` handling
- `tests/dap_test/src/groups/reloading.rs`: "defer sourceModified breakpoints until reload" check
