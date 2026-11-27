# Locating test lines in `dap_test` with `// @name` markers

`dap_test` sets breakpoints and expects stops on many lines of its host world. Hard-coding line
numbers in the tests would break whenever the host world is edited, so each target line ends with a
`// @name` comment instead.

`Marks::place` searches the `.dm` files under the host directory in sorted order, with
subdirectories after files, and returns the first line ending with the requested marker. Because the
first match wins, each marker name must appear only once across all host files:

- These comments are test anchors rather than documentation. Removing comments from the host world
  must preserve every `// @name` marker or replace the lookup.
- Reusing a marker name makes the tests target whichever line `Marks::place` finds first.

## Applies to

- `tests/dap_test/src/world.rs`: `Marks`
- `tests/dap_test_host/`: `// @name` comments
