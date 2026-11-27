# BYOND reuses a deleted datum's ID for the next datum

BYOND assigns a newly created datum the ID of a datum deleted immediately before it, so a weak
reference that relied only on the datum's ID would resolve to the new datum.

The `auxtest` weak-reference test relies on this behavior to verify `WeakValue`. `test_weak_values`
takes a weak reference to a datum, deletes that datum with `del_value`, and then calls
`create_datum_for_weak` to allocate a new datum that receives the old datum's ID. The test checks
that the weak reference does not resolve to the new datum.

If BYOND stopped reusing the ID, the test would still pass, but it would no longer check anything.

## Applies to

- `tests/auxtest_host/auxtest_host.dm`: `create_datum_for_weak`, `weak_test_datum`
- `tests/auxtest/src/weak.rs`: `test_weak_values`
- `auxtools/src/weak_value.rs`: `WeakValue`
