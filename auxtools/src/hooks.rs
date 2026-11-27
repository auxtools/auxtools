use std::{
	cell::RefCell,
	collections::HashMap,
	ffi::{c_void, CStr},
	hash::BuildHasherDefault,
	os::raw::c_char
};

use fxhash::FxHashMap;
use retour::RawDetour;

use super::{proc::Proc, raw_types, value::Value};
use crate::runtime::DMResult;

#[doc(hidden)]
pub struct CompileTimeHook {
	pub proc_path: &'static str,
	pub hook: ProcHook
}

inventory::collect!(CompileTimeHook);

#[doc(hidden)]
pub struct RuntimeErrorHook(pub fn(&str));
inventory::collect!(RuntimeErrorHook);

extern "C" {
	static mut call_proc_by_id_original: *const c_void;

	static mut runtime_original: *const c_void;
	fn runtime_hook(error: *const c_char);

	static runtime_intercepted: bool;

	fn call_proc_by_id_hook_trampoline(
		usr: raw_types::values::Value,
		proc_type: u32,
		proc_id: raw_types::procs::ProcId,
		override_depth: u32,
		src: raw_types::values::Value,
		args: *mut raw_types::values::Value,
		args_count_l: usize,
		callback: u32,
		callback_value: u32
	) -> raw_types::values::Value;
}

struct Detours {
	pub runtime_detour: Option<RawDetour>,
	pub call_proc_detour: Option<RawDetour>
}

impl Detours {
	pub const fn new() -> Self {
		Self {
			runtime_detour: None,
			call_proc_detour: None
		}
	}
}

thread_local!(static DETOURS: RefCell<Detours> = const { RefCell::new(Detours::new()) });

/// Reasons installing a proc hook can fail.
///
/// Only `ProcNotFound` and `AlreadyHooked` are returned. `NotInitialized` and
/// `UnknownFailure` are never produced. The `Debug` output formats each variant
/// as a human-readable message used in initialization errors.
pub enum HookFailure {
	NotInitialized,
	ProcNotFound,
	AlreadyHooked,
	UnknownFailure
}

impl std::fmt::Debug for HookFailure {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotInitialized => write!(f, "library not initialized"),
			Self::ProcNotFound => write!(f, "proc not found"),
			Self::AlreadyHooked => write!(f, "proc is already hooked"),
			Self::UnknownFailure => write!(f, "unknown failure")
		}
	}
}

/// Installs the library's detours on BYOND's runtime-error and proc-call
/// functions.
///
/// Called internally during the library's first initialization. Library authors
/// should not call this directly.
///
/// # Panics
///
/// Panics if either detour cannot be created or enabled.
pub(crate) fn init() -> Result<(), String> {
	unsafe {
		let runtime_hook = RawDetour::new(raw_types::funcs::runtime_byond as *const (), runtime_hook as *const ()).unwrap();

		runtime_hook.enable().unwrap();
		runtime_original = runtime_hook.trampoline() as *const () as *const c_void;

		let call_hook = RawDetour::new(raw_types::funcs::call_proc_by_id_byond as *const (), call_proc_by_id_entry as *const ()).unwrap();

		call_hook.enable().unwrap();
		call_proc_by_id_original = call_hook.trampoline() as *const () as *const c_void;

		DETOURS.with(|detours_cell| {
			let mut detours = detours_cell.borrow_mut();
			detours.runtime_detour = Some(runtime_hook);
			detours.call_proc_detour = Some(call_hook);
		});
	}
	Ok(())
}

/// Disables the detours installed by [`init`].
///
/// Called internally by `auxtools_full_shutdown`. Library authors should not
/// call this directly.
///
/// # Panics
///
/// Panics if called on a thread other than the one that ran [`init`].
pub(crate) fn shutdown() {
	unsafe {
		DETOURS.with(|detours_cell| {
			let detours = detours_cell.borrow();
			let runtime_hook = detours.runtime_detour.as_ref().unwrap();
			let call_proc_hook = detours.call_proc_detour.as_ref().unwrap();
			runtime_hook.disable().unwrap();
			call_proc_hook.disable().unwrap();
		});
	}
}

/// Function signature for a DM proc hook, receiving `src`, `usr`, and the call
/// arguments.
///
/// While a proc is hooked, every call invokes the hook on BYOND's thread in
/// place of the proc's DM bytecode. Returning `Ok(value)` sets the proc's
/// return value. Returning `Err` or panicking (caught and formatted as
/// `panicked: MESSAGE`) returns `null` to DM and passes
/// `MESSAGE HookPath: PATH` to `/proc/auxtools_stack_trace` if the world
/// defines it, without raising a BYOND runtime error.
///
/// `args` contains only the arguments passed by the caller (whereas the
/// `#[hook]` attribute fills missing parameters with `null`). Invoking the
/// hooked proc from inside its own hook re-enters the hook.
pub type ProcHook = fn(&Value, &Value, Vec<Value>) -> DMResult;

struct GameThreadOnly<T>(T);

unsafe impl<T> Sync for GameThreadOnly<T> {}

static PROC_HOOKS: GameThreadOnly<RefCell<FxHashMap<raw_types::procs::ProcId, (ProcHook, String)>>> =
	GameThreadOnly(RefCell::new(HashMap::with_hasher(BuildHasherDefault::new())));

static mut HOOKED_LOW_WORDS: [u8; 0x10000] = [0; 0x10000];

#[cfg(unix)]
const PROC_ID_ARGUMENT: usize = 0x14;
#[cfg(windows)]
const PROC_ID_ARGUMENT: usize = 0x10;

#[cfg(unix)]
#[unsafe(naked)]
unsafe extern "C" fn call_proc_by_id_entry() {
	core::arch::naked_asm!(
		"cmp byte ptr [{work}], 0",
		"jne 2f",
		"cmp byte ptr [{load}], 0",
		"jne 2f",
		"cmp byte ptr [{intercepted}], 0",
		"jne 2f",
		"movzx eax, word ptr [esp + {proc_id}]",
		"cmp byte ptr [{hooked} + eax], 0",
		"jne 2f",
		"inc dword ptr [{calls}]",
		"push ebp",
		"push ebx",
		"push esi",
		"push edi",
		"sub esp, 12",
		".rept 12",
		"push dword ptr [esp + 76]",
		".endr",
		"call dword ptr [{original}]",
		"add esp, 56",
		"pop edi",
		"pop esi",
		"pop ebx",
		"pop ebp",
		"ret 4",
		"2:",
		"jmp {trampoline}",
		work = sym crate::PROC_CALL_WORK_PENDING,
		load = sym crate::WORLD_LOAD_PENDING,
		intercepted = sym runtime_intercepted,
		proc_id = const PROC_ID_ARGUMENT,
		hooked = sym HOOKED_LOW_WORDS,
		calls = sym crate::PROC_CALLS,
		original = sym call_proc_by_id_original,
		trampoline = sym call_proc_by_id_hook_trampoline
	)
}

#[cfg(windows)]
#[unsafe(naked)]
unsafe extern "C" fn call_proc_by_id_entry() {
	core::arch::naked_asm!(
		"cmp byte ptr [{work}], 0",
		"jne 2f",
		"cmp byte ptr [{load}], 0",
		"jne 2f",
		"cmp byte ptr [{intercepted}], 0",
		"jne 2f",
		"movzx eax, word ptr [esp + {proc_id}]",
		"cmp byte ptr [{hooked} + eax], 0",
		"jne 2f",
		"inc dword ptr [{calls}]",
		"jmp dword ptr [{original}]",
		"2:",
		"jmp {trampoline}",
		work = sym crate::PROC_CALL_WORK_PENDING,
		load = sym crate::WORLD_LOAD_PENDING,
		intercepted = sym runtime_intercepted,
		proc_id = const PROC_ID_ARGUMENT,
		hooked = sym HOOKED_LOW_WORDS,
		calls = sym crate::PROC_CALLS,
		original = sym call_proc_by_id_original,
		trampoline = sym call_proc_by_id_hook_trampoline
	)
}

fn hook_by_id(id: raw_types::procs::ProcId, hook: ProcHook, hook_path: String) -> Result<(), HookFailure> {
	let mut map = PROC_HOOKS.0.borrow_mut();
	if let std::collections::hash_map::Entry::Vacant(e) = map.entry(id) {
		e.insert((hook, hook_path));
		unsafe {
			let count = &mut HOOKED_LOW_WORDS[id.0 as usize & 0xFFFF];
			*count = count.saturating_add(1);
		}
		Ok(())
	} else {
		Err(HookFailure::AlreadyHooked)
	}
}

/// Removes all proc hooks installed by this library.
///
/// Called automatically during shutdown and on a failed initialization. Must be
/// called on BYOND's thread because the hook table is unsynchronized.
pub(crate) fn clear_hooks() {
	PROC_HOOKS.0.borrow_mut().clear();
	unsafe {
		HOOKED_LOW_WORDS.fill(0);
	}
}

/// Installs `hook` on the first definition (override ID 0) of the proc at
/// `name`.
///
/// Accepts proc paths in the same format as `Proc::find`. The hook remains
/// active until shutdown removes it and must be re-installed for each new world
/// (use `#[hook("/path")]` to install automatically on every initialization).
///
/// # Errors
///
/// - `HookFailure::ProcNotFound` if no proc exists at `name`, or if called
///   before initialization, after shutdown, or off BYOND's thread.
/// - `HookFailure::AlreadyHooked` if this library already has a hook on that
///   proc.
pub fn hook<S: Into<String>>(name: S, hook: ProcHook) -> Result<(), HookFailure> {
	if !crate::world_is_live() {
		return Err(HookFailure::NotInitialized);
	}
	match super::proc::get_proc(name) {
		Some(p) => hook_by_id(p.id, hook, p.path.to_owned()),
		None => Err(HookFailure::ProcNotFound)
	}
}

impl Proc {
	/// Installs `func` on this specific proc definition (by proc ID), allowing
	/// individual overrides to be hooked independently.
	///
	/// The hook remains active until shutdown removes it.
	///
	/// # Errors
	///
	/// Returns `HookFailure::AlreadyHooked` if this library already has a hook
	/// on this proc ID.
	pub fn hook(&self, func: ProcHook) -> Result<(), HookFailure> {
		hook_by_id(self.id, func, self.path.to_owned())
	}

	/// Returns `true` if this library currently has a hook installed on this
	/// proc ID.
	///
	/// Call on BYOND's thread. Does not report hooks installed by other
	/// auxtools libraries loaded into the same world.
	pub fn is_hooked(&self) -> bool {
		PROC_HOOKS.0.borrow().contains_key(&self.id)
	}
}

#[no_mangle]
extern "C" fn on_runtime(error: *const c_char) {
	let str = crate::text_of(unsafe { CStr::from_ptr(error) }.to_bytes());

	for func in inventory::iter::<RuntimeErrorHook> {
		let _ = crate::guard(|| func.0(&str));
	}
}

fn call_hook(hook: ProcHook, src: &Value, usr: &Value, args: Vec<Value>) -> DMResult {
	crate::guard(|| hook(src, usr, args)).unwrap_or_else(|panic| Err(crate::runtime!("panicked: {}", crate::panic_message(&*panic))))
}

fn hook_failed(proc_id: raw_types::procs::ProcId, error: &crate::Runtime) {
	if !crate::world_is_live() {
		return;
	}
	let path = PROC_HOOKS.0.borrow().get(&proc_id).map(|(_, path)| path.clone()).unwrap_or_default();
	if let (Some(trace), Ok(message)) = (
		Proc::find("/proc/auxtools_stack_trace"),
		Value::from_string(format!("{} HookPath: {}", error.message, path))
	) {
		let _ = trace.call(&[&message]);
	}
}

/// Returns `true` on builds where BYOND's interpreter bypasses the proc-call
/// detour.
///
/// This occurs on Linux before build 1664 (both BYOND 515 and 516), where GCC
/// inlined the interpreter's proc calls. Always returns `false` on Windows. On
/// affected builds, auxtools installs its instruction hook at initialization
/// and dispatches proc hooks when each frame begins.
///
/// Reads the BYOND build recorded during the first initialization, so before
/// initialization it returns `true` on Linux.
pub fn calls_bypass_detour() -> bool {
	cfg!(unix) && crate::version::get().1 < 1664
}

static RETURN_DOT: [u32; 1] = [0x00];

/// Runs per-call housekeeping and proc hook dispatch for a frame about to
/// execute its first instruction.
///
/// Completes any pending world-load initialization, drains `at_proc_calls`
/// work, increments the proc call counter, and invokes any installed hook on
/// the frame's proc (returning the hook's result without executing the proc's
/// bytecode). Called internally by the instruction hook on builds where
/// [`calls_bypass_detour`] returns `true`. Library authors should not call this
/// directly.
///
/// # Safety
///
/// `ctx` must point to the currently executing frame on BYOND's thread.
pub unsafe fn proc_starting(ctx: *mut raw_types::procs::ExecutionContext) {
	if std::ptr::eq(*(*ctx).bytecode() as *const u32, RETURN_DOT.as_ptr()) {
		return;
	}
	let instance = *(*ctx).proc_instance();
	if crate::on_proc_call((*instance).proc, (*instance).src.tag) && *(*ctx).bytecode_offset() == 0 && crate::world_is_live() {
		if let Some(proc) = Proc::from_id((*instance).proc) {
			let (now, _) = proc.bytecode_mut_ptr();
			*(*ctx).bytecode_mut() = now;
		}
	}

	let proc_id = (*instance).proc;
	if HOOKED_LOW_WORDS[proc_id.0 as usize & 0xFFFF] == 0 {
		return;
	}
	let Some(hook) = PROC_HOOKS.0.borrow().get(&proc_id).map(|(hook, _)| *hook) else {
		return;
	};
	let src = Value::from_raw((*instance).src);
	let usr = Value::from_raw((*instance).usr);
	let args = std::slice::from_raw_parts((*instance).args(), (*instance).args_count() as usize)
		.iter()
		.map(|v| Value::from_raw(*v))
		.collect();
	let result = match call_hook(hook, &src, &usr, args) {
		Ok(r) => {
			let raw = r.raw;
			std::mem::forget(r);
			raw
		}
		Err(e) => {
			hook_failed(proc_id, &e);
			Value::NULL.raw
		}
	};
	*(*ctx).dot_mut() = result;
	*(*ctx).bytecode_mut() = RETURN_DOT.as_ptr() as *mut u32;
	*(*ctx).bytecode_offset_mut() = 0;
}

#[no_mangle]
extern "C" fn call_proc_by_id_hook(
	ret: *mut raw_types::values::Value,
	usr_raw: raw_types::values::Value,
	_proc_type: u32,
	proc_id: raw_types::procs::ProcId,
	_override_depth: u32,
	src_raw: raw_types::values::Value,
	args_ptr: *mut raw_types::values::Value,
	num_args: usize,
	_callback: u32,
	_callback_value: u32
) -> u8 {
	crate::on_proc_call(proc_id, src_raw.tag);

	let Some(hook) = PROC_HOOKS.0.borrow().get(&proc_id).map(|(hook, _)| *hook) else {
		return 0;
	};

	let (src, usr, args) = unsafe {
		(
			Value::from_raw(src_raw),
			Value::from_raw(usr_raw),
			std::slice::from_raw_parts(args_ptr, num_args)
				.iter()
				.map(|v| Value::from_raw_owned(*v))
				.collect()
		)
	};

	let result = match call_hook(hook, &src, &usr, args) {
		Ok(r) => {
			let result_raw = r.raw;
			std::mem::forget(r);
			result_raw
		}
		Err(e) => {
			hook_failed(proc_id, &e);
			Value::NULL.raw
		}
	};
	unsafe {
		*ret = result;
	}
	1
}
