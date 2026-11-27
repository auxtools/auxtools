use std::result;

use crate::value::Value;

/// An error returned by auxtools API calls and proc hooks.
///
/// Implements `Debug` only (neither `Display` nor `std::error::Error`). When a
/// proc hook returns `Err(Runtime)`, the proc returns `null` and the message is
/// forwarded to `/proc/auxtools_stack_trace` if the world defines it, without
/// raising a BYOND runtime error.
#[derive(Debug)]
pub struct Runtime {
	pub message: String
}

impl Runtime {
	pub fn new<S: Into<String>>(message: S) -> Self {
		Self { message: message.into() }
	}
}

/// Constructs a [`Runtime`] error.
///
/// Passes a single argument directly to `Runtime::new` without formatting. With
/// additional arguments, formats the message with `format!`.
#[macro_export]
macro_rules! runtime {
	($fmt:expr) => {
		$crate::Runtime::new($fmt)
	};
	($fmt: expr, $( $args:expr ),*) => {
		$crate::Runtime::new(format!( $fmt, $( $args, )* ))
	};
}

pub type DMResult<T = Value> = result::Result<T, Runtime>;
