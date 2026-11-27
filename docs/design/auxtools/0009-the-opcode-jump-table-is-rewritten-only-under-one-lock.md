# Synchronizing rewrites to the opcode jump table

Any thread can call `hear_all` to route pending work to the instruction handler, whereas BYOND's
thread calls `hear_only` to route only selected opcodes to the handler. Both rewrite the opcode jump
table that BYOND's thread reads on every instruction. Meanwhile, the naked assembly stubs read
several statics directly by symbol (`ORIGINAL_TABLE`, `DISPATCH_START`, `PASS`, and `UNKNOWN_NEXT`),
and the table stubs also clear `PASS`. Because those are plain memory accesses outside Rust's view,
those statics cannot be wrapped in a lock.

auxtools keeps table updates safe with three rules:

- The jump-table entries are atomics and are rewritten only while holding the `LISTENING` mutex,
  which makes `Table` safe to share between threads.
- `ORIGINAL_TABLE`, `DISPATCH_START`, `UNKNOWN_ORIGINAL`, `UNKNOWN_NEXT`, `PASS`, and `PASS_UNKNOWN`
  are `static mut`, and only BYOND's thread writes to them once the table is installed.
- `hear_all` stores `ALL = true` only after every entry points to the handler stub. Callers return
  early when `ALL` is set and rely on the entire table already sending every opcode to the handler.
  Setting `ALL` before the loop would let a caller return while some opcodes still bypass the
  handler.

After calling `hear_only`, the caller must check pending work again because another thread may have
called `hear_all` in the meantime.

## Applies to

- `auxtools/src/instruction_hook.rs`: `Table`, `LISTENING`, `ALL`, `hear_all`, `hear_only`,
  `ORIGINAL_TABLE`, `DISPATCH_START`, `UNKNOWN_ORIGINAL`, `UNKNOWN_NEXT`, `PASS`, `PASS_UNKNOWN`
