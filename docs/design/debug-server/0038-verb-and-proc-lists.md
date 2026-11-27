# Ordering new verbs and procs in a type's lists

DreamMaker stores a type's verbs newest first, and BYOND calls the first matching verb. Procs are
listed oldest first. Verbs live on type verb lists rather than in proc name maps, and a verb's
`..()` resolves to the previous definition on the same type, or else to the latest definition on
the nearest ancestor.

A reload matches this layout by inserting a new verb at the start of `VERBS` and appending a new
proc at the end of `PROCS`. It links a new verb's next override using the same `..()` rule.

## Applies to

- `debug_server/src/hot_reload.rs`: `add_to_list`, `new_verb_on_type`
