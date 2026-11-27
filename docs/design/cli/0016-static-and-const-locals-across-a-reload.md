# Matching static and const local variables across a reload

BYOND stores a proc's static and const local variables among the globals, under the same name as any
global variable of that name, so the running world cannot tell them apart on its own. The only
signal that separates them is the order of reads of that name in the proc's old bytecode, where each
read targets either one of the proc's locals of that name or the global.

A static variable's value lives in its slot. Giving an existing static a new slot during a reload
would lose its value, and sending a static the world already has would allocate a second slot and
run its initializer again. A static that no code reads is absent from the bytecode altogether.
Without the read lists described below, a reload that redeclares a global could overwrite a static
of the same name, or the other way around.

## Information sent to the world

The world uses these read lists to distinguish static and const locals from globals of the same
name, and remembers their slots as local slots even after the locals are renamed:

- **`LocalReads`:** Sent for each reloaded proc whose old or new code has static or const locals,
  per variable name, giving the old code's reads of that name. Each read is either the local's
  number among the proc's locals of that name, or `None` for a global. When dmasm could not compile
  the old code, `reads` is empty and `parsed` reports what the parsed body shows instead:
  `ParsedReads::Only(n)` when every read is of local `n`, or `ParsedReads::Unknown` when a read may
  be of a global or of a type's variable of that name, or when the body reads more than one local of
  that name.
- **`LocalReads::kept`:** The new code's locals of that name that keep the slot of an old local,
  encoded as pairs of the new local's number and the old local's number. The old code read each such
  slot as that local.
- **Removed procs:** For each removed proc that declares a static or const local, the old code's
  reads of the local's name. The world uses them to continue recognizing those slots as locals after
  the proc is gone.
- **Procs left as built:** For each proc whose statics are initialized by global init and that the
  reload leaves as built, the old code's reads. Each of their locals keeps its slot, and the world
  finds the statics' slots using these reads.
- **`ShadowingProc`:** Each proc that declares a static or const local sharing a name with a global
  redeclared by the plan, together with its old code's reads of that name, so the world can tell the
  local from the global.

## Which locals keep their slots

- A static the world already has keeps its slot and its value.
- A local of the new code corresponds to the old code's local at the same position among the blocks,
  and keeps that local's slot only when the old code read the slot as that local.
- A const that keeps its slot and is read by the new code is sent marked `constant`, so the world
  sets its slot to the value.
- The initializers of new statics that run once count as reads by the new code.
- A local that does not keep a slot is sent as a new static, except for a local of the old code that
  the new code does not read either (its initializer already ran when the world started, so it is
  not sent).
- An old static whose initializer runs at world start is not sent either, because the world finds
  its slot from where the initializer sets it.
- Locals that matching refuses (described below) receive new slots.

## Refusing ambiguous matches

A reload is refused when it leaves static or const locals of one name in blocks of the same kind
that cannot be matched one-to-one with the old code's locals, making it impossible to determine
which of them should keep their values. The refusal reads `"cannot match static or const local
variables with this name in blocks of the same kind (N now, M before), so auxtools cannot determine
which ones keep their values"`. The need for a rebuild is named only by the `"rebuild required"`
summary of the event that carries the refusals.

## Fallback reads from the parsed body

When read information comes from the parsed body instead of compiled bytecode, a read that may refer
to a global or to a type's variable makes the result `ParsedReads::Unknown`. Parameter defaults run
before the proc body, when none of the body's locals have been declared yet, so they are scanned
first as reads of globals. A parameter with the same name as a local hides that local.

Changing the reload plan or these types changes the wire protocol (see
[protocol compatibility](0005-protocol-compatibility.md)).

## Applies to

- `auxtools-protocol/src/messages.rs`: `LocalReads`, `ParsedReads`, `ShadowingProc`,
  `HotReloadRequest`
- `auxtools-cli/src/reload.rs`: `unmatched_locals`, `shadowing_procs`, `without_existing_statics`,
  `removed_locals`, `owners_locals`
- `auxtools-cli/src/reload/reads.rs`: `parsed_reads`
