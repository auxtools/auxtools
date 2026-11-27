# Initializing a new type from its parent

The running world has no type record for a type added by source edits, and a type's built-in
variables live in its type record. A new type therefore starts as a copy of its parent, with
`NewType::fields` setting the built-in variables in its type record that differ from the parent's:
`name`, `desc`, `icon`, `icon_state`, `text`, `suffix`, `dir`, `layer`, `density`, and `opacity`.
Its other variables and procs arrive separately as new variables, variable defaults, new procs, and
an init proc.

Without its own `name` or an ancestor's, a BYOND type is named after the last segment of its path,
and its `text` is the first letter of its name. Accordingly, when neither the type nor an ancestor
sets `name`, the record receives the last part of the path with underscores replaced by spaces.
When neither sets `text`, and the type has a `name` of its own in `fields` (whether set by the
source or derived from the path), the record receives the first letter of that name.

## Applies to

- `dmasm/reload/src/reload.rs`: `NewType`, `new_type`
