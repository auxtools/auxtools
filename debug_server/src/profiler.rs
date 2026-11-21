//! Count and time interpreter instructions, attributing each interval to the
//! preceding instruction and its callers.
//!
//! Subtract the cheapest instruction's average as estimated hook overhead.
//! Sample all running code so filtering procs does not change the baseline.
//!
//! Exclude time outside the interpreter. Running frames live on the native
//! stack; BYOND moves them to a new heap block on suspension. A frame unrelated
//! to the previous frame's call chain therefore marks an interval of engine
//! work.

use std::{collections::HashMap, time::Instant};

use auxtools::{
	instruction_hook::{ticks_since_last, time_instructions, timestamp},
	*
};

use crate::server_types::{ProfiledInstruction, ProfiledTotal};

/// Maximum tracked call depth. Deeper calls add nothing to callee time.
const MOST_CALLS: usize = 1024;

/// Slots of the code cache
const CACHED: usize = 256;

/// Minimum timed hits for an overhead sample.
const FLOOR_HITS: u64 = 256;

/// Capacity of the overhead sample cache.
const SAMPLED: usize = 4096;

/// Cache collisions before replacing a sample. Repeated hits retain the slot.
const SAMPLE_PATIENCE: u8 = 16;

/// Timing sample, including code outside the profile filter.
#[derive(Clone, Copy)]
struct Sample {
	/// The address of the instruction's first word, 0 for none
	instruction: usize,
	timed: u64,
	ticks: u64,
	/// Hits of other instructions since its last
	passed: u8
}

#[derive(Clone, Copy, Default)]
struct Counter {
	/// Times the instruction started
	hits: u64,
	/// Timed hits and elapsed ticks until the next instruction.
	timed: u64,
	ticks: u64,
	/// Elapsed ticks and timed instruction count in callees.
	callee_ticks: u64,
	callee_timed: u64
}

/// The measurements of one bytecode buffer
struct Code {
	proc: u32,
	bytecode: usize,
	/// Counter index + 1 per word offset; zero until the instruction runs.
	index: Box<[u32]>,
	counters: Vec<Counter>,
	/// Word offset for each counter.
	offsets: Vec<u16>
}

/// An instruction's counter: its code and an index into `Code::counters`.
#[derive(Clone, Copy)]
struct At(*mut Code, u32);

impl At {
	const NONE: Self = Self(std::ptr::null_mut(), 0);

	unsafe fn counter<'a>(self) -> Option<&'a mut Counter> {
		unsafe { self.0.as_mut() }.and_then(|code| code.counters.get_mut(self.1 as usize))
	}
}

/// Frame and proc-instance identity. Suspension moves the frame but preserves
/// the instance; later calls may reuse a freed frame address.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Frame {
	ctx: usize,
	instance: usize
}

impl Frame {
	const NONE: Self = Self { ctx: 0, instance: 0 };

	unsafe fn of(ctx: *mut raw_types::procs::ExecutionContext) -> Self {
		if ctx.is_null() {
			return Self::NONE;
		}
		Self {
			ctx: ctx as usize,
			instance: unsafe { *(*ctx).proc_instance() } as usize
		}
	}
}

/// A call in progress
struct Call {
	caller: Frame,
	/// The instruction that called, if its code is measured
	counter: At,
	/// `Profiler::ran` and `Profiler::timed` when the callee started
	ran: u64,
	timed: u64
}

pub struct Profiler {
	/// Procs measured; all when `None`
	only: Option<Vec<u32>>,
	/// By bytecode address. `None` for code that isn't measured
	codes: HashMap<usize, Option<Box<Code>>>,
	/// Recently run codes, by a hash of their address
	cache: [(usize, *mut Code); CACHED],
	last: Frame,
	last_parent: Frame,
	last_counter: At,
	last_opcode: u32,
	sleep: u32,
	/// Whether the last instruction ran in an evaluation nested inside the
	/// hook.
	evaluating: bool,
	/// The address of the last instruction, for `samples`
	last_instruction: usize,
	/// By a hash of the instruction's address
	samples: Box<[Sample]>,
	calls: Vec<Call>,
	/// Ticks of every timed interval, and how many there were
	ran: u64,
	timed: u64,
	started: (Instant, u64),
	/// Stop time, timestamp counter and estimated hook overhead.
	stopped: Option<(Instant, u64, f64)>
}

static mut PROFILER: Option<Box<Profiler>> = None;
/// Whether instructions are being measured. Read on every instruction
static mut RUNNING: bool = false;

/// Start a profile, replacing the last. `only` limits it to those procs.
pub fn start(only: Option<Vec<u32>>) {
	unsafe {
		PROFILER = Some(Box::new(Profiler {
			only,
			codes: HashMap::new(),
			cache: [(0, std::ptr::null_mut()); CACHED],
			last: Frame::NONE,
			last_parent: Frame::NONE,
			last_counter: At::NONE,
			last_opcode: 0,
			sleep: dmasm::Opcode::Sleep.word(),
			evaluating: false,
			last_instruction: 0,
			samples: vec![
				Sample {
					instruction: 0,
					timed: 0,
					ticks: 0,
					passed: 0
				};
				SAMPLED
			]
			.into_boxed_slice(),
			calls: Vec::new(),
			ran: 0,
			timed: 0,
			started: (Instant::now(), timestamp()),
			stopped: None
		}));
		RUNNING = true;
	}
	time_instructions(true);
}

/// Stop measuring. The measurements stay until `discard`.
pub fn stop() {
	unsafe {
		if RUNNING {
			RUNNING = false;
			time_instructions(false);
			if let Some(profiler) = PROFILER.as_mut() {
				profiler.stopped = Some((Instant::now(), timestamp(), profiler.floor()));
			}
		}
	}
}

pub fn discard() {
	stop();
	unsafe {
		PROFILER = None;
	}
}

pub fn exists() -> bool {
	unsafe { PROFILER.is_some() }
}

pub fn running() -> bool {
	unsafe { RUNNING }
}

// The world's code is gone
#[shutdown]
fn profiler_shutdown() {
	discard();
}

/// Record instruction entry, excluding debugger work and pauses inside the
/// hook. `evaluating` includes debugger expressions and their callees.
#[inline]
pub fn enter(ctx: *mut raw_types::procs::ExecutionContext, evaluating: bool) {
	unsafe {
		if RUNNING {
			if let Some(profiler) = PROFILER.as_mut() {
				profiler.enter(ctx, evaluating);
			}
		}
	}
}

/// Whether an instruction can move execution elsewhere in its proc, such as
/// back to offset 0. An instruction that ends a call cannot.
pub(crate) fn can_jump(opcode: u32) -> bool {
	use dmasm::Opcode::*;
	[
		Jmp,
		Jnz,
		Jz,
		Spawn,
		Switch,
		PickSwitch,
		SwitchRange,
		JmpOr,
		JmpAnd,
		JmpLoop,
		JnzLoop,
		JzLoop,
		ForRange,
		ForRangeStep,
		Try,
		Catch,
		TryJmp,
		SetCacheJmpIfNull,
		SetCachePopJmpIfNull
	]
	.iter()
	.any(|op| op.word() == opcode)
}

impl Profiler {
	unsafe fn enter(&mut self, ctx: *mut raw_types::procs::ExecutionContext, evaluating: bool) {
		// Nested evaluation breaks the instruction timing interval.
		if evaluating != self.evaluating {
			self.evaluating = evaluating;
			self.last = Frame::NONE;
			self.last_parent = Frame::NONE;
			self.last_counter = At::NONE;
		}
		let spent = ticks_since_last();
		let (frame, parent, bytecode, offset) = unsafe {
			(
				Frame::of(ctx),
				Frame::of(*(*ctx).parent_context()),
				*(*ctx).bytecode(),
				*(*ctx).bytecode_offset() as usize
			)
		};

		// Detect calls and returns from frame identity.
		let mut returned = None;
		let followed = if frame == self.last {
			// A new call of a proc can get the frame the last call had. At
			// offset 0, only the proc's own jump continues the same call.
			offset != 0 || can_jump(self.last_opcode)
		} else if parent == self.last && self.last != Frame::NONE {
			true
		} else if let Some(depth) = self.calls.iter().rposition(|call| call.caller == frame) {
			returned = Some(depth);
			true
		} else {
			// Its caller, when the call wasn't seen
			frame == self.last_parent
		};
		// Exclude sleeps even when execution resumes immediately.
		if let Some(spent) = spent.filter(|_| followed && self.last_opcode != self.sleep) {
			self.ran += spent;
			self.timed += 1;
			self.sample(spent);
			if let Some(counter) = unsafe { self.last_counter.counter() } {
				counter.ticks += spent;
				counter.timed += 1;
			}
		}

		if !followed {
			self.calls.clear();
		} else if let Some(depth) = returned {
			// Frames above ended without returning (a runtime, a throw)
			self.calls.truncate(depth + 1);
			if let Some(call) = self.calls.pop() {
				if let Some(counter) = unsafe { call.counter.counter() } {
					counter.callee_ticks += self.ran - call.ran;
					counter.callee_timed += self.timed - call.timed;
				}
			}
		} else if frame != self.last && parent == self.last && self.calls.len() < MOST_CALLS {
			self.calls.push(Call {
				caller: self.last,
				counter: self.last_counter,
				ran: self.ran,
				timed: self.timed
			});
		}

		let code = self.code(bytecode, unsafe { (*(frame.instance as *mut raw_types::procs::ProcInstance)).proc });
		self.last_counter = match unsafe { code.as_mut() }.and_then(|code| code.hit(offset)) {
			Some(at) => At(code, at),
			None => At::NONE
		};
		self.last = frame;
		self.last_parent = parent;
		self.last_opcode = unsafe { *bytecode.add(offset) };
		self.last_instruction = bytecode as usize + offset * std::mem::size_of::<u32>();
	}

	/// Add the previous instruction interval to its overhead sample.
	fn sample(&mut self, spent: u64) {
		let sample = &mut self.samples[(self.last_instruction >> 2) % SAMPLED];
		if sample.instruction == self.last_instruction {
			sample.passed = 0;
		} else if sample.passed < SAMPLE_PATIENCE && sample.instruction != 0 {
			sample.passed += 1;
			return;
		} else {
			*sample = Sample {
				instruction: self.last_instruction,
				timed: 0,
				ticks: 0,
				passed: 0
			};
		}
		sample.timed += 1;
		sample.ticks += spent;
	}

	/// The measurements of the code at `bytecode`, null if it isn't measured
	fn code(&mut self, bytecode: *mut u32, proc: raw_types::procs::ProcId) -> *mut Code {
		let address = bytecode as usize;
		let slot = (address >> 3 ^ address >> 11) % CACHED;
		if self.cache[slot].0 == address {
			return self.cache[slot].1;
		}
		let only = &self.only;
		let code = self
			.codes
			.entry(address)
			.or_insert_with(|| Self::measured(only.as_deref(), bytecode, proc));
		let code = code.as_deref_mut().map_or(std::ptr::null_mut(), std::ptr::from_mut);
		self.cache[slot] = (address, code);
		code
	}

	/// Allocate counters for new code, excluding helpers and filtered procs.
	fn measured(only: Option<&[u32]>, bytecode: *mut u32, proc: raw_types::procs::ProcId) -> Option<Box<Code>> {
		if only.is_some_and(|only| !only.contains(&proc.0)) {
			return None;
		}
		let proc = Proc::from_id(proc)?;
		if crate::launched::is_helper(&proc) {
			return None;
		}
		let (current, len) = unsafe { proc.bytecode_mut_ptr() };
		let len = if current == bytecode {
			len as usize
		} else {
			// Code a hot reload replaced, still running
			auxtools::bytecode_at(bytecode)?.len()
		};
		Some(Box::new(Code {
			proc: proc.id.0,
			bytecode: bytecode as usize,
			index: vec![0; len].into_boxed_slice(),
			counters: vec![],
			offsets: vec![]
		}))
	}

	/// Nanoseconds a tick, and seconds measured
	fn clock(&self) -> (f64, f64) {
		let (ended, ticks) = self
			.stopped
			.map_or_else(|| (Instant::now(), timestamp()), |(ended, ticks, _)| (ended, ticks));
		let elapsed = ended.duration_since(self.started.0);
		let ticks = ticks.wrapping_sub(self.started.1).max(1);
		(elapsed.as_nanos() as f64 / ticks as f64, elapsed.as_secs_f64())
	}

	/// Estimate hook overhead from the cheapest sample with enough timed hits.
	/// Return zero if no sample qualifies.
	fn floor(&self) -> f64 {
		if let Some((.., floor)) = self.stopped {
			return floor;
		}
		let floor = (self.samples.iter())
			.filter(|sample| sample.timed >= FLOOR_HITS)
			.map(|sample| sample.ticks as f64 / sample.timed as f64)
			.fold(f64::INFINITY, f64::min);
		if floor.is_finite() {
			floor
		} else {
			0.0
		}
	}
}

impl Code {
	/// Count a hit and return its counter index.
	fn hit(&mut self, offset: usize) -> Option<u32> {
		let index = self.index.get_mut(offset)?;
		if *index == 0 {
			self.counters.push(Counter::default());
			self.offsets.push(offset as u16);
			*index = self.counters.len() as u32;
		}
		let at = *index - 1;
		self.counters[at as usize].hits += 1;
		Some(at)
	}
}

impl Counter {
	/// Self and callee time in nanoseconds. Estimate untimed hits from timed
	/// hits.
	fn took(&self, floor: f64, ns: f64) -> (u64, u64) {
		let own = match self.timed {
			0 => 0.0,
			timed => (self.ticks as f64 / timed as f64 - floor).max(0.0) * self.hits as f64
		};
		let callee = (self.callee_ticks as f64 - floor * self.callee_timed as f64).max(0.0);
		((own * ns) as u64, (callee * ns) as u64)
	}
}

/// Aggregate profile measurements.
pub struct Totals {
	pub seconds: f64,
	pub running: bool,
	/// Nanoseconds taken off each instruction for the profiler itself
	pub overhead: f64,
	/// Proc ID, bytecode address and totals for each executed buffer.
	pub codes: Vec<(u32, usize, ProfiledTotal)>
}

pub fn totals() -> Option<Totals> {
	let profiler = unsafe { PROFILER.as_ref() }?;
	let (ns, seconds) = profiler.clock();
	let floor = profiler.floor();
	let mut codes = vec![];
	for code in profiler.codes.values().flatten() {
		let first = code.index.first().and_then(|first| code.counters.get((*first as usize).checked_sub(1)?));
		let mut total = ProfiledTotal {
			calls: first.map_or(0, |first| first.hits),
			hits: 0,
			own_ns: 0,
			callee_ns: 0
		};
		for counter in &code.counters {
			let (own, callee) = counter.took(floor, ns);
			total.hits += counter.hits;
			total.own_ns += own;
			total.callee_ns += callee;
		}
		if total.hits != 0 {
			codes.push((code.proc, code.bytecode, total));
		}
	}
	Some(Totals {
		seconds,
		running: running(),
		overhead: floor * ns,
		codes
	})
}

/// Return executed instructions for a bytecode buffer.
pub fn instructions(bytecode: usize) -> Option<Vec<ProfiledInstruction>> {
	let profiler = unsafe { PROFILER.as_ref() }?;
	let code = profiler.codes.get(&bytecode)?.as_ref()?;
	let (ns, _) = profiler.clock();
	let floor = profiler.floor();
	let mut instructions: Vec<ProfiledInstruction> = (code.counters.iter().zip(&code.offsets))
		.map(|(counter, offset)| {
			let (own_ns, callee_ns) = counter.took(floor, ns);
			ProfiledInstruction {
				offset: *offset,
				hits: counter.hits,
				own_ns,
				callee_ns
			}
		})
		.collect();
	instructions.sort_by_key(|instruction| instruction.offset);
	Some(instructions)
}
