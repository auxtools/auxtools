# Strings returned to DM are truncated at a NUL

Functions defined with `byond_ffi_fn` are `extern "C"` entry points called by BYOND, so panicking
inside one aborts the world process. Calling `CString::new(...).unwrap()` would panic on any string
containing an interior NUL byte. `byond_return` instead truncates returned strings at the first NUL
byte, and returns the empty string when the result is empty or missing.

## Applies to

- `auxtools/src/byond_ffi.rs`: `byond_return`
