# DreamMaker names included files relative to the including `.dme`

When a `.dme` other than the world's own includes a file, DreamMaker names that file relative to the
including `.dme`. The same short name can therefore refer to different files in debug information:

- `lib/lib.dme` is a separate environment included by the world's `.dme`. Because `lib/lib.dme`
  includes `tool.dm`, the world's debug information names `lib/tool.dm` `"tool.dm"` rather than
  `"lib/tool.dm"`.
- The world's own `.dme` includes `sub/tool.dm`, so the world's debug information names it
  `"sub/tool.dm"`. That name ends with `"tool.dm"`, which is the world's name for `lib/tool.dm`.

A debugger that matched source names by their ending would place a breakpoint for `sub/tool.dm` into
`lib/tool.dm` as well, so the debugger matches source names exactly.

The DAP test host reproduces this case by placing code on the same line numbers in `lib/tool.dm`
(`// @lib_tool`) and `sub/tool.dm` (`// @sub_tool`), so a breakpoint matched only by line and the
end of the name would stop in the wrong file. `files_another_environment_names` asserts that the two
markers are on the same line, runs `sc_tools` (which calls `lib_tool` first), and expects to stop
only in `sub_tool`. Both files must keep their markers on the same line so that assertion continues
to hold.

## Applies to

- `tests/dap_test/src/groups/basics.rs`: `files_another_environment_names`
- `tests/dap_test_host/lib/lib.dme`
- `tests/dap_test_host/lib/tool.dm`
- `tests/dap_test_host/sub/tool.dm`
