# dmasm

dmasm assembles, disassembles, and compiles BYOND's DM bytecode. auxtools uses it to evaluate
expressions in the debugger, show a proc's instructions, and hot-reload source changes into a
running world. It consists of three crates (`dmasm-bytecode`, `dmasm-compiler`, and `dmasm-reload`)
plus a `dmasm` crate that re-exports them.

Documentation lives in `docs/`:

- [dmasm](../docs/dmasm.md): Crate overview, supported BYOND builds, tests, and DreamMaker
  comparison
- [Proc compiler](../docs/dmasm-compiler.md): Supported DM constructs, DreamMaker validation,
  known differences, and bug-checker warnings
- [DM bytecode specification](../docs/dmasm-specification.md): Instruction set reference

dmasm is licensed under GPL-3.0. See [LICENSE](LICENSE).
