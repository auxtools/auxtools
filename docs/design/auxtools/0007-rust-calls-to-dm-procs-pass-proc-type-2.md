# Passing proc type `2` when calling DM procs from Rust

Both `call_proc_by_id` and `call_datum_proc_by_name` accept a proc type parameter, and BYOND's own
direct proc calls pass `2`. Passing `0` causes `..()` to misbehave inside a proc that has no parent.

`Proc::call` passes proc type `2` to `call_proc_by_id`, matching BYOND's direct calls, and
`Value::call` passes `2` to `call_datum_proc_by_name` as well. Changing this value alters how `..()`
behaves in procs called from Rust.

## Applies to

- `auxtools/src/proc.rs`: `Proc::call`
- `auxtools/src/value.rs`: `Value::call`
