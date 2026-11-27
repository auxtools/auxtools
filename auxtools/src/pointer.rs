use std::sync::OnceLock;

use crate::{
	raw_types::{
		funcs,
		values::{Value as RawValue, ValueData, ValueTag}
	},
	Value
};

fn found() -> bool {
	static FOUND: OnceLock<bool> = OnceLock::new();
	*FOUND.get_or_init(|| match crate::registry::find!("Byond_ReadPointer") {
		Ok(address) => {
			unsafe { funcs::read_pointer_byond = address as *const std::ffi::c_void };
			true
		}
		Err(_) => false
	})
}

/// Returns `true` if the running BYOND build exports `Byond_ReadPointer`.
///
/// The lookup result is cached for the lifetime of the process. When this
/// returns `false`, `read` and [`read_raw`] always return `None`.
pub fn readable() -> bool {
	found()
}

/// Reads the raw value targeted by a DM pointer (`&var`).
///
/// Returns `None` if `pointer` is not a pointer value, if the running BYOND
/// build lacks `Byond_ReadPointer`, or if BYOND fails the read (for example,
/// for a variable on a deleted object).
///
/// The returned `RawValue` holds no BYOND reference. Compare or convert it
/// immediately, or use `read` to obtain an owning `Value`.
pub fn read_raw(pointer: &Value) -> Option<RawValue> {
	if pointer.raw.tag != ValueTag::Pointer {
		return None;
	}
	if !found() {
		return None;
	}
	let mut result = RawValue {
		tag: ValueTag::Null,
		data: ValueData { id: 0 }
	};
	(unsafe { funcs::read_pointer(&mut result, &pointer.raw) } == 1).then_some(result)
}

pub fn read(pointer: &Value) -> Option<Value> {
	read_raw(pointer).map(|raw| unsafe { Value::from_raw(raw) })
}
