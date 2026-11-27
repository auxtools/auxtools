# The `auxtools` Rust library

The `auxtools` crate lets native Rust libraries hook DM procs, read and mutate BYOND values, lists,
and strings, call DM procs, and subscribe to runtime errors. For supported BYOND builds, see
[Supported BYOND builds](byond-builds.md), and for an overview of auxtools, see
[README.md](../README.md).

## Setup

The `auxtools` crate is not published on crates.io, so depend on this repository by Git release tag.
Build your crate as a 32-bit `cdylib` targeting `i686-unknown-linux-gnu` or `i686-pc-windows-msvc`
(minimum supported Rust version: `1.88`):

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
auxtools = { git = "https://github.com/auxtools/auxtools", tag = "v4.0.0" }
```

## Writing a hook

A `#[hook]` replaces a DM proc's body with a Rust function once DM calls `auxtools_init`:

```rust
use auxtools::*;

#[hook("/proc/add_one")]
fn add_one(x: Value) {
	Ok(Value::from(x.as_number()? + 1.0))
}
```

The world must declare the hooked proc, provide `/proc/auxtools_stack_trace(msg)` for error
reporting, call `auxtools_init` before invoking hooks, and call `auxtools_shutdown` before rebooting
or exiting:

```dm
/proc/add_one(x)
	CRASH("mylib is not loaded")

/proc/auxtools_stack_trace(msg)
	CRASH(msg)

/proc/mylib()
	return world.system_type == MS_WINDOWS ? "mylib.dll" : "./libmylib.so"

/world/New()
	var/result = call_ext(mylib(), "auxtools_init")()
	if(result != "SUCCESS")
		world.log << "mylib: [result]"
	. = ..()

/world/Reboot()
	call_ext(mylib(), "auxtools_shutdown")()
	. = ..()

/world/Del()
	call_ext(mylib(), "auxtools_shutdown")()
	. = ..()
```

## Lifecycle, error handling, and thread safety

`auxtools_init` returns `"SUCCESS"` on success or `"FAILED (...)"` with a reason. Returning
`Err(Runtime)` or panicking inside a hook is caught before unwinding into BYOND and forwarded to
`/proc/auxtools_stack_trace` as a DM runtime error.

Touch BYOND values only on BYOND's main thread while a world is live (between `auxtools_init` and
`auxtools_shutdown`), and do not store a `Value` past the hook call that produced it. Keep a
`WeakValue` instead.

The library API may change between major versions. [tests/auxtest](../tests/auxtest) and its host
world in [tests/auxtest_host](../tests/auxtest_host) exercise most of the public API.
