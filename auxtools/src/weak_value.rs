use crate::{byond_string, raw_types, DMResult, Value};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

fn get_next_id() -> f32 {
	static NEXT_WEAKREF_ID: AtomicU32 = AtomicU32::new(1);
	let id = NEXT_WEAKREF_ID.fetch_add(1, Relaxed);
	id as f32
}

/// A non-owning weak reference to a DM datum.
///
/// `Copy`, `Send`, and `Sync`, so it can be stored in statics or sent across
/// threads without keeping the datum alive. Create one with `Value::as_weak`
/// and call [`WeakValue::upgrade`] only on BYOND's thread.
///
/// Target datum types must declare `var/__auxtools_weakref_id` in DM so
/// auxtools can tag instances with a unique text ID (reused across multiple
/// auxtools libraries in the same world).
#[derive(Copy, Clone)]
pub struct WeakValue {
	inner: raw_types::values::Value,
	id: f32
}

impl WeakValue {
	pub(crate) fn new(val: &Value) -> DMResult<Self> {
		if let Ok(id) = val.get_number(byond_string!("__auxtools_weakref_id")) {
			return Ok(Self { inner: val.raw, id });
		}

		let id = get_next_id();
		val.set(byond_string!("__auxtools_weakref_id"), Value::from(id))?;
		Ok(Self { inner: val.raw, id })
	}

	/// Returns `None` if the datum was deleted. Panics on any thread other than
	/// BYOND's.
	pub fn upgrade(&self) -> Option<Value> {
		assert!(crate::on_byond_thread(), "WeakValue::upgrade must run on BYOND's thread");
		let real_val = unsafe { Value::from_raw(self.inner) };
		let id = real_val.get_number(byond_string!("__auxtools_weakref_id")).ok()?;

		if self.id != id {
			return None;
		}

		Some(real_val)
	}

	pub fn upgrade_or_null(&self) -> Value {
		match self.upgrade() {
			Some(v) => v,
			None => Value::NULL
		}
	}
}

impl Value {
	/// Creates a `WeakValue` referencing this datum.
	///
	/// Reads the datum's `__auxtools_weakref_id` variable and assigns a new ID
	/// if it does not already hold one.
	///
	/// # Errors
	///
	/// Returns an error if this value does not have an `__auxtools_weakref_id`
	/// variable.
	pub fn as_weak(&self) -> DMResult<WeakValue> {
		WeakValue::new(self)
	}
}
