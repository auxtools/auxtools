# dmasm

dmasm assembles, disassembles, and compiles BYOND's DM bytecode. auxtools uses it to evaluate
expressions in the debugger, show a proc's instructions, and hot-reload source changes into a
running world.

## Crates

dmasm is split into three crates:

- `dmasm-bytecode` (`dmasm/bytecode/`) converts a proc's bytecode to named instructions and assembles
  instructions back into bytecode. It requires no parser. The
  [DM bytecode specification](dmasm-specification.md) lists every instruction.
- `dmasm-compiler` (`dmasm/compiler/`) compiles procs, proc bodies, and expressions parsed by
  SpacemanDMM's `dreammaker` crate. It emits the same instructions as DreamMaker for a target BYOND
  build and warns about known BYOND bugs. The [proc compiler reference](dmasm-compiler.md) covers
  supported constructs, validation against DreamMaker, and known differences.
- `dmasm-reload` (`dmasm/reload/`) diffs two parses of a codebase and uses `dmasm-compiler` to compute the
  proc, variable, type, and resource updates for a running world, flagging any edits that require a
  reboot or a rebuild.

The `dmasm` crate re-exports all three crates at the paths they had when dmasm was a single crate,
exposing the bytecode types, `assembler`, and `disassembler` at its root alongside the `compiler`,
`reload`, and `diagnostic` modules.

Every error, warning, reload refusal, and reload note carries an ID from `diagnostic::Id`. Match on
the ID in code rather than the message text, which is written for human readers and may change.

Each crate provides a `serde` feature that derives serde's traits for its types. Enabling `serde` on
`dmasm` enables it across all three crates.

## Supported builds and testing

dmasm supports BYOND builds 515.1637–516.1689. Select the target build with `ByondVersion`.

Run the tests without a BYOND installation:

```sh
cargo test -p dmasm-bytecode -p dmasm-compiler -p dmasm-reload --all-features
```

## Comparing with DreamMaker

`dmasm/compiler/tests/dreammaker/` holds small worlds that dmasm must compile the same way DreamMaker
does, drawn from reduced fuzzer findings and code compiled by the tests. The `compare_dreammaker`
example compiles each world with the DreamMaker binary you supply, compiles every proc with dmasm
for the same BYOND build, and compares the resulting bytecode.

```sh
cargo run --release -p dmasm-compiler --example compare_dreammaker -- BYOND/bin/DreamMaker
```

A proc passes when its bytecode matches DreamMaker's, is equivalent, or is a known difference.
Equivalent code differs only in encoding (for example, spelling out a cache load, calling a proc by
name instead of by ID, or placing labels differently). Any other difference fails the run. The
comparison skips debug line records and the global init proc.

CI runs this comparison on Linux against every BYOND build it tests, and a run fails if no test case
applies to its build.

Each test case begins with a `// Guards:` comment explaining what the case covers and runs until the
next `// Guards:` line. Optional directives can follow:

- `// Builds: A-B` gives the build range whose DreamMaker accepts the case.
- `// Known [A-B]: PATH  WHY` marks a proc that differs intentionally, with `WHY` stating the reason.
  It can also mark an unfixed bug, in which case `WHY` notes that.
- `// Known [A-B] on glibc 2.41: PATH  WHY` marks a proc that differs only when DreamMaker rounds
  inverse trigonometric radians to `f32`, which Linux DreamMaker does on glibc 2.41 and later. The
  comparison detects this by compiling `arctan(2)` first. dmasm must still compile and assemble the
  proc and warn on each folded angle. Only the pushed angles may differ, and each of DreamMaker's
  values must match the value reported in its warning.

## License

dmasm is licensed under GPL-3.0. See [dmasm/LICENSE](../dmasm/LICENSE).
