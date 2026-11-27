# Weak value IDs are namespaced text

`WeakValue` marks a datum with an ID in the DM variable `__auxtools_weakref_id` and checks that ID
on upgrade to verify that the datum is still the same instance. Because every native library linked
with auxtools has its own issuer and all of them write to `__auxtools_weakref_id`, a library adopts
any ID that another library has already written.

Each issuer generates a random namespace by hashing the process ID with `RandomState`, ensuring one
library never issues an ID already held by another library's handle. The ID is stored as text in the
form `namespace-serial`, with the namespace formatted in hex and the serial in decimal starting at
1. Storing the ID as text avoids numeric collisions past 2^24, which would occur with DM numbers
because they are `f32` values. Changing this format or dropping the namespace would allow separate
libraries to confuse each other's datums or cause IDs to collide above 2^24.

## Applies to

- `auxtools/src/weak_value.rs`: `Issuer`, `id_text`, `parse_id`, `id_of`, `WeakValue`
