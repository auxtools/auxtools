# Running proc hooks and per-call work without holding locks

Proc hooks and callbacks queued with `at_proc_calls` can call DM procs, which re-enter the proc-call
path (`call_proc_by_id_hook` and `on_proc_call`), and can also register new hooks or queue
additional work. The proc-call path handles its shared state as follows:

- `call_proc_by_id_hook` and `proc_starting` copy the hook out of `PROC_HOOKS` and drop the
  `RefCell` borrow before running the hook. Holding the borrow across a hook would panic if the hook
  registers another hook.
- `on_proc_call` drains the queued items out of `PROC_CALL_WORK` and releases the mutex before
  running them. Holding the mutex would deadlock if a work item calls a DM proc. Items that return
  `false` are not finished and are placed back in the queue afterward, while items that panic count
  as finished and are dropped. Any new work queued while the drained batch runs stays in the queue,
  and `PROC_CALL_WORK_PENDING` is updated from whether the queue is empty.
- `PROC_CALLS` counts proc calls, is read on every proc call, and can be read from any thread via
  `proc_calls`. Because only BYOND's thread increments `PROC_CALLS`, `on_proc_call` uses a relaxed
  load followed by a relaxed store rather than an atomic read-modify-write on every proc call, while
  other threads only read it.

## Applies to

- `auxtools/src/hooks.rs`: `call_proc_by_id_hook`, `proc_starting`
- `auxtools/src/lib.rs`: `on_proc_call`, `at_proc_calls`, `PROC_CALL_WORK`,
  `PROC_CALL_WORK_PENDING`, `PROC_CALLS`, `proc_calls`
