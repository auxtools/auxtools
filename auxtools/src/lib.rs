#[doc(hidden)]
pub mod appearance_tables;
pub mod byond_ffi;
mod bytecode_manager;
pub mod debug;
pub mod hooks;
mod init;
pub mod instruction_hook;
mod list;
pub mod pointer;
mod proc;
pub mod raw_types;
pub mod registry;
mod runtime;
pub mod runtime_tables;
mod string;
mod string_intern;
mod symbols;
mod value;
mod value_from;
pub mod version;
mod weak_value;

use std::{
	any::Any,
	sync::{
		atomic::{AtomicBool, AtomicU32, Ordering},
		Mutex, OnceLock
	}
};

pub use auxtools_impl::{full_shutdown, hook, init, pin_dll, runtime_handler, shutdown};
pub use byond_scan;
pub use bytecode_manager::{bytecode_at, bytecode_changes, remember_bytecode};
pub use ctor;
pub use hooks::{CompileTimeHook, HookFailure, ProcHook, RuntimeErrorHook};
use init::{get_init_level, set_init_level, InitLevel};
pub use init::{FullInitFunc, FullShutdownFunc, PartialInitFunc, PartialShutdownFunc};
pub use inventory;
pub use list::List;
pub use proc::Proc;
pub use raw_types::variables::VariableNameIdTable;
pub use registry::{byond_build, find_recipe};
pub use runtime::{DMResult, Runtime};
pub use string::{text_of, StringRef};
pub use string_intern::InternedString;
pub use value::Value;
pub use weak_value::WeakValue;

#[cfg(windows)]
extern crate winapi;

/// Controls whether the library pins itself in memory on Windows.
///
/// Defaults to `true` unless `pin_dll!(false)` clears it at library load time.
/// The value is read during the first initialization, when pinning takes
/// permanent effect, and again during `auxtools_full_shutdown`, which calls
/// `FreeLibrary` when this flag is `false`. Has no effect on Linux.
///
/// Configure this flag with `pin_dll!` rather than storing to it directly.
pub static PIN_DLL: AtomicBool = AtomicBool::new(true);

pub const BYONDCORE: &str = byond_scan::MODULE_NAME;

#[cfg(windows)]
fn pin_dll() -> Result<(), ()> {
	unsafe {
		use winapi::um::libloaderapi::{GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_PIN};
		let mut module = std::ptr::null_mut();

		let flags = match PIN_DLL.load(Ordering::Relaxed) {
			true => GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
			false => GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
		};

		let res = GetModuleHandleExW(flags, pin_dll as *const _, &mut module);

		if res == 0 {
			return Err(());
		}
	}
	Ok(())
}
#[cfg(unix)]
fn pin_dll() -> Result<(), ()> {
	Ok(())
}

/// Runs `work` and catches any unwinding panic.
///
/// Returns `Ok` with the return value of `work`, or `Err` with the panic
/// payload if `work` unwinds. A panic that unwinds across an `extern "C"`
/// boundary into BYOND aborts the world process.
pub fn guard<R>(work: impl FnOnce() -> R) -> Result<R, Box<dyn Any + Send>> {
	std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
}

pub fn panic_message(payload: &(dyn Any + Send)) -> String {
	let message = payload.downcast_ref::<&str>().copied();
	let message = message.or_else(|| payload.downcast_ref::<String>().map(String::as_str));
	message.unwrap_or("no message").to_owned()
}

static BYOND_THREAD: OnceLock<std::thread::ThreadId> = OnceLock::new();

pub(crate) fn on_byond_thread() -> bool {
	BYOND_THREAD.get() == Some(&std::thread::current().id())
}

static WORLD_GENERATION: AtomicU32 = AtomicU32::new(0);

pub(crate) fn world_generation() -> u32 {
	WORLD_GENERATION.load(Ordering::Relaxed)
}

static INIT_FAILURE: OnceLock<String> = OnceLock::new();

byond_ffi_fn! { auxtools_init(_input) {
	if let Some(failure) = INIT_FAILURE.get() {
		return Some(failure.clone());
	}

	match try_init() {
		Ok(()) => Some("SUCCESS".to_owned()),
		Err(e) => {
			hooks::clear_hooks();
			proc::clear_procs();
			Some(INIT_FAILURE.get_or_init(|| format!("FAILED ({})", e)).clone())
		}
	}
} }

fn try_init() -> Result<(), String> {
	try_init_with(None)
}

fn try_init_with(prepare: Option<fn() -> Result<(), String>>) -> Result<(), String> {
	guard(|| init_with(prepare)).unwrap_or_else(|panic| Err(format!("panicked: {}", panic_message(&*panic))))
}

fn init_with(prepare: Option<fn() -> Result<(), String>>) -> Result<(), String> {
	if get_init_level() == InitLevel::None {
		return Ok(());
	}

	let mut did_full = false;
	let mut did_partial = false;

	if get_init_level() == InitLevel::Full {
		did_full = true;
		init_full()?;
	}

	if get_init_level() == InitLevel::Partial {
		did_partial = true;
		BYOND_THREAD.get_or_init(|| std::thread::current().id());

		symbols::resolve_partial()?;

		proc::populate_procs();
		bytecode_manager::init();
		string_intern::setup_interned_strings();

		if let Some(prepare) = prepare {
			prepare()?;
		}

		for cthook in inventory::iter::<hooks::CompileTimeHook> {
			hooks::hook(cthook.proc_path, cthook.hook).map_err(|e| format!("could not hook proc {}: {:?}", cthook.proc_path, e))?;
		}
		set_init_level(InitLevel::None);
	}

	if did_full || FULL_INIT_FUNCS_PENDING.swap(false, Ordering::Relaxed) {
		init::run_full_init()?;
	}

	if did_partial {
		init::run_partial_init()?;
	}

	Ok(())
}

fn init_full() -> Result<(), String> {
	version::init()?;
	symbols::resolve_full()?;
	pin_dll().map_err(|()| "could not pin the library in memory".to_owned())?;
	hooks::init().map_err(|_| "could not initialize proc hooking".to_owned())?;
	instruction_hook::init()?;

	set_init_level(InitLevel::Partial);
	Ok(())
}

/// Lifecycle callbacks passed to [`init_on_world_load`].
///
/// All three callbacks are function pointers invoked on BYOND's thread during
/// the proc call that completes initialization, and they run again for each
/// subsequent world armed with [`init_on_next_world_load`]:
///
/// - `prepare` runs once the proc lookup table and `byond_string!` strings are
///   ready, before `#[hook("...")]` hooks are installed and before `#[init]`
///   functions run. Returning `Err` or panicking fails initialization.
/// - `ready` runs after initialization succeeds. [`starting_call`] returns
///   `Some` only while `ready` is executing. Panics are caught and ignored.
/// - `failed` receives the error message if initialization fails, after hooks
///   and the proc lookup table have been cleared. Panics are caught and
///   ignored.
pub struct WorldLoad {
	pub prepare: fn() -> Result<(), String>,
	pub ready: fn(),
	pub failed: fn(&str)
}

static WORLD_LOAD: OnceLock<&'static WorldLoad> = OnceLock::new();
pub(crate) static WORLD_LOAD_PENDING: AtomicBool = AtomicBool::new(false);
static FULL_INIT_FUNCS_PENDING: AtomicBool = AtomicBool::new(false);
static ENDING_WORLD: Mutex<Option<(raw_types::procs::ProcId, runtime_tables::ProcEntryWords)>> = Mutex::new(None);

/// Initializes the library before a world loads, for libraries injected into
/// BYOND rather than initialized through `auxtools_init`.
///
/// Reads the BYOND build, resolves BYOND's internal functions, and installs
/// detours immediately on the calling thread. The remaining initialization
/// completes on BYOND's thread at the first proc call, before global
/// initialization (ignoring calls BYOND makes while loading the world). If the
/// deferred phase fails, `WorldLoad::failed` is called and no further attempts
/// are made.
///
/// Detour handles are stored in thread-local storage on the calling thread, so
/// `auxtools_full_shutdown` will fail unless it runs on that same thread.
///
/// # Errors
///
/// Returns an error if `init_on_world_load` has already been called (even if
/// the first call failed), if the BYOND build is unsupported, or if a required
/// BYOND function cannot be found.
pub fn init_on_world_load(on_load: &'static WorldLoad) -> Result<(), String> {
	if WORLD_LOAD.set(on_load).is_err() {
		return Err("already waiting for the world to load".to_owned());
	}

	if let Err(e) = init_full() {
		hooks::clear_hooks();
		return Err(e);
	}

	FULL_INIT_FUNCS_PENDING.store(true, Ordering::Relaxed);
	WORLD_LOAD_PENDING.store(true, Ordering::Relaxed);
	Ok(())
}

/// Intended for worlds that reboot in place. Runs the same shutdown sequence as
/// `auxtools_shutdown` (`#[shutdown]` callbacks, hook removal, and clearing the
/// proc lookup and [`at_proc_calls`] queue), then waits for a subsequent proc
/// call to run [`WorldLoad`] initialization again. Has no effect unless the
/// library is fully initialized through [`init_on_world_load`].
///
/// When `added` is `None`, re-initialization triggers on the very next proc
/// call, which may still belong to the ending world. When `added` is `Some`,
/// pass a proc that was added after the world loaded so initialization waits
/// until that proc's entry is absent or changed at its ID in BYOND's proc
/// table.
pub fn init_on_next_world_load(added: Option<&Proc>) {
	if WORLD_LOAD.get().is_none() || get_init_level() != InitLevel::None {
		return;
	}
	*ENDING_WORLD.lock().unwrap() = added.and_then(|proc| Some((proc.id, unsafe { runtime_tables::proc_entry(proc.id) }?)));
	shutdown_partial();
	WORLD_LOAD_PENDING.store(true, Ordering::Relaxed);
}

pub fn shut_down_with_world() {
	if WORLD_LOAD.get().is_none() || get_init_level() != InitLevel::None {
		return;
	}
	shutdown_partial();
}

pub(crate) static PROC_CALLS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Returns a wrapping counter of proc calls, readable from any thread.
///
/// The counter is never reset, including across shutdowns, and serves as a
/// liveness indicator rather than an exact tally. On Linux before build 1664, a
/// call that goes through BYOND's call function is counted a second time when
/// its first instruction executes.
pub fn proc_calls() -> u32 {
	PROC_CALLS.load(Ordering::Relaxed)
}

static PROC_CALL_WORK: Mutex<Vec<fn() -> bool>> = Mutex::new(Vec::new());
pub(crate) static PROC_CALL_WORK_PENDING: AtomicBool = AtomicBool::new(false);

/// Queues `work` to run on BYOND's thread before each proc call until it
/// returns `true`.
///
/// May be called from any thread. A panic inside `work` is caught and treated
/// as returning `true`. Callbacks may invoke DM procs and enqueue additional
/// callbacks.
///
/// Every shutdown path clears the queue. Queued work can run before a pending
/// world-load initialization completes.
pub fn at_proc_calls(work: fn() -> bool) {
	PROC_CALL_WORK.lock().unwrap().push(work);
	PROC_CALL_WORK_PENDING.store(true, Ordering::Relaxed);
}

static STARTING: Mutex<Option<raw_types::procs::ProcId>> = Mutex::new(None);

/// Returns the ID of the proc whose call completed world-load initialization.
///
/// Returns `Some` only while `WorldLoad::ready` is executing, and `None` at all
/// other times (including inside proc hooks). `Proc::from_id` may return `None`
/// for the returned ID if the proc is absent from the proc table.
pub fn starting_call() -> Option<raw_types::procs::ProcId> {
	*STARTING.lock().unwrap()
}

static START_RUNS_NEW: AtomicU32 = AtomicU32::new(u32::MAX);

/// Directs the call that triggered world-load initialization to run `proc`'s
/// current bytecode instead of the bytecode it started with.
///
/// Call this from `WorldLoad::ready` (or `WorldLoad::prepare`) after replacing
/// the bytecode of the proc reported by [`starting_call`]. This is only needed
/// on builds where calls bypass the call detour (Linux before build 1664, see
/// `hooks::calls_bypass_detour`), where the frame already exists at its first
/// instruction. On other builds the call has not started yet and runs the
/// current bytecode automatically.
///
/// Because the existing frame keeps its allocated local slots, the replacement
/// bytecode must not require more locals than the original frame has. Has no
/// effect if called outside world-load initialization or with a different proc
/// ID.
pub fn start_runs_new_code(proc: raw_types::procs::ProcId) {
	START_RUNS_NEW.store(proc.0, Ordering::Relaxed);
}

pub(crate) fn on_proc_call(proc: raw_types::procs::ProcId, src: raw_types::values::ValueTag) -> bool {
	PROC_CALLS.store(PROC_CALLS.load(Ordering::Relaxed).wrapping_add(1), Ordering::Relaxed);
	if PROC_CALL_WORK_PENDING.load(Ordering::Relaxed) {
		let work = std::mem::take(&mut *PROC_CALL_WORK.lock().unwrap());
		let waiting: Vec<_> = work.into_iter().filter(|&work| !guard(work).unwrap_or(true)).collect();
		let mut queue = PROC_CALL_WORK.lock().unwrap();
		queue.extend(waiting);
		PROC_CALL_WORK_PENDING.store(!queue.is_empty(), Ordering::Relaxed);
	}

	if !WORLD_LOAD_PENDING.load(Ordering::Relaxed) {
		return false;
	}
	if src == raw_types::values::ValueTag::InitVarRecorder {
		return false;
	}
	{
		let mut ending = ENDING_WORLD.lock().unwrap();
		if let Some((id, entry)) = *ending {
			if unsafe { runtime_tables::proc_entry(id) } == Some(entry) {
				return false;
			}
		}
		*ending = None;
	}
	WORLD_LOAD_PENDING.store(false, Ordering::Relaxed);
	let Some(on_load) = WORLD_LOAD.get() else {
		return false;
	};

	START_RUNS_NEW.store(u32::MAX, Ordering::Relaxed);
	match try_init_with(Some(on_load.prepare)) {
		Ok(()) => {
			*STARTING.lock().unwrap() = Some(proc);
			let _ = guard(on_load.ready);
			*STARTING.lock().unwrap() = None;
		}
		Err(e) => {
			hooks::clear_hooks();
			proc::clear_procs();
			let _ = guard(|| (on_load.failed)(&e));
		}
	}
	START_RUNS_NEW.swap(u32::MAX, Ordering::Relaxed) == proc.0
}

byond_ffi_fn! { auxtools_shutdown(_input) {
	if get_init_level() != InitLevel::None {
		return Some("FAILED (already shut down)".to_owned())
	};
	shutdown_partial();
	Some("SUCCESS".to_owned())
} }

fn shutdown_partial() {
	PROC_CALL_WORK.lock().unwrap().clear();
	PROC_CALL_WORK_PENDING.store(false, Ordering::Relaxed);
	init::run_partial_shutdown();
	string_intern::destroy_interned_strings();
	bytecode_manager::shutdown();
	debug::clear_local_names();
	runtime_tables::reset();

	hooks::clear_hooks();
	proc::clear_procs();

	unsafe {
		raw_types::funcs::VARIABLE_NAME_ENTRIES = std::ptr::null();
		raw_types::funcs::VARIABLE_NAME_COUNT = std::ptr::null();
	}

	WORLD_GENERATION.fetch_add(1, Ordering::Relaxed);
	set_init_level(InitLevel::Partial);
}

byond_ffi_fn! { auxtools_full_shutdown(_input) {
	if get_init_level() == InitLevel::Full {
		return Some("FAILED (already shut down)".to_owned())
	};
	if get_init_level() == InitLevel::None {
		shutdown_partial();
	}
	hooks::shutdown();
	set_init_level(InitLevel::Full);
	init::run_full_shutdown();

	if !PIN_DLL.load(Ordering::Relaxed) {
		#[cfg(windows)]
		unsafe {
			use winapi::um::libloaderapi::{
				FreeLibrary, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
				GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
			};
			let mut module = std::ptr::null_mut();

			let get_handle_res = GetModuleHandleExW(
				GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
				auxtools_full_shutdown as *const _,
				&mut module,
			);

			if get_handle_res == 0 {
				return Some("FAILED (could not unpin the library from memory)".to_owned())
			}

			FreeLibrary(module);
		}
	};
	Some("SUCCESS".to_owned())
} }
