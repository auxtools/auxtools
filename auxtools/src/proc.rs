use std::{
	cell::RefCell,
	collections::{hash_map::Entry, HashMap},
	fmt,
	marker::PhantomData
};

use ahash::RandomState;
use fxhash::FxHashMap;

use crate::*;

/// A handle to a single DM proc definition, holding its BYOND proc ID (`id`)
/// and normalized path (`path`, with `/proc/` and `/verb/` removed).
///
/// Proc IDs are valid only within the world they came from. Lookup tables are
/// thread-local and cleared on shutdown, so a `Proc` kept across a reboot
/// refers to whatever proc holds that ID in the new world. Both fields are
/// public and unchecked.
///
/// Neither `Send` nor `Sync`, because calling a proc from another thread would
/// execute DM code concurrently with BYOND's tick.
///
/// ```compile_fail
/// fn assert_send<T: Send>() {}
/// assert_send::<auxtools::Proc>();
/// ```
#[derive(Clone)]
pub struct Proc {
	pub id: raw_types::procs::ProcId,
	pub path: String,
	not_send: PhantomData<*mut ()>
}

fn entry_by_id(id: raw_types::procs::ProcId) -> *mut raw_types::procs::ProcEntry {
	let mut entry: *mut raw_types::procs::ProcEntry = std::ptr::null_mut();
	unsafe {
		assert_eq!(raw_types::funcs::get_proc_array_entry(&mut entry, id), 1);
	}
	entry
}

impl Proc {
	/// Override ID (`u32::MAX`) assigned to a proc set apart with
	/// [`Proc::set_apart`].
	pub const APART: u32 = u32::MAX;

	pub fn overrides<S: Into<String>>(path: S) -> Vec<Self> {
		let path = strip_path(path.into());
		PROCS_BY_NAME.with(|h| h.borrow().get(&path).cloned().unwrap_or_default())
	}

	/// Replaces the library's list of proc definitions for `path`, assigning
	/// each entry an override ID equal to its position in `procs`.
	///
	/// Updates only the library's thread-local lookup table (cleared at
	/// shutdown), not BYOND's internal tables. Definitions omitted from `procs`
	/// are no longer returned by [`Proc::find_override`], although their
	/// `override_id` method continues to report their previous ID.
	pub fn set_overrides<S: Into<String>>(path: S, procs: Vec<Self>) {
		let path = strip_path(path.into());
		PROC_OVERRIDE_IDS.with(|override_ids| {
			let mut override_ids = override_ids.borrow_mut();
			for (i, proc) in procs.iter().enumerate() {
				override_ids.insert(proc.id, i as u32);
			}
		});
		PROCS_BY_NAME.with(|h| h.borrow_mut().insert(path, procs));
	}

	/// Returns a pointer to this proc's entry in BYOND's proc table, or null if
	/// the ID does not exist.
	///
	/// The pointer is valid only until DM code runs again or the proc table
	/// grows, because BYOND may reallocate the table in memory.
	pub fn entry(&self) -> *mut raw_types::procs::ProcEntry {
		entry_by_id(self.id)
	}

	/// Finds the first definition (override ID 0) of the proc at `path`.
	///
	/// `path` may be written with or without `/proc/` and `/verb/`. Use
	/// [`Proc::find_override`] to look up subsequent definitions. Returns
	/// `None` if no matching proc exists, or if called before initialization,
	/// after shutdown, or off BYOND's thread.
	pub fn find<S: Into<String>>(path: S) -> Option<Self> {
		get_proc(path)
	}

	/// Finds the definition of `path` with the given `override_id`.
	///
	/// Override ID `0` matches [`Proc::find`]. Passing [`Proc::APART`] returns
	/// the proc most recently set apart for `path`.
	pub fn find_override<S: Into<String>>(path: S, override_id: u32) -> Option<Self> {
		get_proc_override(path, override_id)
	}

	pub fn from_id(id: raw_types::procs::ProcId) -> Option<Self> {
		let proc_entry = entry_by_id(id);
		if proc_entry.is_null() {
			return None;
		}
		let path = strip_path(unsafe { StringRef::from_id((*proc_entry).path).into() });
		Some(Proc {
			id,
			path,
			not_send: PhantomData
		})
	}

	/// Returns the source file named by this proc's first instruction, if any.
	///
	/// Returns `None` if the bytecode does not start with a file record (opcode
	/// `0x84`) or if that record names no file.
	///
	/// # Safety
	///
	/// Reads the proc's bytecode via [`Proc::bytecode`] and requires the same
	/// safety invariants.
	pub unsafe fn file_name(&self) -> Option<StringRef> {
		let bytecode = self.bytecode();
		if bytecode.len() < 2 || bytecode[0] != 0x84 {
			return None;
		}

		let file_id = raw_types::strings::StringId(bytecode[0x01]);
		if !file_id.valid() {
			return None;
		}

		Some(StringRef::from_id(file_id))
	}

	/// Returns this proc's parameter names in declaration order.
	///
	/// Call on BYOND's thread for a proc ID valid in the current world. Each
	/// returned `StringRef` holds a BYOND reference. Returns an empty `Vec` if
	/// the proc has no parameter table (for example, a type's initializer
	/// proc).
	pub fn parameter_names(&self) -> Vec<StringRef> {
		use raw_types::misc::AsMiscId;
		unsafe {
			let parameters = (*self.entry()).metadata.parameters;
			if parameters.as_misc_id().raw() == runtime_tables::NONE {
				return vec![];
			}
			let (data, count) = raw_types::misc::get_parameters(parameters);
			(0..count).map(|i| StringRef::from_variable_id((*data.add(i)).name)).collect()
		}
	}

	pub fn local_names(&self) -> Vec<StringRef> {
		unsafe {
			let (names, count) = raw_types::misc::get_locals((*self.entry()).metadata.locals);
			(0..count).map(|i| StringRef::from_variable_id(*names.add(i))).collect()
		}
	}

	/// Replaces this proc's bytecode for subsequent calls.
	///
	/// Allocates a new bytecode entry for the proc unless it already has an
	/// unshared entry created by this library. Active frames continue running
	/// the previous bytecode. The library keeps the replacement buffer alive
	/// until shutdown, when unshared entries are restored to their original
	/// bytecode. Call on BYOND's thread while the library is initialized.
	///
	/// # Errors
	///
	/// Returns an error without modifying the proc if `bytecode` exceeds 65,535
	/// words or if BYOND cannot allocate a new bytecode entry (including on
	/// builds where `runtime_tables::available` fails).
	///
	/// # Panics
	///
	/// Panics if called before the library is initialized or after shutdown.
	pub fn set_bytecode(&self, bytecode: Vec<u32>) -> Result<(), String> {
		crate::bytecode_manager::set_bytecode(self, bytecode)
	}

	/// Registers all procs from ID `first` to the end of BYOND's proc table in
	/// the library's lookup table (`find`, `overrides`).
	///
	/// Intended for procs added after initialization with
	/// `runtime_tables::add_proc`. Each proc is appended to its path's override
	/// list, so registering the same ID more than once duplicates the entry and
	/// shifts subsequent override IDs.
	pub fn add_to_table(first: raw_types::procs::ProcId) {
		let mut id = first.0;
		while let Some(proc) = Proc::from_id(raw_types::procs::ProcId(id)) {
			register_proc(proc);
			id += 1;
		}
	}

	pub fn locals_count(&self) -> usize {
		unsafe { raw_types::misc::get_locals((*self.entry()).metadata.locals).1 }
	}

	/// Returns a mutable pointer to this proc's current bytecode buffer and its
	/// length in 32-bit words.
	///
	/// # Safety
	///
	/// Must be called on BYOND's thread for a proc ID valid in the current
	/// world. Other procs may share the same bytecode buffer, so writing
	/// through the pointer affects them unless `own_code` is called first. The
	/// pointer remains valid only until the proc's bytecode is replaced or the
	/// world shuts down.
	pub unsafe fn bytecode_mut_ptr(&self) -> (*mut u32, u16) {
		raw_types::misc::get_bytecode((*self.entry()).metadata.bytecode)
	}

	/// Returns this proc's current bytecode as a slice of at most 65,535 words.
	///
	/// # Safety
	///
	/// Requires the same invariants as [`Proc::bytecode_mut_ptr`]. In addition,
	/// the returned slice's lifetime is not tied to the underlying buffer and
	/// must not be held across [`Proc::set_bytecode`], DM execution, or
	/// shutdown.
	pub unsafe fn bytecode(&self) -> &[u32] {
		let (ptr, count) = self.bytecode_mut_ptr();
		std::slice::from_raw_parts(ptr, count as usize)
	}

	/// Synchronously calls this proc on BYOND's thread with `null` for `src`
	/// and `usr`.
	///
	/// Takes a temporary reference to each argument so the caller's `Value`s
	/// remain valid, and returns a `Value` owning the reference returned by
	/// BYOND. If this library has hooked the proc, the hook executes in place
	/// of the DM code.
	///
	/// Runtime errors inside the proc follow BYOND's normal error handling
	/// while this method still returns `Ok`. Because this runs DM code,
	/// pointers into BYOND tables (such as [`Proc::entry`]) are invalidated.
	pub fn call(&self, args: &[&Value]) -> runtime::DMResult {
		let mut ret = raw_types::values::Value {
			tag: raw_types::values::ValueTag::Null,
			data: raw_types::values::ValueData { id: 0 }
		};

		unsafe {
			for v in args {
				raw_types::funcs::inc_ref_count(v.raw);
			}

			let args: Vec<_> = args.iter().map(|e| e.raw).collect();

			if raw_types::funcs::call_proc_by_id(&mut ret, Value::NULL.raw, 2, self.id, 0, Value::NULL.raw, args.as_ptr(), args.len(), 0, 0) == 1 {
				return Ok(Value::from_raw_owned(ret));
			}
		}

		Err(runtime!("proc call failed"))
	}

	/// Moves this proc out of its path's override list and assigns it the
	/// override ID [`Proc::APART`].
	///
	/// Updates only the library's lookup table without modifying BYOND's
	/// tables. The proc keeps its BYOND proc ID, subsequent overrides shift
	/// down by one override ID, and [`Proc::find`], [`Proc::find_override`]
	/// (for numeric IDs), and `Proc::overrides` no longer return it. Only the
	/// most recently set-apart proc for a given path can be retrieved with
	/// [`Proc::APART`].
	pub fn set_apart(&self) {
		PROC_OVERRIDE_IDS.with(|override_ids| {
			let mut override_ids = override_ids.borrow_mut();
			PROCS_BY_NAME.with(|h| {
				if let Some(procs) = h.borrow_mut().get_mut(&self.path) {
					procs.retain(|proc| proc.id != self.id);
					for (i, proc) in procs.iter().enumerate() {
						override_ids.insert(proc.id, i as u32);
					}
				}
			});
			override_ids.insert(self.id, Self::APART);
		});
		PROCS_APART.with(|apart| apart.borrow_mut().insert(self.path.clone(), self.clone()));
	}

	pub fn override_id(&self) -> u32 {
		PROC_OVERRIDE_IDS.with(|override_ids| match override_ids.borrow().get(&self.id) {
			Some(id) => *id,
			None => 0
		})
	}
}

impl fmt::Debug for Proc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let path = unsafe { (*self.entry()).path };
		write!(f, "Proc({:?})", unsafe { StringRef::from_id(path) })
	}
}

thread_local!(static PROCS_BY_NAME: RefCell<HashMap<String, Vec<Proc>, RandomState>> = RefCell::new(HashMap::with_hasher(RandomState::default())));
thread_local!(static PROC_OVERRIDE_IDS: RefCell<FxHashMap<raw_types::procs::ProcId, u32>> = RefCell::new(FxHashMap::default()));
thread_local!(static PROCS_APART: RefCell<HashMap<String, Proc, RandomState>> = RefCell::new(HashMap::with_hasher(RandomState::default())));

fn strip_path(p: String) -> String {
	p.replace("/proc/", "/").replace("/verb/", "/")
}

pub fn populate_procs() {
	let mut i: u32 = 0;
	loop {
		let proc = Proc::from_id(raw_types::procs::ProcId(i));
		if proc.is_none() {
			break;
		}
		register_proc(proc.unwrap());

		i += 1;
	}
}

fn register_proc(proc: Proc) {
	PROC_OVERRIDE_IDS.with(|override_ids| {
		let mut override_ids = override_ids.borrow_mut();

		PROCS_BY_NAME.with(|h| {
			match h.borrow_mut().entry(proc.path.clone()) {
				Entry::Occupied(mut o) => {
					let vec = o.get_mut();
					override_ids.insert(proc.id, vec.len() as u32);
					vec.push(proc);
				}
				Entry::Vacant(v) => {
					override_ids.insert(proc.id, 0);
					v.insert(vec![proc]);
				}
			};
		});
	});
}

pub fn clear_procs() {
	PROCS_BY_NAME.with(|h| h.borrow_mut().clear());
	PROC_OVERRIDE_IDS.with(|override_ids| override_ids.borrow_mut().clear());
	PROCS_APART.with(|apart| apart.borrow_mut().clear());
}

pub fn get_proc_override<S: Into<String>>(path: S, override_id: u32) -> Option<Proc> {
	let s = strip_path(path.into());
	if override_id == Proc::APART {
		return PROCS_APART.with(|apart| apart.borrow().get(&s).cloned());
	}
	let found = PROCS_BY_NAME.with(|h| h.borrow().get(&s)?.get(override_id as usize).cloned());
	if found.is_none() && s.ends_with("/(init)") && adopt_named(&s) {
		return PROCS_BY_NAME.with(|h| h.borrow().get(&s)?.get(override_id as usize).cloned());
	}
	found
}

fn adopt_named(path: &str) -> bool {
	if PROCS_BY_NAME.with(|h| h.borrow().contains_key(path)) {
		return false;
	}
	let unnamed = PROCS_BY_NAME.with(|h| h.borrow().get("").cloned().unwrap_or_default());
	let named: Vec<Proc> = unnamed
		.iter()
		.filter_map(|proc| Proc::from_id(proc.id))
		.filter(|proc| proc.path == path)
		.collect();
	let any = !named.is_empty();
	for proc in named {
		register_proc(proc);
	}
	any
}

pub fn get_proc<S: Into<String>>(path: S) -> Option<Proc> {
	get_proc_override(path, 0)
}
