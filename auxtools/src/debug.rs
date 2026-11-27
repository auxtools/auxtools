use std::{cell::RefCell, collections::HashMap};

use crate::{
	raw_types::{funcs, procs},
	Proc, StringRef, Value
};

#[derive(Clone)]
struct Names {
	locals: Vec<String>,
	parameters: Vec<String>
}

thread_local! {
	static LOCAL_NAMES: RefCell<HashMap<(procs::ProcId, usize), Names>> = RefCell::new(HashMap::new());
}

/// Records local variable names for frames of `proc` executing the bytecode
/// starting at `bytecode`.
///
/// Also snapshots `proc`'s current parameter names so active frames continue to
/// display them if a reload later renames parameters. `CallStacks`,
/// `local_names_for`, and `parameter_names_for` prefer these recorded names
/// over BYOND's proc table.
///
/// Calling this again for the same proc and address replaces the previous
/// record. Records are stored per thread (use on BYOND's thread) and cleared at
/// shutdown. `bytecode` is used only as a lookup key and is never dereferenced.
pub fn set_local_names(proc: &Proc, bytecode: *const u32, names: Vec<String>) {
	let names = Names {
		locals: names,
		parameters: proc.parameter_names().into_iter().map(String::from).collect()
	};
	LOCAL_NAMES.with(|map| map.borrow_mut().insert((proc.id, bytecode as usize), names));
}

pub fn has_local_names(proc: &Proc, bytecode: *const u32) -> bool {
	LOCAL_NAMES.with(|map| map.borrow().contains_key(&(proc.id, bytecode as usize)))
}

pub fn local_names_for(proc: &Proc, bytecode: *const u32) -> Vec<String> {
	LOCAL_NAMES
		.with(|map| map.borrow().get(&(proc.id, bytecode as usize)).map(|names| names.locals.clone()))
		.unwrap_or_else(|| proc.local_names().into_iter().map(String::from).collect())
}

pub fn parameter_names_for(proc: &Proc, bytecode: *const u32) -> Vec<String> {
	LOCAL_NAMES
		.with(|map| map.borrow().get(&(proc.id, bytecode as usize)).map(|names| names.parameters.clone()))
		.unwrap_or_else(|| proc.parameter_names().into_iter().map(String::from).collect())
}

pub(crate) fn copy_local_names(from: *const u32, to: *const u32, procs: impl Fn(procs::ProcId) -> bool) {
	LOCAL_NAMES.with(|map| {
		let mut map = map.borrow_mut();
		let copied: Vec<_> = map
			.iter()
			.filter(|((proc, at), _)| *at == from as usize && procs(*proc))
			.map(|(&(proc, _), names)| ((proc, to as usize), names.clone()))
			.collect();
		map.extend(copied);
	});
}

pub(crate) fn clear_local_names() {
	LOCAL_NAMES.with(|map| map.borrow_mut().clear());
}

/// A snapshot of a single DM call stack frame captured when reading
/// [`CallStacks`].
///
/// `context` and `instance` are raw pointers into BYOND's live frame structures
/// and remain valid only until DM code runs again. The captured `Value` fields
/// (`usr`, `src`, `dot`, arguments, and locals) each hold a BYOND reference and
/// remain valid as a snapshot.
///
/// `offset` is the current bytecode offset. An argument's name is `None` if
/// more arguments were passed than the proc declares, and unnamed locals are
/// labeled `<local N>`. `file_name` and `line_number` are `None` when the frame
/// has no source file recorded. Not `Send`.
pub struct StackFrame {
	pub context: *mut procs::ExecutionContext,
	pub instance: *mut procs::ProcInstance,
	pub proc: Proc,
	pub offset: u16,
	pub usr: Value,
	pub src: Value,
	pub dot: Value,
	pub args: Vec<(Option<StringRef>, Value)>,
	pub locals: Vec<(StringRef, Value)>,
	pub file_name: Option<StringRef>,
	pub line_number: Option<u32>
}

/// Snapshots of the active DM call stack (`active`) and every sleeping proc's
/// stack (`suspended`), each ordered with the innermost frame first.
///
/// The constructors and inspection methods (`new`, `active`, `suspended`,
/// `suspended_first`, and `suspended_instances`) read BYOND's internal frame
/// structures directly. Call them only on BYOND's thread after the library's
/// first initialization, because the underlying pointers are null before then.
///
/// `suspended_first(limit)` reads at most `limit` sleeping procs in BYOND's
/// queue order. Raw pointers returned by `suspended_instances` remain valid
/// only until DM code runs again.
pub struct CallStacks {
	pub active: Vec<StackFrame>,
	pub suspended: Vec<Vec<StackFrame>>
}

impl StackFrame {
	unsafe fn from_context(context: *mut procs::ExecutionContext) -> StackFrame {
		let instance = *(*context).proc_instance();

		let proc = Proc::from_id((*instance).proc).unwrap();
		let offset = *(*context).bytecode_offset();
		let bytecode = *(*context).bytecode();
		let interned = |names: &[String]| names.iter().map(|name| StringRef::new(name).ok()).collect::<Vec<_>>();
		let names = LOCAL_NAMES
			.with(|map| (map.borrow().get(&(proc.id, bytecode as usize))).map(|names| (interned(&names.parameters), interned(&names.locals))));
		let (param_names, local_names) = names.unwrap_or_else(|| {
			(
				proc.parameter_names().into_iter().map(Some).collect(),
				proc.local_names().into_iter().map(Some).collect()
			)
		});

		let usr = Value::from_raw((*instance).usr);
		let src = Value::from_raw((*instance).src);
		let dot = Value::from_raw(*(*context).dot());

		let args = (0..(*instance).args_count())
			.map(|i| {
				let name = param_names.get(i as usize).cloned().flatten();
				(name, Value::from_raw(*((*instance).args()).add(i as usize)))
			})
			.collect();

		let locals = (0..*(*context).locals_count())
			.map(|i| {
				let name = match local_names.get(i as usize) {
					Some(Some(name)) => name.clone(),
					_ => StringRef::new(&format!("<local {}>", i)).unwrap()
				};
				(name, Value::from_raw(*(*(*context).locals()).add(i as usize)))
			})
			.collect();

		let mut file_name = None;
		let mut line_number = None;
		if (*context).filename().valid() {
			file_name = Some(StringRef::from_id(*(*context).filename()));
			line_number = Some(*(*context).line());
		}

		StackFrame {
			context,
			instance,
			proc,
			offset,
			usr,
			src,
			dot,
			args,
			locals,
			file_name,
			line_number
		}
	}
}

enum CallStackKind {
	Active,
	Suspended
}

impl Default for CallStacks {
	fn default() -> Self {
		Self::new()
	}
}

impl CallStacks {
	pub fn new() -> CallStacks {
		CallStacks {
			active: Self::active(),
			suspended: Self::suspended()
		}
	}

	/// Returns the currently running proc's call stack, with the innermost
	/// frame first.
	///
	/// Returns an empty `Vec` if no proc is running. Call on BYOND's thread
	/// after the library's first initialization.
	pub fn active() -> Vec<StackFrame> {
		unsafe { CallStacks::from_context(*funcs::CURRENT_EXECUTION_CONTEXT, CallStackKind::Active) }
	}

	pub fn suspended() -> Vec<Vec<StackFrame>> {
		Self::suspended_first(usize::MAX)
	}

	pub fn suspended_first(limit: usize) -> Vec<Vec<StackFrame>> {
		let mut suspended = vec![];

		unsafe {
			let buffer = (*funcs::SUSPENDED_PROCS_BUFFER).buffer;
			for x in funcs::suspended_range().take(limit) {
				let instance = *buffer.add(x);
				let context = (*instance).context;
				suspended.push(CallStacks::from_context(context, CallStackKind::Suspended));
			}
		}

		suspended
	}

	pub fn suspended_instances() -> Vec<*mut procs::ProcInstance> {
		unsafe {
			let buffer = (*funcs::SUSPENDED_PROCS_BUFFER).buffer;
			funcs::suspended_range().map(|x| *buffer.add(x)).collect()
		}
	}

	fn from_context(mut context: *mut procs::ExecutionContext, kind: CallStackKind) -> Vec<StackFrame> {
		let mut frames = vec![];

		loop {
			if context.is_null() {
				break;
			}

			unsafe {
				frames.push(StackFrame::from_context(context));
				context = *(*context).parent_context();
			}
		}

		match kind {
			CallStackKind::Active => frames,
			CallStackKind::Suspended => frames.into_iter().rev().collect()
		}
	}
}
