use super::values;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ListId(pub u32);

/// A 32-byte red-black tree node in a DM list's associative map or `/alist`.
///
/// BYOND reads the key at offset `+0` and the child pointers at `+16` and
/// `+20`.
#[repr(C)]
pub struct AssociativeListEntry {
	/// Entry key, holding no BYOND reference (the same value is also stored in
	/// a `vector_part` slot).
	pub key: values::Value,
	/// Associated value, holding a BYOND reference that is released when the
	/// tree is freed.
	pub value: values::Value,
	pub left: *mut AssociativeListEntry,
	pub right: *mut AssociativeListEntry,
	pub parent: *mut AssociativeListEntry,
	/// Red-black tree node color (`0` for red, `1` for black).
	pub color: u8
}

/// An 8-byte `/alist` record in BYOND's alist table (BYOND 516 and later).
///
/// Indexed by an `/alist` value's ID. A null `root` indicates an empty
/// `/alist`.
#[repr(C)]
pub struct Alist {
	pub root: *mut AssociativeListEntry,
	/// Reference count for this `/alist`, starting at `1`.
	pub refcount: u32
}

#[repr(C)]
pub struct List {
	pub vector_part: *mut values::Value,
	pub assoc_part: *mut AssociativeListEntry,
	pub allocated: u32,
	pub length: u32,
	pub refcount: u32,
	unknown: u32
}
