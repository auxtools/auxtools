# dmasm and the DM bytecode instruction set

dmasm converts a BYOND proc's bytecode to named instructions and back. This specification documents
every instruction dmasm decodes, including the values each instruction pops from and pushes onto the
operand stack and a summary of its behavior.

This document describes BYOND 516 as of 2026-10-04. Unless a row names another build, its details
were checked against build 516.1687.

Keep three caveats in mind when relying on any row:

- **`bytecode/src/instructions.rs` is authoritative for instructions and operands.** If this
  document disagrees with that file, `instructions.rs` is right. On 2026-10-04, its operand counts
  were compared with the interpreter's handlers across eleven Windows builds (516.1659–516.1688) and
  agreed, though that check could not cover `Input` or `InputColor` and cannot see `Variable`
  operands (so it would not catch a missing `Variable`).
- **Behavior descriptions are summaries.** If a description disagrees with the BYOND binary, the
  binary is right.
- **Check the `Basis` column for the strength of evidence.** Roughly a quarter of the rows are
  marked `dm`, meaning their stack effect was inferred from the bytecode DreamMaker emits around the
  instruction rather than by reading the interpreter handler.

The instruction names belong to dmasm, not BYOND. Some are guesses, and two (`Log10` and
`LocateRef`) are known misnomers noted in their rows.

## Bytecode shape

- A proc's bytecode is an array of 32-bit words, and offsets count words rather than bytes.
- Each instruction consists of one opcode word followed by zero or more operand words.
- BYOND's opcodes span `0x000..=0x18C` (397 slots). dmasm decodes 387 of them, and the remaining 10
  are listed under [Opcode slots dmasm does not decode](#opcode-slots-dmasm-does-not-decode).
- Jump targets are absolute word offsets within the same proc. The interpreter stores its position
  in a 16-bit field and reads jump operands as 16 bits.

## VM state

Instructions read and write the following virtual-machine state:

| State | Description | Written by | Read by |
| --- | --- | --- | --- |
| Operand stack | Array of 8-byte Values (tag, data). The first ten live in a buffer inside the proc's frame before moving to the heap | Most instructions | Most instructions |
| Test flag | One byte, true or false | `Test`, `IsLoc`/`IsMob`/`IsObj`/`IsArea`/`IsTurf`/`IsMovable`, `IsNaN`, `IsInf`, `IsIn`, `IterNext`, `TestEquiv`, `TestNotEquiv` | `Jz`, `Jnz`, `JzLoop`, `JnzLoop`, `GetFlag` |
| `dot` | The proc's `.` variable (its default return value) | `SetVar dot`, compound assignment on `dot` | `End`, `GetVar dot` |
| `cache` | One Value holding the object or list whose member is being accessed | The `SetCache` variable form, `SetCacheJmpIfNull`, `SetCachePopJmpIfNull`, `PopCache` | Field reads and writes, calls by name |
| `cache_key` | One Value holding the index into `cache` | `SetVar cache_key`, `PopCacheKey` | The `cache[cache_key]` variable form |
| `src`, `usr`, `args`, locals | Values in the current call frame | `SetVar` and related instructions | `GetVar` and related instructions |
| Active iterator | Array, length, index, type filter, and kind byte for the innermost `for ... in` loop | `IterLoad`, `IterNext` | `IterNext`, `KeyValueIter` |
| Saved iterators | Linked stack of outer loops' iterators | `IterPush` | `IterPop` |
| Catch frames | Linked stack with one node per open `try` | `Try` | `Catch`, `TryJmp`, a thrown error |
| Loop budget | Countdown that catches runaway loops | Every `*Loop` jump and `TryJmp` | Same |
| File and line | Source location reported for runtime errors | `DbgFile`, `DbgLine` | Error reporting |

Two details of this state are easy to misread:

- **There is no separate cache stack.** `PushCache` and `PushCacheKey` save the register at index 0
  (the bottom) of the operand stack, shifting every live operand up by one slot.
- **The test flag and a pushed Number are distinct.** Several predicates set only the test flag and
  push nothing onto the stack, so DreamMaker emits `GetFlag` afterward when a value is needed.
  Confusing the two throws off stack-depth tracking by one from that point forward.

## Text syntax

dmasm formats disassembled bytecode as text, but cannot parse that text back into bytecode. The
printed format follows four rules:

- One instruction, label, or comment appears per line.
- Comments begin with `;`.
- Labels are an identifier followed by `:`. Disassembled labels print as `LAB_XXXX`, using the
  target word offset in four uppercase hexadecimal digits.
- Instructions print as their name followed by space-separated operands.

## Operands

| Operand | Words | Printed as | Encoding |
| --- | --- | --- | --- |
| `u32` | 1 | Decimal | The word itself |
| `i32` | 1 | Signed decimal | The word's bits interpreted as signed |
| `Label` | 1 | `LAB_0012` or a name | Absolute word offset in the proc |
| `DMString` | 1 | `"text"` | String-table ID. An ID in `0xFFCD..=0xFFEF` is stored with bit `0x1000_0000` set so it cannot be read as an access modifier |
| `Proc` | 1 | The proc's path | Proc-table ID |
| `Value` | 2, or 3 for a Number | See [Value](#value) | See [Value](#value) |
| `Variable` | 1 or more | See [Variable](#variable) | See [Variable](#variable) |
| `TypeFilter` | 1 | `(mob \| obj \| )` | Bit flags (see [Type filter](#type-filter)) |
| `IsInParams` | 1 | `Range`, `Value`, `BlockCorners`, `BlockCoords` | `0x0B`, `0x05`, `0x06`, `0x13` |
| `SwitchParams` | Varies | `default => L, v => L, ` | Count, then `(Value, Label)` pairs, then the default `Label` |
| `PickSwitchParams` | Varies | Same | Count, then `(u32 threshold, Label)` pairs, then the default `Label`. Each threshold is the cumulative weight so far, scaled so all weights sum to 65535 |
| `SwitchRangeParams` | Varies | Same, with ranges as `(lo to hi) => L` | Range count, `(Value lo, Value hi, Label)` triples, then a `SwitchParams` |
| `PickProbParams` | Varies | `L, L, ` | Count, then that many `Label`s |

### Value

A `Value` operand encodes a constant DM value. Most tags occupy two words:

```text
word0 = tag in bits 0..7, data bits 16..23 in bits 8..15
word1 = data bits 0..15
```

This limits a constant's data to 24 bits. A Number occupies three words instead: `0x2A`,
followed by the upper 16 bits of the `f32`, followed by the lower 16 bits.

dmasm accepts the following tags in a `Value` operand and stops disassembly with an unknown-value
error on any other tag:

| Tag | Printed as |
| --- | --- |
| `0x00` with data 0 | `null` |
| `0x06` | Quoted string |
| `0x2A` | Number |
| `0x08` `0x09` `0x0A` `0x0B` `0x24` `0x26` `0x28` `0x3B` `0x3F` `0x59` | Type or proc path |
| `0x20` | `ref(...)`, kept as raw tag and data so a consumer can rebuild the datum type path |
| `0x0C` | `'resource path'` |
| `0x27` with data 0 | `/file` |
| `0x29` | `ref(...)` |

### Variable

A `Variable` operand names a location to read, write, or call, with its first word selecting the
form:

- Any word outside `0xFFCD..=0xFFEF` is a string ID naming a field of the value in `cache`, printed
  as `cache["name"]`.
- Any word inside `0xFFCD..=0xFFEF` is an access modifier, followed by its own operands.

| Word | Form | Extra operands | Refers to |
| --- | --- | --- | --- |
| `0xFFCD` | `usr` | - | The proc's `usr` |
| `0xFFCE` | `src` | - | The proc's `src` |
| `0xFFCF` | `args` | - | The argument list |
| `0xFFD0` | `dot` | - | The `.` variable |
| `0xFFD5` | `*var` | A `Variable` | Pointer dereference |
| `0xFFD8` | `cache` | - | The `cache` register |
| `0xFFD9` | `arg(n)` | `u32` | Argument `n` |
| `0xFFDA` | `local(n)` | `u32` | Local `n` |
| `0xFFDB` | `global("name")` | `u32` | Slot in the global variable array. dmasm prints the name, but the slot identifies the variable (two `var/static` declarations with the same name in one proc occupy two slots) |
| `0xFFDC` | `cache = a; b` | Two `Variable`s | Load `a` into `cache`, then evaluate `b` on it (`a.b` compiles to this form) |
| `0xFFDD` | `dynamic_proc("name")` | `DMString` | Proc looked up by name on `cache` |
| `0xFFDE` | `dynamic_verb("name")` | `DMString` | Verb looked up by name on `cache` |
| `0xFFDF` | `static_proc(path)` | `Proc` | Proc by ID |
| `0xFFE0` | `static_verb(path)` | `Proc` | Verb by ID |
| `0xFFE3` | `cache_key` | - | The `cache_key` register |
| `0xFFE4` | `cache[cache_key]` | - | Element of `cache` at `cache_key` |
| `0xFFE5` | `world` | - | The world |
| `0xFFE6` | `null` | - | `null` |
| `0xFFE7` | `initial(var)` | A `Variable` | The variable's compile-time default |
| `0xFFE8` | `issaved(var)` | A `Variable` | Whether the variable is saved |
| `0xFFEF` | `&var` | A `Variable` | Pointer to the variable |

Words `0xFFD1`–`0xFFD4`, `0xFFD6`, `0xFFD7`, `0xFFE1`, `0xFFE2`, and `0xFFE9`–`0xFFEE` fall inside
the modifier range but have no form in dmasm, so disassembly stops on any of them.

Individual instruction handlers do not decode `Variable` operands themselves. Two shared helper
functions (one for reads and one for writes) consume the operand words, which is why handlers such
as `SetVar` contain no operand decoding of their own.

A call through a bare `dynamic_proc` without a leading `cache = ...;` uses whatever object an
earlier instruction left in `cache`, possibly before a branch. This commonly comes from `o?.f(a)`,
where `SetCacheJmpIfNull` populates `cache` and the following `Call` omits `cache = ...;`.

### Argument counts and the 65535 marker

An `arg_count` operand specifies how many Values the instruction pops from the stack. On `Call`,
`CallStatement`, and `CallParentArgs`, the value **65535 is not a count**: it indicates that the
stack holds a single list Value whose elements become the callee's arguments. Both `f(arglist(L))`
and `f(a = 1, b = 2)` compile to this encoding. Global calls use `CallGlobalArgList` instead of the
65535 marker. See [Calls](#calls) for the encoding of each call form.

### Type filter

The second operand of `IterLoad` encodes the broad category of a typed loop variable (as in
`for(var/mob/M in L)`) or an `as` clause. Disassembly stops if it encounters an unknown bit.

| Bit | Name | Bit | Name |
| --- | --- | --- | --- |
| `0x1` | mob | `0x200` | icon |
| `0x2` | obj | `0x400` | sound |
| `0x4` | text | `0x800` | message |
| `0x8` | num | `0x1000` | anything |
| `0x10` | file | `0x4000` | datum instances |
| `0x20` | turf | `0x8000` | password |
| `0x40` | key | `0x10000` | command text |
| `0x80` | null | `0x20000` | color |
| `0x100` | area | | |

## Instructions

Table conventions:

- **Instruction** matches the declaration in `bytecode/src/instructions.rs`, including its operands.
- **Stack** is written as `taken -> left` in Values. `flag` indicates that the instruction sets the
  test flag, and `?` means the value has not been determined.
- Stack contents are listed from bottom to top, so in `list, index` the index is on top.
- **Basis** indicates the strength of evidence for the row:

| Basis | Meaning |
| --- | --- |
| `read` | The interpreter handler was read in the BYOND binary, the behavior was measured in a live world, or both |
| `jit` | A JIT compiler built on dmasm compiles this instruction, declares and checks its stack effect on every compile, and tests results against the interpreter |
| `asm` | The interpreter handler was read once in disassembly for this specification on 2026-10-04 (516.1687 Windows), sometimes along with helper functions it calls. Where a row cites DM source that compiles to the instruction, a test world was compiled with `dm.exe` 516.1687 and disassembled with dmasm, without running it |
| `dmasm` | dmasm's compiler emits the instruction for that built-in proc with that argument count (`compiler/src/compiler/builtin_procs.rs`). Because the compiler uses it as a value, the row assumes it pushes one result. The handler has not been read |
| `dm` | `dm.exe` 516.1687 compiled a small proc using the built-in proc and dmasm disassembled it on 2026-10-04. **Taken** is the number of values pushed immediately before the instruction. **Left** is inferred from the following instruction (`Ret` or `Pop` means one value, `GetFlag` means the result went to the test flag, and nothing means zero). Because the handler was not read, the row summarizes documented rather than verified behavior |
| `name` | Only the name and operands are known, so the behavior description is a guess |

| Op | Instruction | Stack | Behavior | Basis |
| --- | --- | --- | --- | --- |
| `0x00` | `End` | 0 -> 0 | Returns `dot` from the proc. Every proc ends with one | jit |
| `0x01` | `New(arg_count: u32)` | n+1 -> 1 | `new T(args)`. Stack is `type, arg0..argN-1`. Allocates the object and runs `New()` before the next instruction | jit |
| `0x02` | `Format(pattern: DMString, arg_count: u32)` | n -> 1 | String interpolation (`"[x] and [y]"`). The pattern marks each embedded expression with a `0xFF` byte and a code. An embedded datum runs its `operator""()` | jit |
| `0x03` | `Output` | 2 -> 0 | `target << value`. Stack is `target, value` | dm |
| `0x04` | `OutputFormat(pattern: DMString, arg_count: u32)` | n+1 -> 0 | `target << "text [x]"`: combines `Format` and `Output` in one instruction. Stack is `target` followed by one value per embed | dm |
| `0x05` | `Stat` | 2 -> 0 | `stat(name, value)` | dm |
| `0x07` | `Link` | 2 -> 0 | `target << link(url)`. Stack is `target, url` | dm |
| `0x08` | `OutputFtp` | 3 -> 0 | `target << ftp(file, name)`. Stack is `target, file, name`. The compiler pushes `null` when `name` is omitted | dm |
| `0x09` | `OutputRun` | 2 -> 0 | `target << run(file)`. Stack is `target, file` | dm |
| `0x0A` | `OutputKey` | 2 -> 0 | `target << key(file)`. Stack is `target, file`. The current compiler accepts the statement without warning. Sends the file through the same helper `OutputFtp` uses with no filename and mode 2 (`ftp` is 0, `browse` of a file is 3, and `browse_rsc` is 4, while `run` was 1 in the 2013 build). Raises `bad file` for a non-file-like value. Client handling of mode 2 was read only in 2013 build 500.1205, where the client saves the file to a temporary file without prompting and then ignores it | asm |
| `0x0B` | `Missile` | 3 -> 0 | `missile(type, start, end)` | dm |
| `0x0C` | `Del` | 1 -> 0 | `del x`. For `del(v)` on a variable, the compiler pushes the value, stores `null` into `v`, and emits `Del` | dm |
| `0x0D` | `Test` | 1 -> 0, flag | Sets the test flag according to the value's truthiness. `null`, `0`, `""`, and erased datums are false. `NaN`, `"0"`, and empty lists are true | read |
| `0x0E` | `Not` | 1 -> 1 | Replaces the value with the Number `1` or `0`, negating its truthiness | read |
| `0x0F` | `Jmp(destination: Label)` | 0 -> 0 | Unconditional jump | read |
| `0x10` | `Jnz(destination: Label)` | 0 -> 0 | Jumps if the test flag is set | asm |
| `0x11` | `Jz(destination: Label)` | 0 -> 0 | Jumps if the test flag is clear | asm |
| `0x12` | `Ret` | 1 -> 0 | Returns the top of the stack from the proc | jit |
| `0x13` | `IsLoc` | 1 -> 0, flag | `isloc(x)`. Performs **no tag test at all**, checking only whether the atom still exists | read |
| `0x14` | `IsMob` | 1 -> 0, flag | `ismob(x)`. Requires tag `0x03` and that the atom still exists | read |
| `0x15` | `IsObj` | 1 -> 0, flag | `isobj(x)`. Requires tag `0x02` and that the atom still exists | read |
| `0x16` | `IsArea` | 1 -> 0, flag | `isarea(x)`. Requires tag `0x04` and that the atom still exists | read |
| `0x17` | `IsTurf` | 1 -> 0, flag | `isturf(x)`. Requires tag `0x01` and that the atom still exists | read |
| `0x18` | `Alert` | 6 -> 1 | `alert()`. dmasm's compiler always pushes six arguments, using `null` for omitted ones. Parks the proc once the prompt is sent | dmasm |
| `0x19` | `EmptyList` | 1 -> 1 | Despite its name, takes a length and pushes a new list filled with that many `null`s. Emitted for `var/list/L[n]` | jit |
| `0x1A` | `NewList(arg_count: u32)` | n -> 1 | `list(a, b, c)` | jit |
| `0x1B` | `View` | 2 -> 1 | `view(dist, center)` in expression position. Either argument may be `null`, and the two may appear in either order | dmasm |
| `0x1C` | `OView` | 2 -> 1 | `oview(dist, center)`, with the same argument rules | dmasm |
| `0x1D` | `ViewTarget` | 2 -> 1 | `view(dist, center)` as the left operand of `<<` (as in `view() << "hi"`). Pushes the shared scratch list directly without copying (see [The shared scratch list](#the-shared-scratch-list)) | asm |
| `0x1E` | `OViewTarget` | 2 -> 1 | Same as `ViewTarget` for `oview()` | asm |
| `0x1F` | `Block` | 2 -> 1 | `block(start, end)` in expression position | dmasm |
| `0x20` | `BlockTarget` | 2 -> 1 | `block(start, end)` as the left operand of `<<`. Pushes the shared scratch list directly without copying | asm |
| `0x21` | `Prob` | 1 -> 1 | `prob(p)`. Pushes the Number `1` or `0` | jit |
| `0x22` | `Rand` | 1 -> 1 | `rand()` and `rand(high)`. The compiler pushes `null` for the zero-argument form. BYOND's built-in `/generator/proc/Rand` also uses it on the generator handle | dm |
| `0x23` | `RandRange` | 2 -> 1 | `rand(low, high)` | dm |
| `0x24` | `Sleep` | 1 -> 0 | `sleep(n)`. `n >= 0` parks the proc and its callers, `-5 < n < 0` parks only when the server is behind, and `n <= -5` runs a string-table self-check without parking | read |
| `0x25` | `Spawn(destination: Label)` | 1 -> 0 | `spawn(delay) { body }`. The label marks where the **parent** continues rather than where the child starts: the frame is copied with its position already at the body, and the parent jumps to the label without yielding | read |
| `0x27` | `BrowseRsc` | 3 -> 0 | `target << browse_rsc(file, name)`. Stack is `target, file, name`. The compiler pushes `null` when `name` is omitted | dm |
| `0x28` | `IsIcon` | 1 -> 1 | `isicon(x)`. Not a tag check: reads the resource to inspect the icon header | read |
| `0x29` | `Call(proc: Variable, arg_count: u32)` | n -> 1 | Calls the proc named by `Variable`. `a.f(x)` encodes as `cache = a; dynamic_proc("f")`, while a bare `dynamic_proc` uses whatever is in `cache`. See [Argument counts and the 65535 marker](#argument-counts-and-the-65535-marker) for 65535 | jit |
| `0x2A` | `CallStatement(proc: Variable, arg_count: u32)` | n -> 1 | Same shape as `Call` and still pushes a result (the compiler emits a `Pop` after it). What distinguishes it from `Call` is not established, though it is **not** value position versus statement position | jit |
| `0x2B` | `CallPath(arg_count: u32)` | n+1 -> 1 | `call(p)(args)`. Stack is `proc, arg0..argN-1`. A proc path calls like `CallGlob`, while a string passes through `text2path()` first so only full paths like `"/proc/foo"` work | read |
| `0x2C` | `CallParent` | 0 -> 1 | `..()` with no arguments written, forwarding the proc's own arguments. Calls BYOND's built-in proc of the same name when there is no DM parent | jit |
| `0x2D` | `CallParentArgs(arg_count: u32)` | n -> 1 | `..(a, b)` | jit |
| `0x2E` | `CallSelf` | 0 -> 1 | `.()`, calling the current proc recursively with no pushed arguments | dm |
| `0x2F` | `CallSelfArgs(arg_count: u32)` | n -> 1 | `.(a, b)`, calling the current proc recursively with `arg_count` arguments | dm |
| `0x30` | `CallGlob(arg_count: u32, proc: Proc)` | n -> 1 | Calls a global proc by ID. The count word comes **before** the proc ID | jit |
| `0x31` | `Log10` | 1 -> 1 | **Misnamed.** One-argument `log(x)` (natural logarithm). Raises for `x <= 0` | read |
| `0x32` | `Log` | 2 -> 1 | `log(base, x)`. Stack is `base, x`. Not base 10. Raises for `x <= 0`, `base <= 0`, or `base == 1` | read |
| `0x33` | `GetVar(var: Variable)` | 0 -> 1 | Pushes the variable's value | jit |
| `0x34` | `SetVar(var: Variable)` | 1 -> 0 | Stores the top of the stack into the variable | read |
| `0x35` | `SetVarExpr(var: Variable)` | 1 -> 1 | Like `SetVar`, but leaves the stored value on the stack for value-position assignments such as `b = (a = 7)` | read |
| `0x36` | `GetFlag` | 0 -> 1 | Pushes the test flag as the Number `1` or `0` | asm |
| `0x37` | `Teq` | 2 -> 1 | `a == b`. Pushes the Number `1` or `0` | jit |
| `0x38` | `Tne` | 2 -> 1 | `a != b` | jit |
| `0x39` | `Tl` | 2 -> 1 | `a < b` | jit |
| `0x3A` | `Tg` | 2 -> 1 | `a > b` | jit |
| `0x3B` | `Tle` | 2 -> 1 | `a <= b` | jit |
| `0x3C` | `Tge` | 2 -> 1 | `a >= b` | jit |
| `0x3D` | `UnaryNeg` | 1 -> 1 | `-x` | jit |
| `0x3E` | `Add` | 2 -> 1 | `a + b`. Dispatches on the left operand's tag: numbers add, strings concatenate, and a list on the left produces a **new** list with the right operand folded in one level deep. A `null` operand yields the other operand unchanged (see [Operators](#operators)) | read |
| `0x3F` | `Sub` | 2 -> 1 | `a - b`. Numbers subtract, a list on the left produces a new list removing the right operand's elements, and `null` counts as 0 | read |
| `0x40` | `Mul` | 2 -> 1 | `a * b`. `null` counts as 0 | jit |
| `0x41` | `Div` | 2 -> 1 | `a / b`. A zero divisor raises `Division by zero`, while a `null` divisor raises `Undefined operation` | read |
| `0x42` | `Mod` | 2 -> 1 | `a % b`. Truncates both operands to integers first. A zero or `null` divisor raises `Division by zero` | read |
| `0x43` | `Round` | 1 -> 1 | One-argument `round(x)`, which computes `floor(x)` | jit |
| `0x44` | `RoundN` | 2 -> 1 | `round(a, b)`: `floor(a / b + 0.5) * b` | jit |
| `0x45` | `AugAdd(var: Variable)` | 1 -> 0 | `var += x`. Mutates a list in place, preserving its identity | read |
| `0x46` | `AugSub(var: Variable)` | 1 -> 0 | `var -= x`, mutating a list in place | read |
| `0x47` | `AugMul(var: Variable)` | 1 -> 0 | `var *= x` | jit |
| `0x48` | `AugDiv(var: Variable)` | 1 -> 0 | `var /= x` | jit |
| `0x49` | `AugMod(var: Variable)` | 1 -> 0 | `var %= x` | jit |
| `0x4A` | `AugBand(var: Variable)` | 1 -> 0 | `var &= x`, mutating a list in place | read |
| `0x4B` | `AugBor(var: Variable)` | 1 -> 0 | `var \|= x`, mutating a list in place | read |
| `0x4C` | `AugXor(var: Variable)` | 1 -> 0 | `var ^= x`, mutating a list in place | read |
| `0x4D` | `AugLShift(var: Variable)` | 1 -> 0 | `var <<= x` | jit |
| `0x4E` | `AugRShift(var: Variable)` | 1 -> 0 | `var >>= x` | jit |
| `0x50` | `PushInt(value: i32)` | 0 -> 1 | Pushes the integer as a Number | jit |
| `0x51` | `Pop` | 1 -> 0 | Discards the top of the stack | jit |
| `0x52` | `IterLoad(unk0: u32, types: TypeFilter)` | 1, 2, or 6 -> 0 | Starts a `for ... in` loop. The first operand is the iterator kind: 5 for a list or list-like value (takes 1), 20 for `for(k, v in L)` (takes 1), 6 for `block(a, b)` (takes 2), 7 for `view`, 8 for `oview`, 13 for `range`, 14 for `orange`, and 15–18 for the rest of that family (each takes 2), and 19 for six-argument `block()` (takes 6). Lists are copied here so later mutations do not affect the iteration. `null`, numbers, and strings iterate zero times | read |
| `0x53` | `IterNext` | 0 -> 1, flag | Sets the test flag to whether an element remained and pushes that element (or `null` when exhausted), skipping elements rejected by the type filter. Kind 20 pushes two Values: key, then value | read |
| `0x54` | `IterPush` | 0 -> 0 | Saves the active iterator before entering a nested loop | read |
| `0x55` | `IterPop` | 0 -> 0 | Frees the active iterator and restores the saved one. Emitted only to close a nested loop, as a top-level loop gets neither this nor `IterPush` | read |
| `0x56` | `Num2TextSigFigs` | 2 -> 1 | `num2text(x, digits)`. Stack is `x, digits`. Evaluates `sprintf("%.*g", clamp(digits, 0, 100), x)` | read |
| `0x57` | `Roll` | 2 -> 1 | `roll(dice, sides)` | dm |
| `0x58` | `NewListTarget(arg_count: u32)` | n -> 1 | `list(a, b)` as the left operand of `<<`. Moves `arg_count` values into the shared scratch list and pushes it without copying | asm |
| `0x59` | `Range` | 2 -> 1 | `range(dist, center)`. Populates the shared scratch list and pushes it without copying (see [The shared scratch list](#the-shared-scratch-list)). The compiler emits `CopyList` immediately afterward except when the range is the left operand of `<<` | asm |
| `0x5A` | `LocatePos` | 3 -> 1 | `locate(x, y, z)`. Truncates coordinates toward zero. Zero, negative, out-of-range, or non-numeric coordinates yield `null` rather than raising an error | read |
| `0x5B` | `LocateRef` | 1 -> 1 | **Misnamed (covers more than refs).** Handles all one-argument `locate(X)` calls: a `[0x...]` reference string, an atom `tag` string, or an atom type path (first instance of that type). `locate(/datum/foo)` is always `null`, and `locate(/area/foo)` creates the area if none exists | read |
| `0x5C` | `Flick` | 2 -> 0 | `flick(icon, object)` | dm |
| `0x5D` | `Shutdown` | 0 -> 0 | Zero-argument `shutdown()` | dm |
| `0x5E` | `Startup(arg_count: u32)` | n -> ? | `startup()`. Parks the proc once the child world starts | dmasm |
| `0x5F` | `RollStr` | 1 -> 1 | One-argument `roll(x)`, such as `roll("2d6")` | dm |
| `0x60` | `PushVal(value: Value)` | 0 -> 1 | Pushes a constant | read |
| `0x61` | `NewImage` | 2 -> 1 | Two-argument `image(icon, loc)`. Six arguments compile to `NewImageArgs`, named arguments to `NewImageArgList`, and other counts were not probed | dm |
| `0x62` | `PreInc(var: Variable)` | 0 -> 1 | `++var` in value position: adds 1 and pushes the new value | jit |
| `0x63` | `PostInc(var: Variable)` | 0 -> 1 | `var++` in value position: pushes the old value, then adds 1. On a `null` variable, the pushed old value is `null` rather than `0` | jit |
| `0x64` | `PreDec(var: Variable)` | 0 -> 1 | `--var` in value position | jit |
| `0x65` | `PostDec(var: Variable)` | 0 -> 1 | `var--` in value position | jit |
| `0x66` | `Inc(var: Variable)` | 0 -> 0 | `var++` in statement position | jit |
| `0x67` | `Dec(var: Variable)` | 0 -> 0 | `var--` in statement position | jit |
| `0x68` | `Abs` | 1 -> 1 | `abs(x)` | jit |
| `0x69` | `Sqrt` | 1 -> 1 | `sqrt(x)` | jit |
| `0x6A` | `Pow` | 2 -> 1 | `base ** exponent`. Stack is `base, exponent` | jit |
| `0x6B` | `Turn` | 2 -> 1 | `turn(A, angle)`. A number is treated as a direction and rotated in 45-degree steps, discarding any remainder. Also handles icons, matrices, and vectors. An atom's own `operator_turn` never runs | read |
| `0x6C` | `AddText(arg_count: u32)` | n -> 1 | `addtext(a, b, ...)` | jit |
| `0x6D` | `Length` | 1 -> 1 | `length(x)`. Returns byte length for strings | jit |
| `0x6E` | `CopyText` | 3 -> 1 | `copytext(text, start, end)` | dmasm |
| `0x6F` | `FindText` | 4 -> 1 | `findtext(haystack, needle, start, end)`. Positions count bytes, with full Unicode case folding (not just ASCII) | jit |
| `0x70` | `FindTextEx` | 4 -> 1 | Case-sensitive `findtextEx(...)` | dmasm |
| `0x71` | `CmpText` | 2 -> 1, flag | `cmptext(a, b, ...)`. Compares the top two values into the test flag and keeps the lower value on the stack, matching `Teq`'s shape. The compiler chains calls as `a, b, CmpText, Jz end, c, CmpText`, followed by `Pop` and `GetFlag` | dm |
| `0x72` | `SortText(arg_count: u32)` | n -> 1 | `sorttext(a, b, ...)` | dmasm |
| `0x73` | `SortTextEx(arg_count: u32)` | n -> 1 | `sorttextEx(a, b, ...)` | dmasm |
| `0x74` | `UpperText` | 1 -> 1 | Unicode-aware `uppertext(t)` | jit |
| `0x75` | `LowerText` | 1 -> 1 | Unicode-aware `lowertext(t)` | jit |
| `0x76` | `Text2Num` | 1 -> 1 | `text2num(t)`. Returns `null` if the text is not a number | jit |
| `0x77` | `Num2Text` | 1 -> 1 | `num2text(x)`: `sprintf("%.*g", 6, x)` (six significant digits) | read |
| `0x78` | `Switch(params: SwitchParams)` | 1 -> 0 | `switch(x)` with plain cases. Jumps to the first case whose tag and data both match exactly, or to the default label | read |
| `0x79` | `PickSwitch(params: PickSwitchParams)` | 0 -> 0 | `pick()` with literal options and compile-time weights (`pick(a, b, c)`, `pick(50; a, 200; b)`, and the statement form). Draws a 16-bit random number and jumps to the first label whose threshold is not below it, or to the default label. Does not touch the stack, as the code at each label pushes its option | asm |
| `0x7A` | `SwitchRange(params: SwitchRangeParams)` | 1 -> 0 | `switch(x)` with `lo to hi` range cases. Checks ranges first in order (matching Numbers only), then plain cases | read |
| `0x7B` | `ListGet` | 2 -> 1 | `L[i]`. Stack is `list, index` | jit |
| `0x7C` | `ListSet` | 3 -> 0 | `L[i] = v`. Stack is `value, list, index`. Writing past the end raises an error and leaks one reference to the value | jit |
| `0x7D` | `IsType` | 2 -> 1 | `istype(value, type)`. Stack is `value, type`. Pushes a Number without modifying the test flag | read |
| `0x7E` | `Band` | 2 -> 1 | `a & b`. Operates on 24-bit integers for numbers. A list on the left computes an intersection, preserving the left side's order and dropping duplicates | read |
| `0x7F` | `Bor` | 2 -> 1 | `a \| b`. A list on the left computes a union, removing duplicates by subtraction (so `list(1) \| list(1,1)` is `list(1,1)`). A `null` operand yields the other operand unchanged | read |
| `0x80` | `Bxor` | 2 -> 1 | `a ^ b`. A list on the left produces elements present in only one operand. A `null` operand yields the other operand unchanged | read |
| `0x81` | `Bnot` | 1 -> 1 | `~x`, 24 bits | jit |
| `0x82` | `LShift` | 2 -> 1 | `a << b`, 24 bits | jit |
| `0x83` | `RShift` | 2 -> 1 | `a >> b`, 24 bits | jit |
| `0x84` | `DbgFile(name: DMString)` | 0 -> 0 | Records the current source filename for error messages | read |
| `0x85` | `DbgLine(line: u32)` | 0 -> 0 | Records the current source line number | read |
| `0x86` | `Step` | 2 -> 0, flag | `step(ref, dir)`. Writes whether the atom moved into the test flag | dm |
| `0x87` | `StepTo` | 3 -> 0, flag | `step_to(ref, target, min)` | dm |
| `0x88` | `StepAway` | 3 -> 0, flag | `step_away(ref, target, max)` | dm |
| `0x89` | `StepTowards` | 2 -> 0, flag | `step_towards(ref, target)` | dm |
| `0x8A` | `StepRand` | 1 -> 0, flag | `step_rand(ref)` | dm |
| `0x8B` | `Walk` | 3 -> 0 | `walk(ref, dir, lag)` | dm |
| `0x8C` | `WalkTo` | 4 -> 0 | `walk_to(ref, target, min, lag)` | dm |
| `0x8D` | `WalkAway` | 4 -> 0 | `walk_away(ref, target, max, lag)` | dm |
| `0x8E` | `WalkTowards` | 3 -> 0 | `walk_towards(ref, target, lag)` | dm |
| `0x8F` | `WalkRand` | 2 -> 0 | `walk_rand(ref, lag)` | dm |
| `0x90` | `GetStep` | 2 -> 1 | `get_step(ref, dir)`. Pushes the neighboring turf or `null`, following `ref`'s `loc` chain to a turf first. Direction bits `0x10` and `0x20` move up and down a z-level | read |
| `0x91` | `GetStepTo` | 3 -> 1 | `get_step_to(ref, target, min)` | dmasm |
| `0x92` | `GetStepAway` | 3 -> 1 | `get_step_away(ref, target, max)` | dmasm |
| `0x93` | `GetStepTowards` | 2 -> 1 | `get_step_towards(ref, target)` | dmasm |
| `0x94` | `GetStepRand` | 1 -> 1 | `get_step_rand(ref)` | dmasm |
| `0x95` | `GetDist` | 2 -> 1 | `get_dist(a, b)`: `max(abs(dx), abs(dy))`. **Ignores z.** Returns `-1` when both are the same object, or infinity when either is not on a turf | read |
| `0x96` | `GetDir` | 2 -> 1 | `get_dir(a, b)`. Returns a direction bit field, or `0` when either is not on a turf. Ignores z | read |
| `0x97` | `LocateType` | 2 -> 1 | `locate(T) in C`. Stack is `T, C`. Returns the first element of `C` whose type is `T` or a subtype. A string `T` performs an atom `tag` lookup rather than a type-path lookup. `C == world` scans every instance | read |
| `0x98` | `Shell` | 1 -> 1 | `shell(command)`. Parks the proc once the command starts | dm |
| `0x99` | `Text2File` | 2 -> 1 | `text2file(text, file)` | dmasm |
| `0x9A` | `File2Text` | 1 -> 1 | `file2text(path)` | dmasm |
| `0x9B` | `FCopy` | 2 -> 1 | `fcopy(src, dst)` | dmasm |
| `0x9E` | `IsNull` | 1 -> 1 | `isnull(x)`. True for tag `0x00` only | read |
| `0x9F` | `IsNum` | 1 -> 1 | `isnum(x)`. True for tag `0x2A` only | read |
| `0xA0` | `IsText` | 1 -> 1 | `istext(x)`. True for tag `0x06` only | read |
| `0xA1` | `StatPanel` | 3 -> 0 | `statpanel(panel, name, value)` | dm |
| `0xA2` | `StatPanelCheck` | 1 -> 0, flag | One-argument `statpanel(panel)` check, storing the result in the test flag | dm |
| `0xA3` | `WaitforBegin` | 0 -> 0 | Start of the obsolete `waitfor` block statement (which the compiler still accepts with the warning `waitfor is now obsolete, since it is the default behavior`). Adds 1 to a one-byte counter in the frame. No named proc in nine compiled SS13 codebases contained one when swept on 2026-09-16 | asm |
| `0xA4` | `WaitforEnd` | 0 -> 0 | End of a `waitfor` block, subtracting 1 from the same counter | asm |
| `0xA5` | `Min(arg_count: u32)` | n -> 1 | `min(a, b, ...)` | jit |
| `0xA6` | `Max(arg_count: u32)` | n -> 1 | `max(a, b, ...)` | jit |
| `0xA7` | `TypesOf(arg_count: u32)` | n -> 1 | `typesof(a, ...)` | dmasm |
| `0xA8` | `CKey` | 1 -> 1 | `ckey(t)`. Lowercases and drops every byte outside `a-z0-9` | jit |
| `0xA9` | `IsIn(params: IsInParams)` | 2, 3, or 7 -> 0, flag | `x in ...`. `Value`: stack `container, x`. `Range` (`x in lo to hi`): stack `lo, hi, x`, true when `hi >= x && x >= lo` (non-numbers read as 0). `BlockCorners`: stack `a, b, x`. `BlockCoords`: six coordinates followed by `x`, padded with `null` by the compiler | read |
| `0xAA` | `Browse` | 2 -> 0 | `target << browse(body)`. Stack is `target, body` | dm |
| `0xAB` | `BrowseOpt` | 3 -> 0 | `target << browse(body, options)`. Stack is `target, body, options` | dm |
| `0xAC` | `FList` | 1 -> 1 | `flist(path)` | dmasm |
| `0xAD` | `ORange` | 2 -> 1 | `orange(dist, center)`. Like `Range`, pushes the shared scratch list, followed by `CopyList` except as the left operand of `<<` | asm |
| `0xAE` | `CopyList` | 1 -> 1 | Replaces the list on top of the stack with a private copy. Emitted after `Range` and `ORange` | asm |
| `0xAF` | `Read` | 1 -> 1 | `source >> var`. Pops the source and pushes the value read, followed by `SetVar` to store it | dm |
| `0xB0` | `Index` | 2 -> 1 | Legacy indexing instruction. Stack is `container, index`. A string index requires a savefile underneath and returns that savefile path, while any other container raises `bad savefile or list`. Any other index is converted to a whole number and looked up by 1-based position in a helper that switches on the container's tag (not read further). `dm.exe` 516.1687 compiles both `F["x"]` and `L[i]` to `ListGet` instead | asm |
| `0xB1` | `PickProb(params: PickProbParams)` | n -> 0 | `pick(prob(w); a, b)`. Pops one weight per label and jumps to the chosen label, where the compiler pushes that option. Options without `prob()` receive weight 100 | dm |
| `0xB2` | `JmpOr(destination: Label)` | 1 -> 0, or stays on the jump | `a \|\| b`. Leaves a truthy value on the stack and jumps, or pops a falsy value and falls through | read |
| `0xB3` | `JmpAnd(destination: Label)` | 1 -> 0, or stays on the jump | `a && b`. Leaves a falsy value on the stack and jumps, or pops a truthy value and falls through | read |
| `0xB4` | `FDel` | 1 -> 1 | `fdel(path)` | dmasm |
| `0xB5` | `CallName(arg_count: u32)` | n+2 -> 1 | `call(recv, "name")(args)`. Stack is `recv, name, arg0..argN-1`. A string receiver uses the legacy library-call spelling and takes the `call_ext` path | read |
| `0xB6` | `ShutdownArgs` | 2 -> 1 | `shutdown(addr, natural)`. Parks the proc when `addr` names a world started by this one | dm |
| `0xB7` | `List2Params` | 1 -> 1 | `list2params(L)` | dmasm |
| `0xB8` | `Params2List` | 1 -> 1 | `params2list(text)` | dmasm |
| `0xB9` | `CKeyEx` | 1 -> 1 | `ckeyEx(t)` | dmasm |
| `0xBA` | `PromptCheck` | 0 -> 0 | Emitted after every `Input` and `InputColor`. Peeks at the top value without popping it, raising `empty prompt list` on tag `0x22` and passing anything else | asm |
| `0xBB` | `Rgb` | 3 -> 1 | `rgb(r, g, b)` | dm |
| `0xBC` | `HasCall` | 2 -> 1 | `hascall(obj, name)` | dmasm |
| `0xBE` | `HtmlEncode` | 1 -> 1 | `html_encode(t)` | dmasm |
| `0xBF` | `HtmlDecode` | 1 -> 1 | `html_decode(t)` | dmasm |
| `0xC0` | `Time2Text` | 2 -> 1 | `time2text(t)` and `time2text(t, format)`. The compiler pushes a `null` format for the one-argument form. Still the common form in BYOND 516 | read |
| `0xC1` | `Input(unk0: u32, unk1: u32, unk2: u32)` | 4 or 5 -> 1 | `input()`. Stack is `recipient, message, title, default`, with the `in` list beneath those four when `unk2` is 64. `unk0` holds the `as` type as type-filter bits (`0` for none). `unk2` selects the choice source: `0` for none, `64` for a list on the stack, `16` for `in world`. `unk1` was `0` in every probe except `127` for `in world` and `3` for `in view(3, M)`, suggesting a distance (unconfirmed). When the recipient is omitted, the compiler pushes the other three values one slot early, pads with `null`, and the handler uses `usr`. Parks the proc once the prompt is sent | asm |
| `0xC2` | `Sin` | 1 -> 1 | `sin(x)` in degrees | jit |
| `0xC3` | `Cos` | 1 -> 1 | `cos(x)` in degrees | jit |
| `0xC4` | `ArcSin` | 1 -> 1 | `arcsin(x)` with result in degrees | jit |
| `0xC5` | `ArcCos` | 1 -> 1 | `arccos(x)` with result in degrees | jit |
| `0xC6` | `InputColor(unk0: u32, unk1: u32, unk2: u32)` | 5 or 6 -> 1 | `input() as color`. Shares code with `Input`, taking one additional number at the bottom of its stack block to carry type bits in place of `unk0` (the compiler pushes `131072` for `as color`) | asm |
| `0xC7` | `Crash` | 1 -> 0 | `CRASH(msg)` | dm |
| `0xC8` | `NewAssocList(arg_count: u32)` | 2n -> 1 | `list(k1 = v1, k2 = v2)`. Stack is `k1, v1, k2, v2, ...`. Also constructs the argument list for named-argument calls | jit |
| `0xC9` | `CallParentArgList` | 1 -> 1 | `..(arglist(L))` and `..(a = 1)`. Takes the argument list. Shape comes from `dm.exe` output, and the handler was not read | read |
| `0xCA` | `CallSelfArgList` | 1 -> 1 | `.(arglist(L))`. Takes the argument list | dm |
| `0xCB` | `CallPathArgList` | 2 -> 1 | `call(p)(arglist(L))`. Stack is `proc, list` | read |
| `0xCC` | `CallNameArgList` | 3 -> 1 | `call(recv, "name")(arglist(L))`. Stack is `recv, name, list` | read |
| `0xCD` | `CallGlobalArgList(proc: Proc)` | 1 -> 1 | Global proc call with a list of arguments: `f(arglist(L))` and `f(a = 1)` | jit |
| `0xCF` | `NewArgList` | 2 -> 1 | `new T(arglist(L))` and `new T(a = 1)`. Stack is `type, list` | dm |
| `0xD0` | `MinList` | 1 -> 1 | `min(L)` on a single list | dm |
| `0xD1` | `MaxList` | 1 -> 1 | `max(L)` on a single list | dm |
| `0xD2` | `Pick` | 1 -> 1 | `pick(L)` on a single list (`pick(a, b, c)` with literal options compiles to `PickSwitch`) | dm |
| `0xD3` | `NewImageArgList` | 1 -> 1 | `image(arglist(L))`, and `image(icon = a, loc = b)` with a preceding `NewAssocList`. Takes the list | dm |
| `0xD4` | `NewImageArgs(arg_count: u32)` | n -> 1 | `image(a, b, ...)` with positional arguments (observed with six) | dm |
| `0xD7` | `FCopyRsc` | 1 -> 1 | `fcopy_rsc(path)` | dmasm |
| `0xD9` | `ShellAllowed` | 0 -> 1 | Zero-argument `shell()`, reporting whether shell access is enabled | dm |
| `0xDA` | `RandSeed` | 1 -> 0 | `rand_seed(seed)` | dm |
| `0xDB` | `Text2Ascii` | 2 -> 1 | `text2ascii(text, pos)`. The compiler pushes `null` when `pos` is omitted | dmasm |
| `0xDC` | `Ascii2Text` | 1 -> 1 | `ascii2text(n)` | dmasm |
| `0xDD` | `IconStates` | 1 -> 1 | `icon_states(icon)` | dm |
| `0xDE` | `IconNew(arg_count: u32)` | n -> 1 | Constructs the icon value backing an `/icon` datum. BYOND's built-in `/icon/New` pushes its five arguments, emits `IconNew 5`, and stores the result in the datum's `icon` variable. `/icon/proc/operator:=` uses `IconNew 1` | dm |
| `0xDF` | `TurnOrFlipIcon(filter_mode: u32, var: Variable)` | 1 -> 0 | `icon.Turn(angle)` or `icon.Flip(dir)` on the icon in `var`, written back in place. BYOND's built-in `/icon` procs pass `filter_mode` 5 for `Turn`, 9 for `Turn` when its second argument is true, and 6 for `Flip`. Distinct from `Turn` (`0x6B`) | dm |
| `0xE0` | `IconBlendSimple(var: Variable)` | 2 -> 0 | `_dm_icon_blend(icon, other, function)` without a position. Stack is `other, function`. Shares code with `IconBlend` at fixed position `(1, 1)`. Because BYOND's built-in `/icon/proc/Blend` always supplies a position, this arises only from DM code calling `_dm_icon_blend` directly | asm |
| `0xE1` | `IconIntensity(var: Variable)` | 3 -> 0 | `icon.SetIntensity(r, g, b)` on the icon in `var`. `/icon/proc/SetIntensity` replaces a `null` `g` or `b` with `-1` first | dm |
| `0xE2` | `IconSwapColor(var: Variable)` | 2 -> 0 | `icon.SwapColor(old, new)` on the icon in `var` | dm |
| `0xE3` | `ShiftIcon(var: Variable)` | 3 -> 0 | `icon.Shift(dir, offset, wrap)` on the icon in `var` | dm |
| `0xE4` | `IsFile` | 1 -> 1 | `isfile(x)`. True for tag `0x0C` or `0x27` | read |
| `0xE5` | `Viewers` | 2 -> 1 | `viewers(depth, center)` | dm |
| `0xE6` | `OViewers` | 2 -> 1 | `oviewers(depth, center)` | dmasm |
| `0xE7` | `Hearers` | 2 -> 1 | `hearers(depth, center)` | dmasm |
| `0xE8` | `OHearers` | 2 -> 1 | `ohearers(depth, center)` | dmasm |
| `0xE9` | `DbNewConnection` | 0 -> 1 | `_dm_db_new_con()`, the internal proc backing database datums | dmasm |
| `0xEA` | `DbNewQuery` | 0 -> 1 | `_dm_db_new_query()` | dmasm |
| `0xEB` | `DbConnect` | 6 -> 1 | `_dm_db_connect(...)` | dmasm |
| `0xEC` | `DbExecute` | 5 -> 1 | `_dm_db_execute(...)` | dmasm |
| `0xED` | `DbNextRow` | 3 -> 1 | `_dm_db_next_row(...)` | dmasm |
| `0xEE` | `DbErrorMsg` | 1 -> 1 | `_dm_db_error_msg(...)` | dmasm |
| `0xEF` | `DbClose` | 1 -> 1 | `_dm_db_close(...)` | dmasm |
| `0xF0` | `DbIsConnected` | 1 -> 1 | `_dm_db_is_connected(...)` | dmasm |
| `0xF1` | `DbRowsAffected` | 1 -> 1 | `_dm_db_rows_affected(...)` | dmasm |
| `0xF2` | `DbRowCount` | 1 -> 1 | `_dm_db_row_count(...)` | dmasm |
| `0xF3` | `DbQuote` | 2 -> 1 | `_dm_db_quote(...)` | dmasm |
| `0xF4` | `DbColumns` | 2 -> 1 | `_dm_db_columns(...)` | dmasm |
| `0xF5` | `IsPath` | 1 -> 1 | One-argument `ispath(x)`. True for the ten type-path tags (excluding proc paths) | read |
| `0xF6` | `IsSubPath` | 2 -> 1 | `ispath(val, type)`. `val` must be a type path (an instance always yields `0`), while `type` may be an instance | read |
| `0xF7` | `FExists` | 1 -> 1 | `fexists(path)`. Returns `null` for anything other than a string or file value, and may raise `Safety violation` in safe mode | read |
| `0xF8` | `JmpLoop(destination: Label)` | 0 -> 0 | `Jmp` for a loop's backward jump, also decrementing the loop budget. At zero, a normal proc logs `"Infinite loop suspected"`, becomes a background proc, and parks. A background proc parks when its time expires | read |
| `0xF9` | `JnzLoop(destination: Label)` | 0 -> 0 | `Jnz` with the same loop-budget check | read |
| `0xFA` | `JzLoop(destination: Label)` | 0 -> 0 | `Jz` with the same loop-budget check | read |
| `0xFB` | `PopN(count: u32)` | n -> 0 | Discards `count` Values. Seen only at the exit of `for(x in a to b)` loops to drop the loop bounds | asm |
| `0xFC` | `Check2Numbers` | 0 -> 0 | Start of `for(x in a to b)`. Raises unless the top two Values are both Numbers, leaving the counter and bound on the stack for the duration of the loop | asm |
| `0xFD` | `ForRange(exit: Label, var: Variable)` | 0 -> 0 | One iteration of `for(x in a to b)`. Stack is `counter, bound`. Jumps to `exit` if the counter exceeds the bound, otherwise stores the counter into `var` and increments the stack copy by 1 | asm |
| `0xFE` | `Check3Numbers` | 0 -> 0 | Start of `for(x in a to b step c)`. Raises unless the top three Values are all Numbers | asm |
| `0xFF` | `ForRangeStep(exit: Label, var: Variable)` | 0 -> 0 | One iteration of a stepped range loop. Stack is `counter, bound, step`. Runs while `bound >= counter` when `step > 0`, and while `counter >= bound` otherwise. Stores the counter into `var`, then adds `step` | asm |
| `0x100` | `DmsNewKernel` | 0 -> 1 | Creates a kernel object for DM Script, a small scripting language built into BYOND (classes `DMSKernelData`, `DMSParser`, and `DMSProcData`), and pushes a handle to it (tag `0x45`, kind 5). Opcodes `0x100`–`0x104` form the VM's entire interface to it, and no probe found DM source that compiles to any of the five | asm |
| `0x101` | `DmsParse` | 2 -> 1 | Stack is `kernel, text`. Runs a DM Script parser over `text` for that kernel, pushing a string on success or `null` otherwise | asm |
| `0x102` | `DmsExportText` | 2 -> 1 | Stack is `kernel, x`. Pushes the kernel's exported text as a string, or `null` when the first value is not a kernel. Pops and ignores the second value | asm |
| `0x103` | `Eval` | 1 -> 1 | Runs DM Script. Compiles and runs a string (raising a runtime error carrying the compiler's message on compile failure), runs a prepared script handle directly, and pushes the result (or `null` for any other input) | asm |
| `0x104` | `DmsPrepare` | 2 -> 1 | Stack is `kernel, text`. Compiles `text` against `kernel` and pushes the compiled script. On compile failure, it appears to push the error text instead (that branch was read in the decompiler only) | asm |
| `0x105` | `IconDrawBox(var: Variable)` | 5 -> 0 | `icon.DrawBox(color, x1, y1, x2, y2)` on the icon in `var` | dm |
| `0x106` | `IconInsert(arg_count: u32, var: Variable)` | n -> 1 | `icon.Insert(new_icon, icon_state, dir, frame, moving, delay)`. BYOND's built-in `/icon/proc/Insert` pushes the icon and those six arguments (count 7). Also writes the icon to `var`, and the proc pops the pushed value. Until 2026-10-04, dmasm omitted `var` here and decoded that word plus the following `Pop` as a spurious `CallPath 81` | asm |
| `0x107` | `UrlEncode` | 2 -> 1 | `url_encode(text, format)`. The compiler pushes `null` when `format` is omitted | dmasm |
| `0x108` | `UrlDecode` | 1 -> 1 | `url_decode(text)` | dmasm |
| `0x109` | `Md5` | 1 -> 1 | `md5(text)` | dmasm |
| `0x10A` | `Text2Path` | 1 -> 1 | `text2path(text)`. Text containing `/proc/` or `/verb/` resolves to a proc, other text starting with `/` resolves to a type, and anything else yields `null` | dmasm |
| `0x10B` | `WinOutput` | 3 -> 0 | `target << output(text, control)`. Stack is `target, text, control` | dm |
| `0x10C` | `WinSet` | 3 -> 0 | `winset(player, control, params)` | dm |
| `0x10D` | `WinGet` | 3 -> 1 | `winget(player, control, params)`. Parks the proc once the query is sent | dmasm |
| `0x10E` | `WinClone` | 3 -> 0 | `winclone(player, window, clone_name)` | dm |
| `0x10F` | `WinShow` | 3 -> 0 | `winshow(player, window, show)` | dm |
| `0x110` | `IconMapColors(arg_count: u32, var: Variable)` | n+1 -> 1 | `icon.MapColors()`. BYOND's built-in `/icon/proc/MapColors` pushes the icon followed by 4, 5, 12, or 20 arguments, and `arg_count` excludes the icon (unlike `IconInsert`). The stack effect is inferred from those pushes rather than from the handler. Writes the icon to `var`, and had the same missing operand in dmasm as `IconInsert` until 2026-10-04 | asm |
| `0x111` | `IconScale(var: Variable)` | 3 -> 1 | `icon.Scale(width, height)`. Stack is `icon, width, height`. Also writes the icon to `var`, and `/icon/proc/Scale` pops the pushed value | dm |
| `0x112` | `IconCrop(var: Variable)` | 5 -> 1 | `icon.Crop(x1, y1, x2, y2)`. Stack is `icon, x1, y1, x2, y2`. Also writes the icon to `var`, and `/icon/proc/Crop` pops the pushed value | dm |
| `0x113` | `Rgba` | 4 -> 1 | `rgb(r, g, b, a)` | dm |
| `0x114` | `IconStatesMode` | 2 -> 1 | `icon_states(icon, mode)`. `/icon/proc/IconStates` also uses it, substituting `0` for a `null` mode | dm |
| `0x115` | `IconGetPixel(arg_count: u32)` | n -> 1 | `icon.GetPixel(x, y, icon_state, dir, frame, moving)`. `/icon/proc/GetPixel` pushes the icon and those six arguments (count 7) | dm |
| `0x116` | `CallLib(arg_count: u32)` | n+2 -> 1 | `call_ext(lib, func)(args)`. Stack is `lib, func, arg0..argN-1`. A `byond,await:` call parks the proc and resumes with `lib` still beneath the result | read |
| `0x117` | `CallLibArgList` | 3 -> 1 | `call_ext(lib, func)(arglist(L))`. Leaks one reference per list item per call | read |
| `0x118` | `WinExists` | 2 -> 1 | `winexists(player, control)`. Parks the proc once the query is sent | dmasm |
| `0x119` | `IconBlend(var: Variable)` | 4 -> 0 | `icon.Blend(other, function, x, y)`. Stack is `other, function, x, y`, matching BYOND's built-in `/icon/proc/Blend`. Blends into the icon in `var` and writes it back | asm |
| `0x11A` | `IconSize` | 2 -> 1 | `icon.Width()` and `icon.Height()`. Stack is `icon, which`, where `which` is `1` for width and `2` for height | dm |
| `0x11B` | `Bounds(arg_count: u32)` | n -> 1 | `bounds(...)`, taking up to five arguments. Shares one handler with `OBounds` on 516.1669 | dmasm |
| `0x11C` | `OBounds(arg_count: u32)` | n -> 1 | `obounds(...)` | dmasm |
| `0x11D` | `BoundsDist` | 2 -> 1 | `bounds_dist(ref, target)` | dmasm |
| `0x11E` | `StepSpeed` | 3 -> 0, flag | `step(ref, dir, speed)` | dm |
| `0x11F` | `StepToSpeed` | 4 -> 0, flag | `step_to(ref, target, min, speed)` | dm |
| `0x120` | `StepAwaySpeed` | 4 -> 0, flag | `step_away(ref, target, max, speed)` | dm |
| `0x121` | `StepTowardsSpeed` | 3 -> 0, flag | `step_towards(ref, target, speed)` | dm |
| `0x122` | `StepRandSpeed` | 2 -> 0, flag | `step_rand(ref, speed)` | dm |
| `0x123` | `WalkSpeed` | 4 -> 0 | `walk(ref, dir, lag, speed)` | dm |
| `0x124` | `WalkToSpeed` | 5 -> 0 | `walk_to(ref, target, min, lag, speed)` | dm |
| `0x125` | `WalkAwaySpeed` | 5 -> 0 | `walk_away(ref, target, max, lag, speed)` | dm |
| `0x126` | `WalkTowardsSpeed` | 4 -> 0 | `walk_towards(ref, target, lag, speed)` | dm |
| `0x127` | `WalkRandSpeed` | 3 -> 0 | `walk_rand(ref, lag, speed)` | dm |
| `0x128` | `Animate` | 1 -> 1 | `animate(...)` with at least one property change. Takes one associative list built by a preceding `NewAssocList`. When an object is supplied, it sits in that list under key `1`, whereas chained `animate(alpha = b)` omits that entry. In statement position, the compiler pops the result | dm |
| `0x129` | `NullAnimate` | 1 -> 1 | `animate(object)` with an object and no changes. Despite the misleading name, the object is its single argument | dm |
| `0x12A` | `MatrixNew(arg_count: u32)` | n -> 1 | `matrix(...)`, taking up to six arguments | dmasm |
| `0x12B` | `Database(arg_count: u32)` | n -> 1 | Handles every database operation, not only creation. All of BYOND's built-in `/database` and `/database/query` procs (`Open`, `Close`, `Error`, `Add`, `Execute`, `NextRow`, `GetRowData`, and the rest) push `src` followed by one or two values and emit `Database 2` or `Database 3`. Which value selects the operation has not been determined | dm |
| `0x12C` | `Try(unk0: Label)` | 0 -> 0 | Opens a `try` block, pushing a catch frame that records the stack depth and handler label. A throw inside the block lands at the label with the thrown value on the stack | read |
| `0x12D` | `Throw` | 1 -> 0 | `throw x` | dm |
| `0x12E` | `Catch(unk0: Label)` | 0 -> 0 | Exits a `try` block normally, jumping to the label and freeing every catch frame whose range the label falls outside. This is a jump rather than the start of the catch handler | asm |
| `0x12F` | `TryJmp(destination: Label)` | 0 -> 0 | Performs the same jump-and-free as `Catch`, then decrements the loop budget like `JmpLoop` | asm |
| `0x130` | `ReplaceText` | 5 -> 1 | `replacetext(haystack, needle, replacement, start, end)`. Byte positions, with Unicode case folding | jit |
| `0x131` | `ReplaceTextEx` | 5 -> 1 | Case-sensitive `replacetextEx(...)` | dmasm |
| `0x132` | `FindLastText` | 4 -> 1 | `findlasttext(haystack, needle, start, end)` | dmasm |
| `0x133` | `FindLastTextEx` | 4 -> 1 | `findlasttextEx(...)` | dmasm |
| `0x134` | `SpanText` | 3 -> 1 | `spantext(haystack, needles, start)` | dmasm |
| `0x135` | `NonSpanText` | 3 -> 1 | `nonspantext(haystack, needles, start)` | dmasm |
| `0x136` | `SplitText` | 5 -> 1 | `splittext(text, delimiter, start, end, include_delimiters)` | dmasm |
| `0x137` | `JoinText` | 4 -> 1 | `jointext(list, glue, start, end)`. Always four Values: the compiler pushes `1` and `null` when `start` and `end` are omitted. Non-text glue joins with nothing | read |
| `0x138` | `JsonEncode` | 1 -> 1 | `json_encode(v)`. Always returns a string | read |
| `0x139` | `JsonDecode` | 1 -> 1 | `json_decode(text)`. Accepts comments by default. A non-string argument fails inside the parser | read |
| `0x13A` | `RegexNew(arg_count: u32)` | n -> 1 | `regex(...)` | dmasm |
| `0x13B` | `FilterNewArgList` | 1 -> 1 | `filter(...)`. Takes one list: either the associative list built by `NewAssocList` for named arguments, or `L` directly for `filter(arglist(L))` | dm |
| `0x13C` | `PushTop` | 0 -> 1 | Duplicates the top of the stack | asm |
| `0x13D` | `SetCacheJmpIfNull(destination: Label)` | 1 -> 0, or stays on the jump | `a?.b` for a read or call. Stores the value into `cache` either way. If `null`, keeps it on the stack as the result and jumps, otherwise pops it | read |
| `0x13E` | `SetCachePopJmpIfNull(destination: Label)` | 1 -> 0 | `a?.b = v` for a write. Pops `null` and jumps, otherwise moves the value into `cache` | asm |
| `0x13F` | `PushEval` | 0 -> 1 | Pushes the result of the immediately preceding compound assignment, for value-position expressions like `(x += 1)`. Takes no operand: the interpreter steps back and re-reads the **previous instruction's** `Variable`. A resumed frame must never start on this instruction | read |
| `0x140` | `TestEquiv` | 2 -> 1, flag | `a ~= b`. Sets the test flag **and** pushes the Number | asm |
| `0x141` | `TestNotEquiv` | 2 -> 1, flag | `a ~! b`. Same as `TestEquiv`, with the result inverted | asm |
| `0x142` | `PushCache` | 0 -> 0 | Saves `cache` at index 0 of the operand stack beneath all live operands, growing stack depth by one | read |
| `0x143` | `PopCache` | 0 -> 0 | Pops index 0 back into `cache`, releasing what `cache` previously held | read |
| `0x144` | `Tan` | 1 -> 1 | `tan(x)` in degrees | jit |
| `0x145` | `ArcTan` | 1 -> 1 | `arctan(x)` with result in degrees | jit |
| `0x146` | `ArcTan2` | 2 -> 1 | `arctan(x, y)`. Stack is `x, y`. Computes `atan2(y, x)` | jit |
| `0x147` | `IsList` | 1 -> 1 | `islist(x)`. Accepts roughly 46 tags covering the list family plus `/alist`, a different set from `istype(x, /list)` | read |
| `0x148` | `Ref` | 1 -> 1 | `ref(x)`. Formats `[0x` + hex of `(tag << 24) \| data` + `]` | read |
| `0x149` | `IsMovable` | 1 -> 0, flag | `ismovable(x)`. Requires tag `0x02` or `0x03` and that the atom still exists | read |
| `0x14A` | `Clamp` | 3 -> 1 | `clamp(value, low, high)`. Stack is `value, low, high`. Swaps bounds if `low > high` | jit |
| `0x14B` | `Sha1` | 1 -> 1 | `sha1(x)` | dmasm |
| `0x14C` | `Text2AsciiChar` | 2 -> 1 | `text2ascii_char(text, pos)` | dmasm |
| `0x14D` | `LengthChar` | 1 -> 1 | `length_char(text)`, counting characters rather than bytes | jit |
| `0x14E` | `CopyTextChar` | 3 -> 1 | `copytext_char(text, start, end)` | dmasm |
| `0x14F` | `FindTextChar` | 4 -> 1 | `findtext_char(...)`, with character positions | jit |
| `0x150` | `FindTextExChar` | 4 -> 1 | `findtextEx_char(...)` | dmasm |
| `0x151` | `ReplaceTextChar` | 5 -> 1 | `replacetext_char(...)` | jit |
| `0x152` | `ReplaceTextExChar` | 5 -> 1 | `replacetextEx_char(...)` | dmasm |
| `0x153` | `FindLastTextChar` | 4 -> 1 | `findlasttext_char(haystack, needle, start, end)`. Always four values: the compiler pushes `0` and `1` for omitted `start` and `end` | dm |
| `0x154` | `FindLastTextExChar` | 4 -> 1 | `findlasttextEx_char(haystack, needle, start, end)`, with the same filled-in defaults | dm |
| `0x155` | `SpanTextChar` | 3 -> 1 | `spantext_char(...)` | dmasm |
| `0x156` | `NonSpanTextChar` | 3 -> 1 | `nonspantext_char(...)` | dmasm |
| `0x157` | `SplitTextChar` | 5 -> 1 | `splittext_char(...)` | dmasm |
| `0x158` | `Text2NumRadix` | 2 -> 1 | `text2num(text, radix)`. Stack is `text, radix` | jit |
| `0x159` | `Num2TextRadix` | 3 -> 1 | `num2text(n, digits, radix)` | dm |
| `0x15A` | `AssignInto(var: Variable)` | 1 -> 0 | `var := x`. Pops the value and writes to `var`. The handler may call an overload proc before finishing in the shared variable store (the rest was not read) | dm |
| `0x15B` | `PushCacheKey` | 0 -> 0 | `PushCache` for the `cache_key` register | read |
| `0x15C` | `PopCacheKey` | 0 -> 0 | `PopCache` for the `cache_key` register | read |
| `0x15D` | `Time2TextTZ(arg_count: u32)` | n -> 1 | Three-argument `time2text(t, format, timezone)` only | read |
| `0x15E` | `MakeGenerator(unk0: u32)` | n -> 1 | Builds the handle backing a `/generator` datum. `generator(a, b, c, d)` compiles to `new /generator(...)`. BYOND's built-in `/generator/New` then pushes four values, emits `MakeGenerator 4`, and stores the result in the datum's `_binobj` variable | dm |
| `0x15F` | `SpliceText` | 4 -> 1 | `splicetext(text, start, end, insert)` | dmasm |
| `0x160` | `SpliceTextChar` | 4 -> 1 | `splicetext_char(...)` | dmasm |
| `0x161` | `RgbEx` | 5 -> 1 | `rgb()` when the compiler cannot prove the color space is plain RGB (per dmasm source comment). For `rgb(a, b, c, space = d)`, the stack is `a, b, c, null, d`, with `null` standing in for the omitted alpha | dm |
| `0x162` | `Rgb2Num` | 2 -> 1 | `rgb2num(color, space)` | dmasm |
| `0x163` | `GradientIndex` | 2 -> 1 | `gradient(list(...), index)` when the gradient is already a single value. Stack is `gradient, index`. Pushes the color as a `#rrggbb` string, or raises `bad gradient` if the first value is not a usable gradient. Added to dmasm on 2026-10-04 | asm |
| `0x164` | `Gradient` | 1 -> 1 | `gradient(...)` with inline items. Pops one list holding the items followed by the index, constructed by a preceding `NewList` (or `NewAssocList` for named arguments), or `L` directly for `gradient(arglist(L))`. Pushes a `#rrggbb` string. `gradient(list(...), index)` compiles to `GradientIndex` instead | asm |
| `0x165` | `LoadResource(arg_count: u32)` | n -> 0 | `target << load_resource(...)`. The count includes the target. Performs the send itself without a following `Output` | read |
| `0x166` | `IsPointer` | 1 -> 1 | True for tag `0x3C` only. No DM built-in proc is confirmed to produce it | read |
| `0x167` | `JsonEncodeFlags(arg_count: u32)` | n -> 1 | `json_encode(v, flags)`. The second Value holds the flags, and `dm.exe` only ever emits count 2. Because `JSON_STRICT` and `JSON_PRETTY_PRINT` share the same numeric value, passing either pretty-prints | read |
| `0x168` | `JsonDecodeFlags(arg_count: u32)` | n -> 1 | `json_decode(text, flags)` | read |
| `0x169` | `Ceil` | 1 -> 1 | `ceil(x)` | jit |
| `0x16A` | `Trunc` | 1 -> 1 | `trunc(x)` | jit |
| `0x16B` | `Fract` | 1 -> 1 | `fract(x)`: `x - trunc(x)` | jit |
| `0x16C` | `IsNaN` | 1 -> 0, flag | `isnan(x)`. Sets the test flag only, like `IsMob` and unlike `IsNum`. Non-numbers evaluate to false | read |
| `0x16D` | `IsInf` | 1 -> 0, flag | `isinf(x)`. Sets the test flag only | read |
| `0x16E` | `TrimText` | 1 -> 1 | `trimtext(text)` | dm |
| `0x16F` | `FTime` | 2 -> 1 | `ftime(file, is_creation)`. Always two values: the compiler pushes `null` when the second argument is omitted | dm |
| `0x170` | `BlockXYZ` | 6 -> 1 | Six-argument `block()` in expression position. Fills the shared scratch list and pushes a private copy (see [The shared scratch list](#the-shared-scratch-list)) | asm |
| `0x171` | `BlockXYZTarget` | 6 -> 1 | Six-argument `block()` as the left operand of `<<`. Pushes the shared scratch list directly without copying | asm |
| `0x172` | `NoiseHash(arg_count: u32)` | n -> 1 | `noise_hash(...)` with `arg_count` arguments | dm |
| `0x173` | `PowSquare` | 1 -> 1 | `x ** 2`, compiled as `x * x` | jit |
| `0x174` | `PowNegativeOne` | 1 -> 1 | `x ** -1`, compiled as `1 / x` | jit |
| `0x175` | `GetStepsTo` | 3 -> 1 | `get_steps_to(ref, target, min)` | dm |
| `0x176` | `FloatMod` | 2 -> 1 | `a %% b`: `a - trunc(a / b) * b`, without the integer truncation performed by `%` | jit |
| `0x177` | `AugFloatMod(var: Variable)` | 1 -> 0 | `var %%= x` | jit |
| `0x178` | `RefCount` | 1 -> 1 | `refcount(x)`. Returns the internal reference count minus one, always `0` for numbers, `null`, strings, turfs, the world, and type paths | read |
| `0x179` | `LoadExt` | 2 -> 1 | `load_ext(lib, func)`. Pushes a handle (tag `0x45`) without invoking anything | read |
| `0x17a` | `CallExtLoaded(arg_count: u32)` | n+1 -> 1 | `call_ext(handle)(args)`. Stack is `handle, arg0..argN-1` | read |
| `0x17b` | `CallExtLoadedArgList` | 2 -> 1 | `call_ext(handle)(arglist(L))`. Stack is `handle, list`. Copies the list's items into an argument array and invokes the same function as `CallExtLoaded`, leaking one reference per list item per call. Added to dmasm on 2026-10-04 | asm |
| `0x17c` | `NewAlist(arg_count: u32)` | 2n -> 1 | `alist(k = v, ...)`. Stack holds one key and one value per pair, and `arg_count` counts pairs | dm |
| `0x17d` | `Spaceship` | 2 -> 1 | `a <=> b`. Invokes an `operator<=>` overload when the left operand defines one (the plain comparison path was not read) | asm |
| `0x17e` | `KeyValueIter(var: Variable)` | 1 -> 0 | Part of `for(k, v in L)`. Pops the value pushed by `IterNext` and stores it into `var`, while a following `SetVar` takes the key. If the loop was downgraded to a plain list walk, pops nothing and stores `null` | read |
| `0x17f` | `NewPixloc(arg_count: u32)` | n -> 1 | `pixloc(...)` with `arg_count` arguments (observed with three) | dm |
| `0x180` | `NewVector(arg_count: u32)` | n -> 1 | `vector(...)` with `arg_count` arguments (observed with two and three) | dm |
| `0x181` | `BoundPixloc` | 2 -> 1 | `bound_pixloc(atom, dir)` | dm |
| `0x182` | `Sin2` | 1 -> 1 | `sin(x)`. BYOND 516 compilers emit either this or `Sin` | jit |
| `0x183` | `Cos2` | 1 -> 1 | `cos(x)` | jit |
| `0x184` | `Tan2` | 1 -> 1 | `tan(x)` | jit |
| `0x185` | `AsType` | 2 -> 1 | `astype(value, type)`. Stack is `value, type`. For one-argument `astype(value)`, the compiler pushes the declared type of the variable being assigned | dm |
| `0x186` | `Sign` | 1 -> 1 | `sign(x)` | jit |
| `0x187` | `Lerp` | 3 -> 1 | `lerp(a, b, factor)` | dm |
| `0x188` | `ValuesSum` | 1 -> 1 | `values_sum(L)`. Iterates associative values only | read |
| `0x189` | `ValuesProduct` | 1 -> 1 | `values_product(L)` | read |
| `0x18a` | `ValuesDot` | 2 -> 1 | `values_dot(A, B)` | read |
| `0x18b` | `ValuesCutUnder` | 3 -> 1 | `values_cut_under(L, min, inclusive)`. The compiler pushes `null` when `inclusive` is omitted. Pushes the count removed. **Under is `0x18B`, over is `0x18C`** | read |
| `0x18c` | `ValuesCutOver` | 3 -> 1 | `values_cut_over(L, max, inclusive)` | read |
| `0x1337` | `AuxtoolsDebugBreak` | - | Not a BYOND opcode (outside `0x000..=0x18C`). Used by the auxtools debugger | name |
| `0x1338` | `AuxtoolsDebugBreakNop` | - | Not a BYOND opcode. Second auxtools debugger opcode | name |

### Operators

Three rules measured in a running world account for most surprises in the arithmetic and bitwise
rows:

- **Null is not always 0.** `-`, `*`, `&`, `<<`, `>>`, `%`, and `%%` treat `null` as `0`. `+`, `|`,
  and `^` return the other operand untouched, so `null | 5.7` is `5.7` and `null + null` is `null`.
  `/` treats a `null` dividend as `0` and raises an error on a `null` divisor.
- **A list on the left changes the operator.** `+`, `-`, `&`, `|`, and `^` each have a list form
  that allocates a new list, while the compound forms (`+=` and so on) mutate the list in place.
- **Bitwise operators work on 24 bits**, because an `f32` holds 24 bits of integer precision.

### Calls

Each call form and the stack layout before execution:

| DM | Instruction | Stack before |
| --- | --- | --- |
| `f(a, b)` (global) | `CallGlob 2 /proc/f` | `a, b` |
| `f(arglist(L))`, `f(a = 1)` (global) | `CallGlobalArgList /proc/f` | `list` |
| `o.f(a, b)` | `Call cache = o; dynamic_proc("f") 2` | `a, b` |
| `o.f(arglist(L))`, `o.f(a = 1)` | `Call cache = o; dynamic_proc("f") 65535` | `list` |
| `o?.f(a)` | `SetCacheJmpIfNull`, then `Call dynamic_proc("f") 1` | `a` |
| `..()` | `CallParent` | Nothing |
| `..(a, b)` | `CallParentArgs 2` | `a, b` |
| `..(arglist(L))` | `CallParentArgList` | `list` |
| `call(p)(a)` | `CallPath 1` | `p, a` |
| `call(o, "f")(a)` | `CallName 1` | `o, "f", a` |
| `call_ext(lib, "f")(a)` | `CallLib 1` | `lib, "f", a` |
| `call_ext(handle)(a)` | `CallExtLoaded 1` | `handle, a` |
| `new T(a, b)` | `New 2` | `T, a, b` |

Every called proc receives a frame padded to its declared parameter count.

### Loops

These listings show `dm.exe` 516.1687 output disassembled with dmasm on 2026-10-04, with debug lines
omitted.

`for(var/x in L)` wraps only nested loops in `IterPush` and `IterPop`. Because the store precedes
`Jz`, the loop also writes the `null` that `IterNext` pushes on exhaustion, keeping the stack empty
along both paths:

```text
          GetVar arg(0)
          IterLoad 5 ()
LAB_0017: IterNext
          SetVar local(1)
          Jz LAB_0027
          ...body...
          JmpLoop LAB_0017
LAB_0027:
```

`for(var/k, v in L)` uses iterator kind 20, with `KeyValueIter` popping the value before `SetVar`
pops the key:

```text
          GetVar arg(0)
          IterLoad 20 ()
LAB_0021: IterNext
          KeyValueIter local(2)
          SetVar local(1)
          Jz LAB_0034
          ...body...
          JmpLoop LAB_0021
LAB_0034:
```

`for(var/i in 1 to n)` keeps its counter and upper bound on the operand stack for the entire loop:

```text
          PushInt 1
          GetVar arg(0)
          Check2Numbers
LAB_0017: ForRange LAB_0025 local(1)
          ...body...
          JmpLoop LAB_0017
LAB_0025: PopN 2
```

`for(var/i in 1 to n step 2)` pushes the step third and uses `Check3Numbers`, `ForRangeStep`, and
`PopN 3`.

### Try and catch

From the same compiler and date as the loop listings above, `Try` carries the label of the `catch`
block, which receives the thrown value on the stack. `Catch` closes the `try` body by jumping past
the `catch` block:

```text
          Try LAB_0011
          ...try body...
          Catch LAB_0021
LAB_0011: SetVar local(0)
          ...catch body...
LAB_0021:
```

## The shared scratch list

The VM maintains a single list with ID 0 and reuses it instead of allocating a fresh list. An
instruction clears list 0, fills it, and pushes a reference that remains valid only until another
instruction fills list 0 again:

- `View`, `OView`, `Block`, and `BlockXYZ` fill list 0 and immediately push a private copy in the
  same handler.
- `Range` and `ORange` push list 0 directly, and the compiler emits `CopyList` afterward.
- On the left side of `<<` (such as `view() << "hi"`), the compiler skips the copy, likely because
  `Output` consumes the list immediately. This position uses `ViewTarget`, `OViewTarget`,
  `BlockTarget`, `BlockXYZTarget`, and `NewListTarget`, and in the cases examined is the only place
  where `Range` and `ORange` omit a following `CopyList`.

`viewers()`, `hearers()`, and `newlist()` have no target-specific variant and compile to the same
instruction on the left of `<<` as anywhere else.

## Opcode slots dmasm does not decode

dmasm stops with an unknown-opcode error if it encounters any of the 10 slots below. No known DM
statement causes `dm.exe` 516.1687 to emit them, and the 2013 compiler
([The 2013 compiler](#the-2013-compiler)) has no code emitting them directly except for `0x26` and
`0xCE` on an unreachable path. On 2026-09-12, a full SS13 codebase of 77,118 named procs
disassembled without errors.

Each row below comes from reading the handler and its callees in 516.1687 Windows disassembly on
2026-10-04, without executing them. Rows `0x26`, `0xCE`, and `0xD5` also draw on 2013 Linux build
500.1205, which retained function names and bundles both the DM compiler and interpreter (see
[The 2013 compiler](#the-2013-compiler)). Where those three rows describe pushed stack values, those
come from the 2013 compiler's code, and its handlers read the same stack slots as 516.1687.

| Op | Operands | Stack | What it does |
| --- | --- | --- | --- |
| `0x06` | None | 3 -> 0 | Legacy `target << sound(file, repeat)`. Stack is `target, file, repeat`. Constructs a sound from `file` (repeating when `repeat` is non-zero) at volume 100 and sends it. The current compiler constructs a `/sound` datum and uses `Output` |
| `0x26` | `arg_count: u32` | n+2 -> 0 | A `spawn` naming a proc in place of a body: queues a call to a proc value (tag `0x26`) after a delay, copying `usr` from the running proc and setting `src` to `null`. Stack is `proc, delay, arg...`, with the 2013 compiler pushing `null` when `delay` is omitted. The handler reads arguments starting two slots above `proc` (matching that layout) but reads `delay` from the top of the stack, which is `delay` only with zero arguments and otherwise reuses the last argument as the delay. No compiler on disk can emit it: although the 2013 compiler contains the code for a bodyless `spawn`, both it and 516.1687 give a bodyless `spawn(P)` an empty body (emitting ordinary `Spawn`) and reject a second argument with `spawn: extra args` |
| `0x4F` | None | 0 -> 0, flag | Flips the test flag |
| `0x9C` | None | 0 -> 0 | No-op |
| `0x9D` | Three words, like `Input` | Same as `Input` | A third entry point into the `input()` handler. `Input` enters in mode 1, `InputColor` in mode 2, and this opcode in mode 0, reading the same stack slots in a different order and parking the proc like the other two |
| `0xBD` | None | 1 -> 1 | Replaces the top of the stack with `null` |
| `0xCE` | None | 3 -> 0 | `arglist(L)` counterpart of `0x26`. Stack is `proc, delay, list`. Passes the list as the single argument with the "arguments come from a list" flag set. Like `0x26`, it reads the delay from the top of the stack, always reading the list rather than `delay`. The 2013 compiler substitutes this for `0x26` when the first argument after `delay` is `arglist()`, on the same unreachable path |
| `0xD5` | `arg_count: u32` | n+1 -> 0 | Legacy `target << sound(file, repeat, wait, channel, volume)` taking 1 to 5 sound arguments: `file` is the first of the `arg_count` values, followed by `repeat`, `wait`, `channel`, and `volume` (defaulting `volume` to 100). The handler reads the target from the `repeat` slot rather than beneath the arguments, reusing `repeat` as the target (and reading one slot past top of stack with one argument). The 2013 handler does the same, and the 2013 compiler already had no code emitting `0xD5`, `0x06`, or `0xD6`. Not executed |
| `0xD6` | None | 2 -> 0 | Legacy `target << S` for a `/sound` datum. Stack is `target, sound`. Reads five variables from the datum and dispatches through the same helper as `0xD5` |
| `0xD8` | None | 0 -> 0 | Bad-instruction handler. Raises `BYOND Error: bad instruction: <number>` and continues |

### Previously undecoded slots

**A codebase disassembling cleanly does not prove a slot is unused.** Twelve slots formerly on the
undecoded list turned out to be live. Prior to 2026-10-04, dmasm could not disassemble any proc
containing them:

| Slots | What emits them |
| --- | --- |
| `0x163`, `0x17B` | `gradient(list(...), index)` and `call_ext(handle)(arglist(L))` |
| `0x0A` | `target << key(file)` |
| `0x1D`, `0x1E`, `0x20`, `0x58`, `0x171` | `view()`, `oview()`, `block()`, or `list()` as the left operand of `<<` |
| `0xAE` | Every `range()` and `orange()`. dmasm previously consumed it as an operand of `Range`, failing on `range() << x` |
| `0xA3`, `0xA4` | Obsolete `waitfor` block statement |
| `0xE0` | Three-argument `_dm_icon_blend()` |

Because a built-in proc can compile to different opcodes depending on syntactic position, test each
built-in as a value, as a loop source, and on the left side of `<<`. Reading the 2013 compiler's
code is the fastest way to spot which positions differ.

## Older names

An earlier version of dmasm used different names for several instructions, and extools uses some of
those same names:

| Op | Older name | Current name |
| --- | --- | --- |
| `0x2A` | `CallNoReturn` | `CallStatement` |
| `0xF8` `0xF9` `0xFA` | `Jmp2` `Jnz2` `Jz2` | `JmpLoop` `JnzLoop` `JzLoop` |
| `0xFC` | `CheckNum` | `Check2Numbers` |
| `0xFE` | `ForRangeStepSetup` | `Check3Numbers` |
| `0x13C` | `BeginListSetExpr` | `PushTop` |
| `0x13D` | `JmpIfNull` | `SetCacheJmpIfNull` |
| `0x13E` | `JmpIfNull2` | `SetCachePopJmpIfNull` |
| `0x13F` | `NullCacheMaybe` | `PushEval` |
| `0x142` `0x143` | `PushToCache` `PopFromCache` | `PushCache` `PopCache` |

`0x35` is named `SetVarExpr` in both versions of dmasm (extools calls it `SETVAR_COPY`).

## Keeping this file current

When dmasm adds or modifies an instruction:

1. Verify the operand count against the interpreter's handler for that opcode. If the handler reads
   a word that dmasm does not declare, the disassembler misinterprets that word as the next opcode
   and fails one or two instructions later.
2. Add or update the instruction's row here, copying its declaration verbatim from
   `bytecode/src/instructions.rs`.
3. Record the weakest `Basis` value that holds for what you actually verified.
4. Update the date near the top of this file.

To identify the DM source that produces an instruction, write the construct in a test `.dme`,
compile it with `dm.exe`, and disassemble the procs in the `.dmb`. That technique uncovered `0x163`
and `0x17B` via unknown-opcode errors and produced the `dm` rows in the table above. Keep three tips
in mind when probing:

- Test each built-in proc in every syntactic position (as a value, as a loop source, and on the left
  side of `<<`), since position can change the emitted opcode.
- Every compiled world includes BYOND's built-in `/icon`, `/sound`, `/database`, `/regex`, and
  `/generator` procs, whose bytecode shows icon and database instructions with real arguments.
- Pass proc arguments rather than literals as inputs, because constant folding can collapse a
  literal-only probe into a single `PushInt`.

### The 2013 compiler

When probing does not trigger an opcode slot, BYOND 500.1205 for Linux (2013) can reveal which
statement emits it. Its `libbyond.so` retained C++ function names and contains both the DM compiler
and the interpreter. Every opcode number checked so far (`0x07`–`0x0A`, `0x25`, `0x26`, `0xCE`, and
`0xD5`) has the same meaning in 500.1205 and 516.

| What | Where in `libbyond.so` 500.1205 |
| --- | --- |
| Emit one bytecode word | `ProcCode::Add(ushort)` (look for a constant passed as its argument) |
| Emit a call instruction | `ProcCode::CallInst(Node*, ushort, Node*, Node*, ushort)`. The opcode is the second argument, and it swaps in the `arglist()` variant automatically |
| Statement and built-in dispatch | `ProcCode::EvalProcBlockInternal(Node*, ulong)`, one large switch on the statement's token ID |
| Token names | An array of `char*` in the data section, where the token ID is the array index plus 1 |

Not every constant passed to `ProcCode::Add` is an opcode: `Add(6)` after `Add(0xDF)` is the mode
operand of `TurnOrFlipIcon`, not opcode `0x06`.

Using this binary established that `0x0A` is `target << key(file)`, that the `*Target` instructions
are emitted for the left side of `<<`, and that `0xA3` and `0xA4` correspond to the `waitfor` block.
The 500.1205 `DreamMaker` executable still runs on Linux and rejects the same `spawn` forms as 516.
