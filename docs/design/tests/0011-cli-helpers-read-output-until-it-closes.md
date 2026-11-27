# Reading CLI output until stream close or deadline

`text_of` in the test support crate reads a command's output in chunks until either the stream
closes or a deadline passes, returning both the collected text and whether the stream closed. Each
stream is read on its own thread.

A process started by an `auxtools` command, such as a background launcher, inherits the command's
output streams and can hold them open after the command exits. Reading only until the command exits
can miss output, while reading until end of stream can wait for the background process. Bounded
chunk reading lets tests capture output from commands that leave a process behind without hanging
until that process exits.

## Applies to

- `tests/support/src/cli.rs`: `chunks_of`, `text_of`
