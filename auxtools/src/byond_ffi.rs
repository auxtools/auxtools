// https://github.com/tgstation/rust-g/tree/2e9b7724ba81894863d77fd1d27a2e24b79d67bf
//
// MIT License
//
// Copyright (c) 2018 Bjorn Neergaard, /tg/station contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{
	borrow::Cow,
	cell::RefCell,
	ffi::{CStr, CString},
	os::raw::{c_char, c_int},
	slice
};

static EMPTY_STRING: c_char = 0;
thread_local! {
	static RETURN_STRING: RefCell<CString> = RefCell::new(CString::default());
}

/// Converts the `argc` and `argv` arguments passed by BYOND into Rust strings.
///
/// Used internally by `byond_ffi_fn!`. Returns an empty `Vec` when `argc` is
/// `0` or `argv` is null. Invalid UTF-8 sequences are replaced lossily.
///
/// # Safety
///
/// When `argc > 0` and `argv` is non-null, `argv` must point to `argc` valid
/// NUL-terminated C strings. The returned strings borrow BYOND's buffers for
/// the caller-chosen lifetime `'a` and must not outlive the call from BYOND.
pub unsafe fn parse_args<'a>(argc: c_int, argv: *const *const c_char) -> Vec<Cow<'a, str>> {
	if argc == 0 || argv.is_null() {
		return Vec::new();
	}
	slice::from_raw_parts(argv, argc as usize)
		.iter()
		.map(|ptr| CStr::from_ptr(*ptr))
		.map(|cstr| cstr.to_string_lossy())
		.collect()
}

/// Converts `value` into a NUL-terminated C string pointer to return to BYOND.
///
/// Returns a pointer to a static empty string when `value` is `None` or empty.
/// Otherwise copies the bytes (truncating at the first NUL byte) into a
/// thread-local buffer owned by the calling thread. The returned pointer
/// remains valid only until the next call to `byond_return` on the same thread.
pub fn byond_return(value: Option<Vec<u8>>) -> *const c_char {
	match value {
		None => &EMPTY_STRING,
		Some(vec) if vec.is_empty() => &EMPTY_STRING,
		Some(vec) => RETURN_STRING.with(|cell| {
			let cstring = match CString::new(vec) {
				Ok(s) => s,
				Err(e) => {
					let (pos, mut vec) = (e.nul_position(), e.into_vec());
					vec.truncate(pos);
					CString::new(vec).unwrap_or_default()
				}
			};
			cell.replace(cstring);
			cell.borrow().as_ptr()
		})
	}
}

/// Exports a function that DM can invoke with `call_ext`.
///
/// Supports zero arguments (`name() { ... }`), named arguments
/// (`name(a, b) { ... }`), and a trailing variadic slice
/// (`name(a, ...rest) { ... }`). Named arguments are `&str` values converted
/// lossily from UTF-8, defaulting to `""` when DM passes fewer arguments.
///
/// The body runs on BYOND's thread (including before `auxtools_init`) and must
/// evaluate to an `Option` whose inner value converts to bytes (such as
/// `Option<String>`). `None`, empty output, or a caught panic returns an empty
/// string to DM. Output is truncated at the first NUL byte.
///
/// # Examples
///
/// ```ignore
/// byond_ffi_fn! { my_proc(_input) {
///     Some("Hello, BYOND!".to_owned())
/// } }
/// ```
#[macro_export]
macro_rules! byond_ffi_fn {
    ($name:ident() $body:block) => {
        #[no_mangle]
        #[allow(clippy::missing_safety_doc)]
        extern "C" fn $name(
            _argc: ::std::os::raw::c_int, _argv: *const *const ::std::os::raw::c_char
        ) -> *const ::std::os::raw::c_char {
            let closure = || ($body);
            $crate::byond_ffi::byond_return($crate::guard(closure).unwrap_or(None).map(From::from))
        }
    };

    ($name:ident($($arg:ident),* $(, ...$rest:ident)?) $body:block) => {
        #[no_mangle]
        #[allow(clippy::missing_safety_doc)]
        extern "C" fn $name(
            _argc: ::std::os::raw::c_int, _argv: *const *const ::std::os::raw::c_char
        ) -> *const ::std::os::raw::c_char {
            let __args;
            unsafe {
                __args = $crate::byond_ffi::parse_args(_argc, _argv);
            }

            let mut __argn = 0;
            $(
                let $arg: &str = __args.get(__argn).map_or("", |cow| &*cow);
                __argn += 1;
            )*
            $(
                let $rest = __args.get(__argn..).unwrap_or(&[]);
            )?

            let closure = || ($body);
            $crate::byond_ffi::byond_return($crate::guard(closure).unwrap_or(None).map(From::from))
        }
    };
}
