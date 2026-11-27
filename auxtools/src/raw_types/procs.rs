#![allow(clippy::missing_const_for_fn)]
use auxtools_impl::versioned;

use super::{misc, strings, values};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ProcId(pub u32);

#[repr(C)]
pub struct ProcEntry {
	pub path: strings::StringId,
	pub name: strings::StringId,
	pub desc: strings::StringId,
	pub category: strings::StringId,
	path_basename: strings::StringId,
	flags: u32,
	pub metadata: ProcMetadata
}

#[versioned(
	Pre1630 if crate::version::BYOND_VERSION_MINOR <= 1627,
	Post1630,
)]
#[repr(C)]
pub struct ProcMetadata {
	#[only_in(Post1630)]
	byte_24: i8,
	#[only_in(Post1630)]
	byte_25: u8,
	#[only_in(Post1630)]
	byte_26: i8,
	pub bytecode: misc::BytecodeId,
	pub locals: misc::LocalsId,
	pub parameters: misc::ParametersId,
	#[only_in(Post1630)]
	next_override: ProcId
}

#[repr(C)]
pub struct ProcInstance {
	pub proc: ProcId,
	/// The `proc_type` value this frame was invoked with, read back by `..()`.
	pub flags: u8,
	/// Which override of the proc this frame runs, 0 for the most derived.
	pub override_depth: u8,
	pub mega_hack: u16,
	pub usr: values::Value,
	pub src: values::Value,
	pub context: *mut ExecutionContext,
	pub argslist_idx: values::ValueData,
	unk_1: u32,
	inner: ProcInstanceInner
}

impl ProcInstance {
	pub fn args_count(&self) -> u32 {
		*self.inner.args_count()
	}

	pub fn args(&self) -> *mut values::Value {
		*self.inner.args()
	}

	pub fn time_to_resume(&self) -> u32 {
		*self.inner.time_to_resume()
	}
}

#[versioned(
	Pre516 if crate::version::BYOND_VERSION_MAJOR < 516,
	Post516,
)]
#[repr(C)]
struct ProcInstanceInner {
	#[only_in(Pre516)]
	unk_2: u32,
	#[only_in(Post516)]
	callback: u32,
	#[only_in(Post516)]
	callback_value: u32,
	pub args_count: u32,
	pub args: *mut values::Value,
	arg_local_store: [u8; 0x50],
	internal_arg_count: u32,
	external_arg_count: u32,
	pub time_to_resume: u32
}

/// BYOND's in-memory representation of a DM execution frame.
///
/// Because the struct layout varies across BYOND builds (for example, changing
/// on Windows at `516.1668`), read and write fields through the generated
/// `field()` and `field_mut()` accessor methods rather than direct field
/// access.
///
/// Those accessors default to the newest layout until the library's first
/// initialization has completed its init functions, so calling them during
/// `WorldLoad::prepare` or inside `#[init(full)]` callbacks on the first
/// initialization may read incorrect offsets. Pointers to an `ExecutionContext`
/// are valid only on BYOND's thread while the frame is alive.
#[versioned(
	Pre1668 if cfg!(unix) || crate::version::BYOND_VERSION_MAJOR <= 515 || (crate::version::BYOND_VERSION_MAJOR == 516 && crate::version::BYOND_VERSION_MINOR <= 1667),
	Post1668,
)]
#[repr(C)]
pub struct ExecutionContext {
	pub proc_instance: *mut ProcInstance,
	pub parent_context: *mut ExecutionContext,
	pub filename: strings::StringId,
	pub line: u32,
	pub bytecode: *mut u32,
	pub bytecode_offset: u16,
	pub test_flag: u8,
	#[only_in(Pre1668)]
	exec_call_flags: u8,
	#[only_in(Post1668)]
	exec_call_flags: u32,
	cache: values::Value,
	cache_key: values::Value,
	scratch_ptr_a: u32,
	scratch_ptr_b: u32,
	pub dot: values::Value,
	pub locals: *mut values::Value,
	pub stack: *mut values::Value,
	pub locals_count: u16,
	pub stack_size: u16,
	iterator_stack: u32,
	pub current_iterator: *mut values::Value,
	pub iterator_allocated: u32,
	pub iterator_length: u32,
	pub iterator_index: u32,
	pub iterator_filter_type: values::Value,
	pub iterator_filter_bitflags: u32,
	pub iterator_kind: u8,
	pub infinite_loop_count: u32,
	not_suspended: u8,
	caller_chain_preserve_depth: u8,
	pub background: u8,
	exception_state: u8,
	pub throw_handler: u32,
	time_now: [u32; 2],
	cpu_clock_init: [u32; 2],
	cpu_clock_quantum: [u32; 2],
	loop_timeout: [u32; 2]
}

#[repr(C)]
pub struct SuspendedProcsBuffer {
	pub buffer: *mut *mut ProcInstance
}

#[cfg(windows)]
#[repr(C)]
pub struct SuspendedProcs {
	pub front: usize,
	pub back: usize,
	pub capacity: usize
}

#[cfg(unix)]
#[repr(C)]
pub struct SuspendedProcs {
	pub back: usize,
	pub front: usize
}

#[cfg(test)]
mod layout_tests {
	use std::mem::{offset_of, size_of};

	#[test]
	fn execution_context_linux_and_old_windows() {
		type Gen = super::ExecutionContextPre1668;
		assert_eq!(size_of::<Gen>(), 0x94);
		assert_eq!(offset_of!(Gen, bytecode_offset), 0x14);
		assert_eq!(offset_of!(Gen, dot), 0x30);
		assert_eq!(offset_of!(Gen, locals), 0x38);
		assert_eq!(offset_of!(Gen, stack_size), 0x42);
		assert_eq!(offset_of!(Gen, current_iterator), 0x48);
		assert_eq!(offset_of!(Gen, iterator_kind), 0x64);
		assert_eq!(offset_of!(Gen, infinite_loop_count), 0x68);
	}

	#[test]
	fn execution_context_windows() {
		type Gen = super::ExecutionContextPost1668;
		assert_eq!(size_of::<Gen>(), 0x98);
		assert_eq!(offset_of!(Gen, bytecode_offset), 0x14);
		assert_eq!(offset_of!(Gen, dot), 0x34);
		assert_eq!(offset_of!(Gen, locals), 0x3C);
		assert_eq!(offset_of!(Gen, stack_size), 0x46);
		assert_eq!(offset_of!(Gen, current_iterator), 0x4C);
		assert_eq!(offset_of!(Gen, iterator_kind), 0x68);
		assert_eq!(offset_of!(Gen, infinite_loop_count), 0x6C);
	}

	#[test]
	fn proc_instance() {
		type Inner = super::ProcInstanceInnerPost516;
		let inner = offset_of!(super::ProcInstance, inner);
		assert_eq!(size_of::<super::ProcInstance>(), 0x90);
		assert_eq!(offset_of!(super::ProcInstance, override_depth), 0x05);
		assert_eq!(offset_of!(super::ProcInstance, context), 0x18);
		assert_eq!(inner + offset_of!(Inner, callback), 0x24);
		assert_eq!(inner + offset_of!(Inner, args_count), 0x2C);
		assert_eq!(inner + offset_of!(Inner, args), 0x30);
		assert_eq!(inner + offset_of!(Inner, external_arg_count), 0x88);
		assert_eq!(inner + offset_of!(Inner, time_to_resume), 0x8C);
		assert_eq!(inner + offset_of!(super::ProcInstanceInnerPre516, args_count), 0x28);
	}

	#[test]
	fn proc_entry() {
		let metadata = offset_of!(super::ProcEntry, metadata);
		assert_eq!(size_of::<super::ProcEntry>(), 44);
		assert_eq!(metadata + offset_of!(super::ProcMetadataPost1630, bytecode), 28);
		assert_eq!(metadata + offset_of!(super::ProcMetadataPost1630, next_override), 40);
		// before 1630 the entry is 36 bytes with the bytecode id at +24
		assert_eq!(metadata + size_of::<super::ProcMetadataPre1630>(), 36);
		assert_eq!(metadata + offset_of!(super::ProcMetadataPre1630, bytecode), 24);
	}
}
