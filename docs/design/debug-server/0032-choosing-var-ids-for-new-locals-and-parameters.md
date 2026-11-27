# Choosing variable IDs for new locals and parameters

A proc's locals table and parameters table list variable name IDs. Choosing those IDs during a hot
reload must account for three BYOND behaviors: BYOND initializes each local from the global slot of
its variable ID (so a local whose ID has a non-`null` global value would start with that value),
BYOND does not reliably create an empty misc entry, and BYOND uses a parameter's name ID only to
match named arguments in a call.

## Locals

For each local name, `locals_table` picks a variable ID with that name whose global slot is `null`.
If no ID with that name has a `null` slot, it takes the first variable ID of any name whose slot is
`null`, and fails if there is none. Every local therefore starts as `null`.

An empty list of locals reuses an existing empty table instead of creating a new one:

- A new proc with no locals receives the template proc's table.
- The evaluation and spawn stubs return to the empty table they were built with.
- A changed proc that loses all of its locals shares an empty table found on another proc in the
  world. If no proc has one, the reload fails and asks for a rebuild.
- A changed proc whose number of locals stays the same keeps its table, relying on per-address name
  storage instead (see
  [local names are kept per code address](0031-local-names-are-kept-per-code-address.md)).

## Parameters

A new parameter name can take any variable ID with that name. If no variable has that name, the
reload adds a new variable name with a `null` value for it.

## Applies to

- `debug_server/src/hot_reload.rs`: `locals_table`, `empty_locals`, `prepared_code`,
  `AddedVars::param_name`
- `debug_server/src/server/eval.rs`: `call_with_locals`
- `debug_server/src/server/spawn.rs`: `set_stub_locals`
