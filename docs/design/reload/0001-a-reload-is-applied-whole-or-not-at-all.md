# All-or-nothing hot reloads and deferred changes

`dmasm-reload` compares two source snapshots and determines the changes between them. The running
world applies a `Plan` only when `Changes` contains no refusal. Applying only the supported part of
a change set would leave the world in a state that no build of either snapshot has, so any refusal
rejects the entire change set and callers never apply a partial change set.

## Refusals and new parse errors

New parse errors can omit declarations from the parsed tree, making a declaration missing due to a
syntax error look identical to one removed by an edit. `diff` therefore refuses new parse errors
before comparing any definitions, preventing a syntax error from deleting definitions from the
running world. A parse error counts as new when the old snapshot does not contain the same error
the same number of times.

## Deferred startup-only changes

Some values are read only when a world starts: world settings and some client settings are read
from the world record at startup, and initializers of existing global and static variables already
ran at startup so editing them has no effect on the running world. Changes that require a reboot or
a rebuild are placed in `Changes::deferred`, and callers hold the entire change set rather than
applying part of it:

- A changed initializer of an existing global or static variable is deferred. It requires a reboot
  if [the global init proc](0004-a-reboot-after-a-reload-runs-dmasms-global-init-proc.md) compiled,
  and a rebuild if it did not. A new variable, or a variable that becomes static, receives its
  value immediately.
- A changed world setting, or a client setting stored in the world record, always requires a
  rebuild and is deferred. `Changes::settings_as_built` drops such a deferral when the value
  matches the one the running build was made with.

## Applies to

- `dmasm/reload/src/reload.rs`: `Plan`, `Changes`, `Changes::deferred`,
  `Changes::settings_as_built`, `diff`, setting check (`in_world_record`) in `var_default`
- `dmasm/reload/src/reload/global_init.rs`: `changed`, `global_init_proc`
