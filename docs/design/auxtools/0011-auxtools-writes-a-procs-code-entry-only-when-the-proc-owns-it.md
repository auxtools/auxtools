# Owning a proc's bytecode entry before modifying it

A proc's code lives in a code entry in BYOND's misc table, and multiple procs with identical
bytecode (as well as other misc table users, such as a proc's locals table) can share one entry. An
entry can therefore be modified in place only when nothing else references it. In addition, a BYOND
frame holds a pointer to the code buffer it started with and continues executing that buffer after
the proc receives new code, while breakpoints patch a code buffer in place while it is running.

Rather than scanning for every user of an entry, the bytecode manager considers a proc to own its
entry only when this library created the entry for that proc and the entry's reference count is
still exactly one. Interning takes a reference, and anything else that names an entry must take a
reference as well. Code that names a code entry without taking a reference breaks this ownership
rule and lets the manager modify an entry it believes only it uses.

## Entry ownership and allocation

- **New entries (`new_entry`):** Appends a serial number as a trailing word so interning other words
  never returns the same entry. Serials count down from `u32::MAX`. `new_entry` accepts the entry
  only if it has one reference and holds those exact words, trying up to eight serials before
  failing. Once the entry points to its code, its word count excludes the serial. Each new entry
  costs one misc entry.
- **Writing:** Before modifying a proc's entry, the manager gives the proc a new entry unless the
  proc already owns its entry. The old entry keeps its words so no other user sees a change. Because
  each library creates its own entries, two libraries never modify the same entry.
- **`own_code` vs. `copy_code` and `set_bytecode`:** Unless the proc already owns its entry,
  `own_code` gives the proc a new entry that takes over the existing code buffer and gives the old
  entry a copy, so the proc's running frames execute any patches applied to it. Its caller vouches
  that no other library has replaced the proc's code, because that library would free the buffer
  when it shuts down. `copy_code` and `set_bytecode` instead give the new entry a copy at a new
  address.
- **Allocation keys:** Allocations are keyed by the proc and the code the buffer was created with,
  not by the buffer's current words, because breakpoints patch the buffer in place.

## Keeping old code readable and shutting down

- **Old code:** `retired_by_address` keeps the words of replaced code keyed by buffer address so the
  debugger can still read code that an old frame is executing via `bytecode_at`. Reading old frames
  depends on both these retired words and the original-code map staying alive until shutdown.
- **First code of an entry:** For an entry created by auxtools, the first code it pointed to is
  either auxtools' own copy or a buffer that `own_code`'s caller vouched for. That code remains
  valid while the world is live, so `bytecode_at` can read it without holding a reference.
- **Shutdown:** Each entry changed by the manager is pointed back at the code it originally had,
  unless something else has taken a reference to it. Buffers that a frame is still executing, or
  that an entry still referenced elsewhere points to, are leaked rather than freed.

## Applies to

- `auxtools/src/bytecode_manager.rs`: `State`, `new_entry`, `owned_entry`, `give_own_entry`,
  `own_code`, `copy_code`, `set_bytecode`, `remember_bytecode`, `bytecode_at`, `shutdown`
