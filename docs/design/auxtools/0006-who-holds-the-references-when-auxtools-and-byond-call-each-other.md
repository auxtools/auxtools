# Reference ownership across auxtools and BYOND calls

BYOND reference-counts its values. Each `Value` in auxtools holds one reference and releases it on
drop. `Value::from_raw` acquires a new reference, whereas `Value::from_raw_owned` takes ownership of
an existing reference.

## When BYOND calls auxtools

- **Detour arguments, `src`, and `usr`:** In the `call_proc_by_id` detour, the callee owns the
  references to the arguments. `call_proc_by_id_hook` takes them over with `Value::from_raw_owned`
  so they are released once the hook finishes, and acquires new references to `src` and `usr` with
  `Value::from_raw`.
- **Detour return value:** The value returned by the detour carries a reference that BYOND takes
  over. The detour forgets its `Value` instead of dropping it, because dropping it would release a
  reference BYOND relies on.
- **Frame values in `proc_starting`:** A running frame holds its own references to its `src`, `usr`,
  and arguments. When `proc_starting` constructs `Value` instances from them, it acquires new
  references with `Value::from_raw` and leaves the frame's references untouched.
- **Frame return slot (`.`):** A frame's `.` is `null` when a proc starts, so `proc_starting` stores
  the hook's result there without releasing an old value, and `.` takes over the result's reference.
  Storing into `.` anywhere other than at a proc's start would leak the previous value.

## When auxtools calls BYOND

- **Arguments:** `call_proc_by_id` and `call_datum_proc_by_name` release one reference to each
  argument. `Proc::call` and `Value::call` therefore acquire an extra reference to each argument
  before the call so the caller's `Value` instances stay valid. Dropping that extra reference before
  a call would free arguments the caller still holds.
- **Return value:** The result returns with a reference, which `Value::from_raw_owned` takes over.

## Applies to

- `auxtools/src/hooks.rs`: `call_proc_by_id_hook`, `proc_starting`
- `auxtools/src/proc.rs`: `Proc::call`
- `auxtools/src/value.rs`: `Value::call`, `Value::from_raw`, `Value::from_raw_owned`
