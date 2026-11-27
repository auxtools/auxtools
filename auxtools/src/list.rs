use std::iter::FromIterator;

use crate::*;

/// A handle to a DM list, holding one BYOND reference.
///
/// Not `Send`. `List::from_value` and `Value::as_list` accept any value that
/// BYOND's `islist()` accepts, including `/alist` on BYOND 516 and later, and
/// return `Err` for everything else.
///
/// `get` and `set` follow DM's `L[x]` indexing rules: numbers are 1-based
/// positions and other values are associative keys, except in an `/alist` where
/// numbers are keys as well. `get` and `set` return `Err` if BYOND raises a
/// runtime error, while `append`, `remove`, `len`, and `is_list` panic instead.
pub struct List {
	value: Value
}

impl List {
	pub fn from_value(val: &Value) -> DMResult<Self> {
		if !Self::is_list(val) {
			return Err(runtime!("cannot create List from a non-list value"));
		}

		Ok(Self { value: val.clone() })
	}

	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_size(capacity: u32) -> Self {
		let mut id: raw_types::lists::ListId = raw_types::lists::ListId(0);
		unsafe {
			assert_eq!(raw_types::funcs::create_list(&mut id, capacity), 1);
		}

		let raw = raw_types::values::Value {
			tag: raw_types::values::ValueTag::List,
			data: raw_types::values::ValueData { id: id.0 }
		};
		Self {
			value: unsafe { Value::from_raw_owned(raw) }
		}
	}

	pub fn get<I: Into<Value>>(&self, index: I) -> runtime::DMResult {
		let index = index.into();

		let mut value = raw_types::values::Value {
			tag: raw_types::values::ValueTag::Null,
			data: raw_types::values::ValueData { id: 0 }
		};

		unsafe {
			if raw_types::funcs::get_assoc_element(&mut value, self.value.raw, index.raw) == 1 {
				return Ok(Value::from_raw_owned(value));
			}

			Err(runtime!("could not read list entry (invalid list, index, or key)"))
		}
	}

	pub fn set<K: Into<Value>, V: Into<Value>>(&self, index: K, value: V) -> Result<(), runtime::Runtime> {
		let index = index.into();
		let value = value.into();

		unsafe {
			if raw_types::funcs::set_assoc_element(self.value.raw, index.raw, value.raw) == 1 {
				return Ok(());
			}

			Err(runtime!("could not write list entry (invalid list, index, or key)"))
		}
	}

	pub fn append<V: Into<Value>>(&self, value: V) {
		let value = value.into();

		unsafe {
			assert_eq!(raw_types::funcs::append_to_list(self.value.raw, value.raw), 1);
		}
	}

	pub fn remove<V: Into<Value>>(&self, value: V) {
		let value = value.into();

		unsafe {
			assert_eq!(raw_types::funcs::remove_from_list(self.value.raw, value.raw), 1);
		}
	}

	pub fn len(&self) -> u32 {
		let mut length: u32 = 0;
		unsafe {
			assert_eq!(raw_types::funcs::get_length(&mut length, self.value.raw), 1);
		}
		length
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn is_alist(&self) -> bool {
		self.value.raw.tag == raw_types::values::ValueTag::Alist
	}

	/// Returns all `(key, value)` pairs of an `/alist` in the same order as
	/// `for (var/k in A)`.
	///
	/// Each returned `Value` holds its own BYOND reference.
	///
	/// # Errors
	///
	/// Returns an error if this list is not an `/alist` (always the case before
	/// BYOND 516).
	pub fn alist_pairs(&self) -> DMResult<Vec<(Value, Value)>> {
		if !self.is_alist() {
			return Err(runtime!("cannot read alist pairs from a non-alist value"));
		}

		let id = unsafe { self.value.raw.data.id } as usize;
		let record = unsafe {
			if id >= *raw_types::funcs::ALIST_TABLE_COUNT as usize {
				return Err(runtime!("alist ID {} is outside the alist table", id));
			}
			*(*raw_types::funcs::ALIST_TABLE).add(id)
		};
		if record.is_null() {
			return Err(runtime!("alist ID {} has no record", id));
		}

		let mut pairs = vec![];
		let mut node = unsafe { (*record).root };
		let mut pending = vec![];
		while !node.is_null() || !pending.is_empty() {
			while !node.is_null() {
				pending.push(node);
				node = unsafe { (*node).left };
			}
			let entry = pending.pop().unwrap();
			unsafe {
				pairs.push((Value::from_raw((*entry).key), Value::from_raw((*entry).value)));
				node = (*entry).right;
			}
		}

		Ok(pairs)
	}

	/// Returns `true` if BYOND's `islist()` accepts `value`.
	///
	/// Queries BYOND directly because a `filters` list and a single filter
	/// share the same raw value tag. A pointer to a list returns `false`.
	///
	/// # Panics
	///
	/// Panics if the call into BYOND fails.
	pub fn is_list(value: &Value) -> bool {
		let mut is_list: u8 = 0;
		unsafe {
			assert_eq!(raw_types::funcs::value_is_list(&mut is_list, value.raw), 1);
		}
		is_list != 0
	}
}

impl FromIterator<Value> for List {
	fn from_iter<I: IntoIterator<Item = Value>>(it: I) -> Self {
		let res = Self::new();

		for val in it {
			res.append(val);
		}

		res
	}
}

impl From<List> for Value {
	fn from(list: List) -> Self {
		list.value
	}
}

impl From<&List> for Value {
	fn from(list: &List) -> Self {
		list.value.clone()
	}
}

impl Default for List {
	fn default() -> Self {
		Self::with_size(0)
	}
}
