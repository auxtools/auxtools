# Resource names compare without case or slash direction

DreamMaker resolves resource names such as `'icons/obj.dmi'` regardless of letter case or slash
direction.

The resource index replaces backslashes with forward slashes and lowercases each name before
comparing it. Two spellings that DreamMaker resolves to the same file are therefore treated as a
single resource.

## Applies to

- `dmasm/reload/src/reload/resources.rs`: `normalized`, `Index`
