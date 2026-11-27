use std::{
	collections::VecDeque,
	ffi::{c_void, CStr, CString},
	os::raw::c_char,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Mutex
	}
};

use auxtools::*;
use auxtools_protocol::guest;

type Formatter = unsafe extern "C" fn(*mut c_void, *const c_char, ...) -> usize;

static BYONDS_FORMATTER: AtomicUsize = AtomicUsize::new(0);
static KEYS: Mutex<Keys> = Mutex::new(Keys {
	given: Vec::new(),
	waiting: VecDeque::new()
});

const BYONDS_FORMAT: &[u8] = b"Guest-%";
const CALL: u8 = 0xE8;

struct Keys {
	given: Vec<CString>,
	waiting: VecDeque<CString>
}

pub fn give_next_guest(key: &str) -> Result<(), String> {
	if let Some(unusable) = guest::unusable(key) {
		return Err(unusable.to_string());
	}
	let format = CString::new(key.replace('%', "%%")).map_err(|e| e.to_string())?;
	let mut keys = KEYS.lock().map_err(|e| e.to_string())?;
	take_the_call()?;
	if !keys.given.contains(&format) {
		keys.given.push(format.clone());
	}
	keys.waiting.retain(|waiting| *waiting != format);
	keys.waiting.push_front(format);
	Ok(())
}

pub fn give_keys_again() {
	if let Ok(mut keys) = KEYS.lock() {
		keys.waiting = keys.given.iter().cloned().collect();
	}
}

#[init(partial)]
fn guests_of_a_new_world() -> Result<(), String> {
	if KEYS.lock().is_ok_and(|keys| !keys.given.is_empty()) {
		let _ = take_the_call();
	}
	Ok(())
}

#[shutdown]
fn guests_of_an_ending_world() {
	give_keys_again();
	give_the_call_back();
}

fn call_distance() -> Result<*mut i32, String> {
	let distance = registry::find!("guest_key_formatter_call")? as *mut i32;
	if unsafe { (distance as *const u8).sub(1).read() } != CALL {
		return Err("could not find BYOND's guest key call at the expected address".to_owned());
	}
	Ok(distance)
}

fn take_the_call() -> Result<(), String> {
	if BYONDS_FORMATTER.load(Ordering::Relaxed) != 0 {
		return Ok(());
	}
	let distance = call_distance()?;
	let returns_to = distance as usize + std::mem::size_of::<i32>();
	unsafe {
		let formatter = returns_to.wrapping_add_signed(distance.read_unaligned() as isize);
		let ours = (formats_guest_key as *const () as usize).wrapping_sub(returns_to) as i32;
		writing(distance as usize, std::mem::size_of::<i32>(), || distance.write_unaligned(ours))?;
		BYONDS_FORMATTER.store(formatter, Ordering::Relaxed);
	}
	Ok(())
}

fn give_the_call_back() {
	let formatter = BYONDS_FORMATTER.load(Ordering::Relaxed);
	let Some(distance) = (formatter != 0).then(call_distance).and_then(Result::ok) else {
		return;
	};
	let returns_to = distance as usize + std::mem::size_of::<i32>();
	let byonds = formatter.wrapping_sub(returns_to) as i32;
	if unsafe { writing(distance as usize, std::mem::size_of::<i32>(), || distance.write_unaligned(byonds)) }.is_ok() {
		BYONDS_FORMATTER.store(0, Ordering::Relaxed);
	}
}

unsafe extern "C" fn formats_guest_key(text: *mut c_void, format: *const c_char, number: u32) -> usize {
	let byonds: Formatter = unsafe { std::mem::transmute(BYONDS_FORMATTER.load(Ordering::Relaxed)) };
	let waiting = if unsafe { CStr::from_ptr(format) }.to_bytes().starts_with(BYONDS_FORMAT) {
		KEYS.lock().ok().and_then(|mut keys| keys.waiting.pop_front())
	} else {
		None
	};
	match waiting {
		Some(key) => unsafe { byonds(text, key.as_ptr(), number) },
		None => unsafe { byonds(text, format, number) }
	}
}

#[cfg(unix)]
unsafe fn writing(start: usize, length: usize, write: impl FnOnce()) -> Result<(), String> {
	extern "C" {
		fn mprotect(address: *mut c_void, length: usize, protection: i32) -> i32;
	}
	const PAGE: usize = 4096;
	const READ_EXECUTE: i32 = 5;
	const READ_WRITE_EXECUTE: i32 = 7;
	let from = start & !(PAGE - 1);
	let pages = ((start + length + PAGE - 1) & !(PAGE - 1)) - from;
	unsafe {
		if mprotect(from as *mut c_void, pages, READ_WRITE_EXECUTE) != 0 {
			return Err(format!("could not make BYOND's code writable: {}", std::io::Error::last_os_error()));
		}
		write();
		mprotect(from as *mut c_void, pages, READ_EXECUTE);
	}
	Ok(())
}

#[cfg(windows)]
unsafe fn writing(start: usize, length: usize, write: impl FnOnce()) -> Result<(), String> {
	let _writable = unsafe { region::protect_with_handle(start as *const u8, length, region::Protection::READ_WRITE_EXECUTE) }
		.map_err(|e| format!("could not make BYOND's code writable: {}", e))?;
	write();
	Ok(())
}
