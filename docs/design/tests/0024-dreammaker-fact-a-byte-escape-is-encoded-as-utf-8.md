# Byte escapes in DreamMaker strings are encoded as UTF-8

DreamMaker and dmasm encode the escape `"\xff\x41"` as the UTF-8 representation of `ÿ` (bytes
`C3 BF`) followed by `A`, rather than storing the raw byte `FF`. A string built from raw bytes
compares equal to the compiled string only if it uses the same encoding.

In `auxtest`, `test_string` builds the string from bytes `C3 BF 41` with `Value::from_string_raw`
and checks through `text_lengths` that DM reads its second character as `65` (`A`).

## Applies to

- `tests/auxtest/src/strings.rs`: `test_string`
