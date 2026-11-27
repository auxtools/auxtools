# Profiler instruction timing and HTML report escaping

The profiler times DM execution by instruction, recording instruction offsets in words. An
instruction's own time ends when the next instruction starts, excluding proc sleeps and BYOND engine
work between runs of DM code.

The profile report is a single HTML file (`profile.html`) with its data embedded as JSON inside a
`<script>` element. Because a raw `<` inside a `<script>` element could close the element
prematurely (for example, in `</script>`), and JSON contains `<` only inside strings where `\u003c`
has the same meaning, `html` replaces every `<` in the embedded JSON data with `\u003c`.

## Applies to

- `auxtools-protocol/src/messages.rs`: `ProfiledInstruction`, `ProfiledTotal`
- `debug_server/src/profiler.rs`
- `auxtools-cli/src/profile.rs`: `html` (fills `profile.html`)
