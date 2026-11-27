# Reserving new IDs before assembling reload bytecode

Code assembled during a hot reload references procs, variables, `in` lists, and prefabs by ID. New
code can refer to definitions added by the same reload, which do not yet exist in BYOND when the
code is assembled.

During preparation, the reload reserves the IDs its new procs will receive by counting up from the
current proc count before resolving references in variable defaults and code. New variable, `in`
list, and prefab IDs are also reserved in the order BYOND will allocate them so that code assembled
during preparation can refer to staged additions by the IDs they will receive.

`append` then adds entries in this order: variables, procs, prefabs, `in` lists, type init procs,
and then the new globals in `global.vars`. If an addition fails or receives a different ID than
expected, `append` stops the reload before any code is installed.

## Deduplicated proc entries

BYOND's `procdef_add` returns the ID of an existing proc when a new proc's entry matches it, which
happens for `in` list procs, prefab init procs, and type init procs. The reload accounts for this
per kind of entry:

- **New procs:** Added with `append_proc`, which hides existing entries with the same path while
  adding so a new proc always receives a fresh ID.
- **`in` lists:** Only the list's index is checked. Its proc may receive the ID of an existing proc
  with the same entry, and that ID works as well.
- **Prefabs:** Only the instance ID is checked, not its init proc's ID.
- **Type init procs:** The ID is not checked. BYOND gives a type the same init proc as another type
  whose init code is identical, and nothing else refers to an init proc by ID.

## Applies to

- `debug_server/src/hot_reload.rs`: ID reservation in `prepare`, `AddedVars`, `append`,
  `append_entries`
- `auxtools/src/runtime_tables.rs`: `add_proc`, `append_proc`
