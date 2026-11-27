# Keying and invalidating the code index

The code index caches each proc's disassembled bytecode, including instruction offsets, lines, and
files. Because auxtools never mutates bytecode in place (`Proc::set_bytecode` installs new code at a
new address) and keeps replaced bytecode at its old address until the world ends so pre-reload
frames can keep running it, the index keys entries by proc ID, bytecode address, and length, and
clears the cache whenever a world ends or loads.

Several rules govern caching and disassembly:

- Because new bytecode always gets a new address, keying by proc ID, code address, and length causes
  a cache miss and forces a fresh read. A stub such as the evaluation stub gets a new entry each
  time it is given different bytecode.
- `proc_changed` drops a proc's entry when a reload modifies it. The next lookup reads the new code,
  and `restore_breakpoints` reuses that same entry.
- To show a frame still running old bytecode, the debugger compares the frame's code pointer with
  the proc's current one and reads the old code at the frame's address when they differ. Old code
  that differs from the current code only in its line numbers counts as the current code. Because
  old code is read in place with any breakpoints still installed, bytecode containing breakpoints is
  never cached.
- The next world loaded in the same process assigns its procs the same IDs and can allocate bytecode
  at addresses previously used by reloaded code. Clearing the cache in `world_ended` and
  `world_loaded` prevents stale entries from matching the new world's procs.
- BYOND counts a proc's words in 16 bits, so offsets and lengths are `u16`. `Code::read` stops
  disassembling at the first instruction whose offset or index does not fit in a `u16` rather than
  wrapping.

## Applies to

- `debug_server/src/code_index.rs`: `of`, `running`, `proc_changed`, `world_ended`, `world_loaded`,
  `Code::read`
- `debug_server/src/hot_reload.rs`: `restore_breakpoints`
- `auxtools/src/bytecode_manager.rs`: `set_bytecode`
