# Built-in `New()` procs that overwrite variable defaults

Variable-default tests usually create an instance and read the variable, but some built-in BYOND
types overwrite variables inside `New()`, which hides the type's configured default:

- **Exceptions:** In BYOND 515.1637, `new()` replaces an `/exception`'s `name`. `hot_reload_test`
  therefore reads `initial(/exception/hot_reload::name)` rather than inspecting a new instance.
- **Sounds:** `/sound`'s `New()` sets `repeat` and `volume` from its arguments. The `/sound` types
  in the hot reload host therefore set `pan`, `priority`, `frequency`, and similar variables
  instead of `repeat` or `volume`.

## Applies to

- `tests/hot_reload_test/src/main.rs`: "var-table built-ins and bounds" checks
- `tests/hot_reload_test_host/main.dm`: `/sound/hr_sa`, `/sound/hr_sb`, `/sound/hr_st`
