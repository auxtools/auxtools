use super::{raw_types, string};
use crate::{list, runtime, runtime::DMResult};
use std::{ffi::CString, fmt, marker::PhantomData};

/// A reference-counted DM value.
///
/// Holds one BYOND reference from creation until drop, and `Clone` acquires an
/// additional reference. Neither `Send` nor `Sync`. Kept `Value`s are not
/// released automatically at shutdown (and dropping one afterward still calls
/// into BYOND), so do not store a `Value` past the hook that created it. Use
/// `WeakValue` via `as_weak` for longer-lived references.
///
/// `PartialEq` and `Hash` compare raw tag and data bits (identity) rather than
/// DM's `==`. Modifying the public `raw` field directly breaks reference
/// counting. Methods taking a variable name (such as `get` and `set`) require a
/// `StringRef` or `byond_string!`, not `&str`.
///
/// Dropping it releases its reference, so its raw value can only be read, with
/// [`Value::raw`].
///
/// ```compile_fail
/// let mut value = auxtools::Value::NULL;
/// value.raw.tag = auxtools::raw_types::values::ValueTag::String;
/// ```
pub struct Value {
	pub(crate) raw: raw_types::values::Value,
	phantom: PhantomData<*mut ()>
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		unsafe { self.raw.tag == other.raw.tag && self.raw.data.id == other.raw.data.id }
	}
}

impl Eq for Value {}

impl std::hash::Hash for Value {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		unsafe {
			self.raw.tag.hash(state);
			self.raw.data.id.hash(state);
		}
	}
}

impl Drop for Value {
	fn drop(&mut self) {
		if self.holds_no_reference() {
			return;
		}
		unsafe {
			raw_types::funcs::dec_ref_count(self.raw);
		}
	}
}

impl Value {
	pub const NULL: Self = Self {
		raw: raw_types::values::Value {
			tag: raw_types::values::ValueTag::Null,
			data: raw_types::values::ValueData { number: 0.0 }
		},
		phantom: PhantomData {}
	};

	const fn holds_no_reference(&self) -> bool {
		matches!(
			self.raw.tag,
			raw_types::values::ValueTag::Null | raw_types::values::ValueTag::Number | raw_types::values::ValueTag::World
		)
	}

	const fn world_value(id: u32) -> Self {
		Self {
			raw: raw_types::values::Value {
				tag: raw_types::values::ValueTag::World,
				data: raw_types::values::ValueData { id }
			},
			phantom: PhantomData {}
		}
	}

	#[track_caller]
	pub fn world() -> Self {
		crate::expect_live("Value::world");
		Self::world_value(0)
	}

	#[track_caller]
	pub fn globals() -> Self {
		crate::expect_live("Value::globals");
		Self::world_value(1)
	}

	/// Creates a `Value` from a raw `tag` and `data`, taking a new BYOND
	/// reference.
	///
	/// # Safety
	///
	/// Must be called on BYOND's thread, and `(tag, data)` must identify a
	/// value that currently exists in BYOND.
	pub unsafe fn new(tag: raw_types::values::ValueTag, data: raw_types::values::ValueData) -> Value {
		let value = Value {
			raw: raw_types::values::Value { tag, data },
			phantom: PhantomData {}
		};
		if !value.holds_no_reference() {
			raw_types::funcs::inc_ref_count(value.raw);
		}
		value
	}

	#[deprecated(note = "use `Value::NULL` instead")]
	pub const fn null() -> Value {
		Self::NULL
	}

	/// Creates a turf `Value` for `id` without bounds checking or incrementing
	/// its reference count.
	///
	/// Use `turf_by_id` for a bounds-checked alternative. Dropping the returned
	/// `Value` still releases a BYOND reference.
	///
	/// # Safety
	///
	/// `id` must be less than `world.maxx * world.maxy * world.maxz`.
	pub const unsafe fn turf_by_id_unchecked(id: u32) -> Value {
		Value {
			raw: raw_types::values::Value {
				tag: raw_types::values::ValueTag::Turf,
				data: raw_types::values::ValueData { id }
			},
			phantom: PhantomData {}
		}
	}

	pub fn turf_by_id(id: u32) -> DMResult {
		crate::check_live("Value::turf_by_id")?;
		let world = Self::world();
		let max_x = world.get_number(crate::byond_string!("maxx"))? as u32;
		let max_y = world.get_number(crate::byond_string!("maxy"))? as u32;
		let max_z = world.get_number(crate::byond_string!("maxz"))? as u32;
		if (0..max_x * max_y * max_z).contains(&(id - 1)) {
			Ok(unsafe { Value::turf_by_id_unchecked(id) })
		} else {
			Err(runtime!("cannot get turf with invalid ID {}", id))
		}
	}

	/// Returns the turf at 1-based DM coordinates `(x, y, z)`.
	///
	/// Reads `world.maxx`, `world.maxy`, and `world.maxz` on each call.
	///
	/// # Errors
	///
	/// Returns an error if any coordinate is outside the current map bounds. In
	/// release builds, passing `0` wraps around and returns this out-of-bounds
	/// error.
	///
	/// # Panics
	///
	/// In debug builds, passing `0` for any coordinate panics due to unsigned
	/// underflow.
	pub fn turf(x: u32, y: u32, z: u32) -> DMResult {
		crate::check_live("Value::turf")?;
		let world = Self::world();
		let max_x = world.get_number(crate::byond_string!("maxx"))? as u32;
		let max_y = world.get_number(crate::byond_string!("maxy"))? as u32;
		let max_z = world.get_number(crate::byond_string!("maxz"))? as u32;
		let x = x - 1;
		let y = y - 1;
		let z = z - 1;
		if (0..max_x).contains(&x) && (0..max_y).contains(&y) && (0..max_z).contains(&z) {
			Ok(unsafe { Value::turf_by_id_unchecked(x + y * max_x + z * max_x * max_y) })
		} else {
			Err(runtime!("turf coordinates ({}, {}, {}) are out of range", x + 1, y + 1, z + 1))
		}
	}

	fn get_by_id(&self, name_id: raw_types::strings::StringId) -> DMResult {
		let mut val = raw_types::values::Value {
			tag: raw_types::values::ValueTag::Null,
			data: raw_types::values::ValueData { id: 0 }
		};

		unsafe {
			if raw_types::funcs::get_variable(&mut val, self.raw, name_id) != 1 {
				let varname: String = string::StringRef::from_id(name_id).into();
				return Err(runtime!("could not read {}.{}", &self, varname));
			}

			Ok(Self::from_raw(val))
		}
	}

	fn set_by_id(&self, name_id: raw_types::strings::StringId, new_value: raw_types::values::Value) -> Result<(), runtime::Runtime> {
		unsafe {
			if raw_types::funcs::set_variable(self.raw, name_id, new_value) != 1 {
				let varname: String = string::StringRef::from_id(name_id).into();
				return Err(runtime!("could not write to {}.{}", self, varname));
			}
		}
		Ok(())
	}

	pub fn get<S: Into<string::StringRef>>(&self, name: S) -> DMResult {
		let name = name.into();
		self.get_by_id(name.get_id())
	}

	pub fn get_number<S: Into<string::StringRef>>(&self, name: S) -> DMResult<f32> {
		self.get(name)?.as_number()
	}

	pub fn get_string<S: Into<string::StringRef>>(&self, name: S) -> DMResult<String> {
		self.get(name)?.as_string()
	}

	pub fn get_list<S: Into<string::StringRef>>(&self, name: S) -> DMResult<list::List> {
		let var = self.get(name)?;
		var.as_list()
	}

	pub fn set<S: Into<string::StringRef>, V: Into<Value>>(&self, name: S, value: V) -> DMResult<()> {
		let value = value.into();

		self.set_by_id(name.into().get_id(), value.raw)?;
		Ok(())
	}

	pub fn as_number(&self) -> DMResult<f32> {
		match self.raw.tag {
			raw_types::values::ValueTag::Number => unsafe { Ok(self.raw.data.number) },
			_ => Err(runtime!("cannot interpret non-number value as a number"))
		}
	}

	pub fn as_string(&self) -> DMResult<String> {
		match self.raw.tag {
			raw_types::values::ValueTag::String => unsafe { Ok(string::StringRef::from_id(self.raw.data.string).into()) },
			_ => Err(runtime!("cannot interpret non-string value as a String"))
		}
	}

	pub fn as_list(&self) -> DMResult<list::List> {
		list::List::from_value(self)
	}

	/// Synchronously calls the proc `procname` on this value on BYOND's thread,
	/// with `null` for `usr`.
	///
	/// Underscores in `procname` are replaced with spaces before BYOND looks up
	/// the proc. Takes a temporary reference to each argument so the caller's
	/// `Value`s stay valid, and returns a `Value` owning the reference returned
	/// by BYOND.
	///
	/// # Errors
	///
	/// Returns an error if `procname` contains a NUL byte or if BYOND fails to
	/// invoke the proc (for example, an unknown proc name or a `null`
	/// receiver).
	pub fn call<S: AsRef<str>>(&self, procname: S, args: &[&Value]) -> DMResult {
		let mut ret = raw_types::values::Value {
			tag: raw_types::values::ValueTag::Null,
			data: raw_types::values::ValueData { id: 0 }
		};

		let name_ref = string::StringRef::new(&procname.as_ref().replace("_", " "))?;
		let mut raw_args: Vec<_> = args.iter().map(|e| e.raw).collect();

		unsafe {
			for v in args {
				raw_types::funcs::inc_ref_count(v.raw);
			}

			if raw_types::funcs::call_datum_proc_by_name(
				&mut ret,
				Value::NULL.raw,
				2,
				name_ref.get_id(),
				self.raw,
				raw_args.as_mut_ptr(),
				raw_args.len(),
				0,
				0
			) == 1
			{
				return Ok(Value::from_raw_owned(ret));
			}
		}

		Err(runtime!("proc call failed"))
	}

	pub fn to_dmstring(&self) -> DMResult<string::StringRef> {
		match self.raw.tag {
			raw_types::values::ValueTag::Null | raw_types::values::ValueTag::Number | raw_types::values::ValueTag::String => {
				return string::StringRef::new(format!("{}", self.raw).as_str())
			}

			_ => {}
		}

		let mut id = raw_types::strings::StringId(0);

		unsafe {
			if raw_types::funcs::to_string(&mut id, self.raw) != 1 {
				return Err(runtime!("to_string failed on {:?}", self));
			}
			Ok(string::StringRef::from_id(id))
		}
	}

	/// Formats this value as a Rust `String` (inherent method shadowing
	/// `Display::to_string`).
	///
	/// Formats `null` as `"null"`, numbers with Rust's default formatting,
	/// strings as quoted Rust-escaped literals, and all other values using
	/// BYOND's text conversion. Use `as_string` for an unquoted string value,
	/// or `to_dmstring` to get the result as a `StringRef`.
	pub fn to_string(&self) -> DMResult<String> {
		match self.raw.tag {
			raw_types::values::ValueTag::Null | raw_types::values::ValueTag::Number | raw_types::values::ValueTag::String => {
				return Ok(format!("{}", self.raw))
			}

			_ => {}
		}

		let mut id = raw_types::strings::StringId(0);

		unsafe {
			if raw_types::funcs::to_string(&mut id, self.raw) != 1 {
				return Err(runtime!("to_string failed on {:?}", self));
			}
			Ok(String::from(string::StringRef::from_id(id)))
		}
	}

	pub fn get_type(&self) -> Result<String, runtime::Runtime> {
		self.get(crate::byond_string!("type"))?.to_string()
	}

	pub fn is_exact_type<S: AsRef<str>>(&self, typepath: S) -> bool {
		match self.get_type() {
			Err(_) => false,
			Ok(my_type) => my_type == typepath.as_ref()
		}
	}

	/// Returns `false` for `null` and the number `0`, and `true` for all other
	/// values.
	///
	/// Unlike DM's truthiness rules, an empty string returns `true`.
	pub fn is_truthy(&self) -> bool {
		match self.raw.tag {
			raw_types::values::ValueTag::Null => false,
			raw_types::values::ValueTag::Number => unsafe { self.raw.data.number != 0.0 },

			_ => true
		}
	}

	pub fn from_string<S: AsRef<str>>(data: S) -> DMResult {
		Self::from_string_raw(data.as_ref().as_bytes())
	}

	pub fn from_string_raw(data: &[u8]) -> DMResult {
		crate::check_live("Value::from_string")?;
		let string = CString::new(data).map_err(|_| runtime!("cannot create string containing a NUL byte"))?;

		unsafe {
			let mut id = raw_types::strings::StringId(0);

			assert_eq!(raw_types::funcs::get_string_id(&mut id, string.as_ptr()), 1);

			Ok(Value::new(raw_types::values::ValueTag::String, raw_types::values::ValueData {
				string: id
			}))
		}
	}

	/// Wraps a raw BYOND value in a `Value` and increments its reference count.
	///
	/// Use this when borrowing a raw value owned by something else, such as a
	/// stack frame variable or a table slot. Dropping the returned `Value`
	/// releases the added reference.
	///
	/// # Safety
	///
	/// Must be called on BYOND's thread, and `v` must identify a value that
	/// currently exists in BYOND.
	pub unsafe fn from_raw(v: raw_types::values::Value) -> Self {
		Value::new(v.tag, v.data)
	}

	/// Wraps a raw BYOND value in a `Value` by taking ownership of the caller's
	/// existing reference.
	///
	/// Does not increment BYOND's reference count, and dropping the returned
	/// `Value` releases one reference.
	///
	/// # Safety
	///
	/// The caller must own one BYOND reference to `v` to transfer (for example,
	/// a return value from a BYOND call). Calling this on a borrowed raw value
	/// releases a reference still relied on elsewhere.
	pub const fn raw(&self) -> raw_types::values::Value {
		self.raw
	}

	pub const unsafe fn from_raw_owned(v: raw_types::values::Value) -> Value {
		Value {
			raw: v,
			phantom: PhantomData {}
		}
	}
}

impl Clone for Value {
	fn clone(&self) -> Value {
		unsafe { Value::from_raw(self.raw) }
	}
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.raw)
	}
}

impl fmt::Debug for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.raw)
	}
}
