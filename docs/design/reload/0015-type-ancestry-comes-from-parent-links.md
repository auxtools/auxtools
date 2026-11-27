# Resolving type ancestry through parent links

A type's path does not determine its ancestors: `/obj` inherits from `/atom/movable`, and
`parent_type` can change a type's ancestry. Hot reload therefore finds ancestors by following
parent links, never by path prefixes.

Built-in types cannot change parents at run time, although source code can still declare a parent
for one (for example, `/client` can declare `/datum` as its `parent_type`). A parent change is
refused when any of the following holds:

- the type is a built-in type
- its instances would become a different built-in type than before
- the new parent is a subtype of the type
- the new parent exists neither in the running world nor among the plan's new types

## Applies to

- `dmasm/reload/src/reload.rs`: `declared_elsewhere`
- `dmasm/reload/src/reload_reparent.rs`: `builtin_base`, reparenting checks
