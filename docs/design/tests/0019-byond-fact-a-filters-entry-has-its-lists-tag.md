# `M.filters[1]` shares the value tag of its filters list

In BYOND, `M.filters[1]`, an individual entry of an atom's filters, carries the same value tag as
the filters list it came from rather than a tag of its own. Code that decides whether a value is a
list from its value tag alone would therefore treat a filter entry as a list.

`List::is_list` avoids this by asking BYOND through `value_is_list` instead of reading the tag. The
`auxtest` list samples include a filters entry, and `test_lists` checks that `List::is_list` agrees
with DM's `islist()` and that the length agrees with `length()` for every sample. Changing
`List::is_list` to rely on the tag causes this test to fail.

## Applies to

- `tests/auxtest_host/auxtest_host.dm`: `auxtest_islist_samples`
- `tests/auxtest/src/lists.rs`: `test_lists`
- `auxtools/src/list.rs`: `List::is_list`
