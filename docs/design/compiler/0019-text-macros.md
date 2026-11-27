# Encoding text macros with BYOND's sentence, color, and embed rules

DM text literals can contain macros such as `\the`, `\ref`, `\th`, color macros, and embedded
values `[x]`. DreamMaker encodes them into the string using `0xFF` codes and resolves some of them,
such as sentence-start capitalization, at compile time. Because the encoded string goes directly
into the string table, a single-byte difference changes the string ID, the bytecode, and the
runtime text.

## Sentence start

- Closing a tag restores the sentence-start state from before the tag.
- A lone `>` restores the state from the last tag start, or the string's initial state.
- Sentence detection treats color codes as characters: blue (code `33`) counts as `!`, green and
  black (codes `32` and `34`) count as a space and a quote, and all other color codes end the
  sentence start.

## Colors and styles

- Color macros use codes `31`–`38`, and style macros use codes `23`–`26`.
- A new color macro ends the previous color, and repeating a style macro ends that style.
- A macro that ends a color or style removes the space before it instead of the space after it.
- Repeating the current color resets all colors with code `30`.

## Embedded values and escapes

- In `\ref [x]`, the space goes before the macro, and the macro consumes the value immediately
  after it.
- `\th` immediately after an embedded value turns it into an ordinal, replacing the value's own
  embed code.
- Only the first reference macro applies to the next embedded value, unless an earlier article
  already consumed that value.
- `\ ` produces nothing and marks the end of a line continuation's indentation.

## Applies to

- `dmasm/compiler/src/compiler/strings.rs`: `text_byte` and macro handling
