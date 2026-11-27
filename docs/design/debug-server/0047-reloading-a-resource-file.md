# Reloading a resource file

BYOND's resource cache assigns files with identical content the same entry, and values that use a
resource hold its ID. A resource reload adds the new file to the cache, repoints appearances from
the file's old IDs to the new ID, and resends them to clients.

- Values already stored elsewhere keep the old resource, including variables, lists, `/icon` and
  `/sound` datums, and mutable appearances. Updating those would require walking every object in
  the world, which auxtools does not do.
- IDs also used by another file are left unchanged, because repointing them would alter the other
  file as well. The reload warns that the two files have the same bytes and share an ID, and that
  appearances using it do not change.
- The debug server records each file's original IDs and the IDs assigned by later reloads so the
  next reload repoints the right ones.

## Applies to

- `debug_server/src/resource_reload.rs`: `prepare`, `apply`
- `debug_server/src/server/reload.rs`: `record_named_files`, `record_resource_ids`,
  `apply_hot_reload`
