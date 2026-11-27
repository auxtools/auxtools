use std::collections::{BTreeMap, HashMap};

use auxtools::{
	raw_types::{
		procs::{ExecutionContext, ProcId},
		strings::StringId
	},
	*
};
use auxtools_protocol::{Coverage, CoveredFile, CoveredLine, HELPER_FILE};

use crate::{code_index::Code, instruction_hooking::original_code};

pub const DBG_LINE: u32 = dmasm::Opcode::DbgLine.word();
const DBG_FILE: u32 = dmasm::Opcode::DbgFile.word();

const MOST_LINES: usize = 1 << 22;

struct File {
	name: String,
	hits: Vec<u32>
}

struct Recording {
	files: HashMap<u32, Box<File>>,
	last: (u32, *mut File)
}

static mut RECORDING: Option<Box<Recording>> = None;

pub fn start() {
	unsafe {
		RECORDING = Some(Box::new(Recording {
			files: HashMap::new(),
			last: (0, std::ptr::null_mut())
		}));
	}
	instruction_hook::hear_all();
}

pub fn running() -> bool {
	unsafe { RECORDING.is_some() }
}

pub fn stop() -> Option<Coverage> {
	let recording = unsafe { RECORDING.take() }?;
	instruction_hook::hear_all();
	Some(recording.report())
}

pub fn discard() {
	if unsafe { RECORDING.take() }.is_some() {
		instruction_hook::hear_all();
	}
}

#[shutdown]
fn coverage_shutdown() {
	discard();
}

#[inline]
pub fn enter(ctx: *mut ExecutionContext) {
	unsafe {
		let Some(recording) = RECORDING.as_mut() else {
			return;
		};
		let at = (*(*ctx).bytecode()).add(*(*ctx).bytecode_offset() as usize);
		if *at == DBG_LINE && !matches!(*at.add(2), DBG_LINE | DBG_FILE) {
			recording.ran(*(*ctx).filename(), *at.add(1));
		}
	}
}

impl Recording {
	#[inline]
	unsafe fn ran(&mut self, file: StringId, line: u32) {
		if self.last.0 != file.0 || self.last.1.is_null() {
			let known = self.files.entry(file.0).or_insert_with(|| {
				let name = unsafe { StringRef::from_id(file) };
				Box::new(File {
					name: String::from_utf8_lossy(name.data()).into_owned(),
					hits: vec![]
				})
			});
			self.last = (file.0, &mut **known);
		}
		let hits = unsafe { &mut (*self.last.1).hits };
		let line = line as usize;
		if line >= hits.len() {
			if line >= MOST_LINES {
				return;
			}
			hits.resize(line + 1, 0);
		}
		hits[line] = hits[line].saturating_add(1);
	}

	fn report(self) -> Coverage {
		let mut files = lines_with_code();
		for file in self.files.into_values() {
			let Some(lines) = files.get_mut(&file.name) else {
				continue;
			};
			for (line, hits) in lines.iter_mut() {
				*hits += file.hits.get(*line as usize).copied().unwrap_or(0) as u64;
			}
		}
		Coverage {
			files: files
				.into_iter()
				.map(|(file, lines)| CoveredFile {
					file,
					lines: lines.into_iter().map(|(line, hits)| CoveredLine { line, hits }).collect()
				})
				.collect()
		}
	}
}

fn lines_with_code() -> BTreeMap<String, BTreeMap<u32, u64>> {
	let mut files: BTreeMap<String, BTreeMap<u32, u64>> = BTreeMap::new();
	for id in 0..runtime_tables::proc_count() {
		let Some(proc) = Proc::from_id(ProcId(id)) else {
			continue;
		};
		let code = Code::read(&original_code(&proc), &mut crate::DisassembleEnv);
		for (file, line) in code.lines_with_code() {
			if file == HELPER_FILE {
				continue;
			}
			match files.get_mut(file) {
				Some(lines) => {
					lines.entry(line).or_insert(0);
				}
				None => {
					files.insert(file.to_owned(), BTreeMap::from([(line, 0)]));
				}
			}
		}
	}
	files
}
