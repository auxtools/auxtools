# Reserved blank node at appearance ID `0xFFFF` past 65,535 appearances

Once the appearance count passes 65,535, BYOND inserts a blank node with the reserved ID `0xFFFF`
into its appearance tree without adding a corresponding entry to the appearance ID table. Code that
walks the appearance tree or indexes the ID table must allow for this gap.

When auxtools checks the appearance tree, it accepts one node whose ID does not lead back to it
through the ID table, provided that node's ID is `UNUSED_ID` (`0xFFFF`) and the table has no entry
for that ID. Any other unmapped node, or a second one, is treated as an error.

To cross this threshold, `make_many_appearances` in the hot reload test host creates 66,000
objects, each with its own name and therefore its own appearance. `hot_reload_test` then reloads
`lamp.dmi` and checks that the reload still finds and updates the lamp's appearances.

## Applies to

- `auxtools/src/appearance_tables.rs`: `UNUSED_ID`
- `tests/hot_reload_test_host/resources.dm`: `make_many_appearances`
- `tests/hot_reload_test/src/main.rs`: "resource reload past 65,535 appearances" check
