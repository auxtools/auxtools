use std::cell::UnsafeCell;

use crate::{inventory, StringRef};

/// Evaluates a string literal to a cached `&'static StringRef`.
///
/// All `byond_string!` literals are interned on each library initialization and
/// dropped at shutdown. Call only on BYOND's thread because the backing storage
/// is unsynchronized. Do not keep the returned reference across a shutdown, or
/// it will dangle.
///
/// # Panics
///
/// Panics if evaluated before initialization or after shutdown.
#[macro_export]
macro_rules! byond_string {
	($s:literal) => {
		unsafe {
			static mut STORE: $crate::InternedString = $crate::InternedString($s, std::cell::UnsafeCell::new(None));
			$crate::inventory::submit!(unsafe { &*::std::ptr::addr_of!(STORE) });
			let x = &*STORE.1.get();
			x.as_ref().unwrap()
		}
	};
}

// hack
unsafe impl Sync for InternedString {}

#[doc(hidden)]
pub struct InternedString(pub &'static str, pub UnsafeCell<Option<StringRef>>);

inventory::collect!(&'static InternedString);

pub fn setup_interned_strings() {
	for info in inventory::iter::<&'static InternedString> {
		let string = StringRef::new(info.0).expect("failed to create interned string");

		unsafe {
			let dst = &mut *info.1.get();
			*dst = Some(string);
		}
	}
}

pub fn destroy_interned_strings() {
	for info in inventory::iter::<&'static InternedString> {
		unsafe {
			let dst = &mut *info.1.get();
			*dst = None;
		}
	}
}
