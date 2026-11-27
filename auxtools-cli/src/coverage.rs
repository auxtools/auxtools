use std::{collections::BTreeMap, fmt::Write, path::Path};

use auxtools_protocol::Coverage;

#[derive(Default, Debug, PartialEq)]
pub struct Report {
	pub files: BTreeMap<String, BTreeMap<u32, u64>>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Count {
	pub lines: usize,
	pub covered: usize
}

impl Count {
	fn of(lines: &BTreeMap<u32, u64>) -> Self {
		Self {
			lines: lines.len(),
			covered: lines.values().filter(|hits| **hits != 0).count()
		}
	}

	pub fn percent(self) -> f64 {
		match self.lines {
			0 => 0.0,
			lines => self.covered as f64 * 100.0 / lines as f64
		}
	}

	fn rate(self) -> String {
		match self.lines {
			0 => "0".to_owned(),
			lines => format!("{:.4}", self.covered as f64 / lines as f64)
		}
	}
}

impl Report {
	pub fn add(&mut self, recorded: Coverage) {
		for file in recorded.files {
			let lines = self.files.entry(file.file.replace('\\', "/")).or_default();
			for line in file.lines {
				let hits = lines.entry(line.line).or_default();
				*hits = hits.saturating_add(line.hits);
			}
		}
	}

	pub fn counts(&self) -> impl Iterator<Item = (&str, Count)> {
		self.files.iter().map(|(file, lines)| (file.as_str(), Count::of(lines)))
	}

	pub fn total(&self) -> Count {
		self.counts().fold(Count { lines: 0, covered: 0 }, |total, (_, file)| Count {
			lines: total.lines + file.lines,
			covered: total.covered + file.covered
		})
	}

	pub fn summary(&self) -> String {
		let total = self.total();
		format!(
			"{} of {} lines covered ({:.1}%) in {} of {} files",
			total.covered,
			total.lines,
			total.percent(),
			self.counts().filter(|(_, file)| file.covered != 0).count(),
			self.files.len()
		)
	}

	pub fn lcov(&self) -> String {
		let mut text = String::new();
		for (file, lines) in &self.files {
			let _ = writeln!(text, "TN:\nSF:{}", file);
			for (line, hits) in lines {
				let _ = writeln!(text, "DA:{},{}", line, hits);
			}
			let count = Count::of(lines);
			let _ = writeln!(text, "LF:{}\nLH:{}\nend_of_record", count.lines, count.covered);
		}
		text
	}

	pub fn cobertura(&self, timestamp: u64) -> String {
		let mut packages: BTreeMap<&str, Vec<(&str, &str, &BTreeMap<u32, u64>)>> = BTreeMap::new();
		for (file, lines) in &self.files {
			let (directory, name) = file.rsplit_once('/').unwrap_or(("", file));
			packages.entry(directory).or_default().push((name, file, lines));
		}

		let total = self.total();
		let mut text = String::new();
		let _ = writeln!(
			text,
			"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE coverage SYSTEM \"http://cobertura.sourceforge.net/xml/coverage-04.dtd\">\n<coverage \
			 line-rate=\"{}\" branch-rate=\"0\" lines-covered=\"{}\" lines-valid=\"{}\" branches-covered=\"0\" branches-valid=\"0\" complexity=\"0\" \
			 version=\"auxtools {}\" timestamp=\"{}\">\n\t<sources>\n\t\t<source>.</source>\n\t</sources>\n\t<packages>",
			total.rate(),
			total.covered,
			total.lines,
			env!("CARGO_PKG_VERSION"),
			timestamp
		);
		for (directory, files) in packages {
			let count = files.iter().fold(Count { lines: 0, covered: 0 }, |count, (.., lines)| {
				let file = Count::of(lines);
				Count {
					lines: count.lines + file.lines,
					covered: count.covered + file.covered
				}
			});
			let _ = writeln!(
				text,
				"\t\t<package name=\"{}\" line-rate=\"{}\" branch-rate=\"0\" complexity=\"0\">\n\t\t\t<classes>",
				xml(directory),
				count.rate()
			);
			for (name, file, lines) in files {
				let _ = writeln!(
					text,
					"\t\t\t\t<class name=\"{}\" filename=\"{}\" line-rate=\"{}\" branch-rate=\"0\" \
					 complexity=\"0\">\n\t\t\t\t\t<methods/>\n\t\t\t\t\t<lines>",
					xml(name),
					xml(file),
					Count::of(lines).rate()
				);
				for (line, hits) in lines {
					let _ = writeln!(text, "\t\t\t\t\t\t<line number=\"{}\" hits=\"{}\"/>", line, hits);
				}
				text.push_str("\t\t\t\t\t</lines>\n\t\t\t\t</class>\n");
			}
			text.push_str("\t\t\t</classes>\n\t\t</package>\n");
		}
		text.push_str("\t</packages>\n</coverage>\n");
		text
	}

	pub fn write(&self, lcov: Option<&Path>, cobertura: Option<&Path>) -> Result<(), String> {
		let write = |file: &Path, text: String| std::fs::write(file, text).map_err(|e| format!("cannot write {}: {}", file.display(), e));
		if let Some(file) = lcov {
			write(file, self.lcov())?;
		}
		if let Some(file) = cobertura {
			let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
			write(file, self.cobertura(now.map_or(0, |now| now.as_secs())))?;
		}
		Ok(())
	}
}

fn xml(text: &str) -> String {
	let mut escaped = String::with_capacity(text.len());
	for c in text.chars() {
		match c {
			'&' => escaped.push_str("&amp;"),
			'<' => escaped.push_str("&lt;"),
			'>' => escaped.push_str("&gt;"),
			'"' => escaped.push_str("&quot;"),
			c => escaped.push(c)
		}
	}
	escaped
}

#[cfg(test)]
mod tests {
	use auxtools_protocol::{Coverage, CoveredFile, CoveredLine};

	use super::Report;

	fn recorded(files: &[(&str, &[(u32, u64)])]) -> Coverage {
		Coverage {
			files: files
				.iter()
				.map(|(file, lines)| CoveredFile {
					file: (*file).to_owned(),
					lines: lines.iter().map(|&(line, hits)| CoveredLine { line, hits }).collect()
				})
				.collect()
		}
	}

	fn report() -> Report {
		let mut report = Report::default();
		report.add(recorded(&[("code/a&b.dm", &[(3, 2), (4, 0), (9, 1)]), ("main.dm", &[(1, 0)])]));
		report.add(recorded(&[("code\\a&b.dm", &[(4, 5), (12, 0)]), ("code/sub/c.dm", &[(7, 1)])]));
		report
	}

	#[test]
	fn recordings_of_one_run_add_up() {
		let report = report();
		assert_eq!(
			report.files["code/a&b.dm"].iter().map(|(line, hits)| (*line, *hits)).collect::<Vec<_>>(),
			[(3, 2), (4, 5), (9, 1), (12, 0)]
		);
		assert_eq!(report.summary(), "4 of 6 lines covered (66.7%) in 2 of 3 files");
		assert_eq!(Report::default().summary(), "0 of 0 lines covered (0.0%) in 0 of 0 files");
	}

	#[test]
	fn an_lcov_tracefile_has_a_record_for_each_file() {
		let records = [
			[
				"TN:",
				"SF:code/a&b.dm",
				"DA:3,2",
				"DA:4,5",
				"DA:9,1",
				"DA:12,0",
				"LF:4",
				"LH:3",
				"end_of_record"
			]
			.as_slice(),
			["TN:", "SF:code/sub/c.dm", "DA:7,1", "LF:1", "LH:1", "end_of_record"].as_slice(),
			["TN:", "SF:main.dm", "DA:1,0", "LF:1", "LH:0", "end_of_record"].as_slice()
		];
		assert_eq!(report().lcov(), records.concat().join("\n") + "\n");
	}

	#[test]
	fn a_cobertura_report_has_a_package_for_each_directory() {
		let text = report().cobertura(17);
		let version = env!("CARGO_PKG_VERSION");
		assert!(
			text.contains(&format!(
				"<coverage line-rate=\"0.6667\" branch-rate=\"0\" lines-covered=\"4\" lines-valid=\"6\" branches-covered=\"0\" branches-valid=\"0\" \
				 complexity=\"0\" version=\"auxtools {}\" timestamp=\"17\">",
				version
			)),
			"{}",
			text
		);
		let packages: Vec<&str> = text.lines().filter(|line| line.contains("<package ")).map(str::trim).collect();
		assert_eq!(packages, [
			"<package name=\"\" line-rate=\"0.0000\" branch-rate=\"0\" complexity=\"0\">",
			"<package name=\"code\" line-rate=\"0.7500\" branch-rate=\"0\" complexity=\"0\">",
			"<package name=\"code/sub\" line-rate=\"1.0000\" branch-rate=\"0\" complexity=\"0\">"
		]);
		assert!(
			text.contains("<class name=\"a&amp;b.dm\" filename=\"code/a&amp;b.dm\" line-rate=\"0.7500\" branch-rate=\"0\" complexity=\"0\">")
				&& text.contains("\t\t\t\t\t\t<line number=\"4\" hits=\"5\"/>\n")
				&& text.ends_with("\t</packages>\n</coverage>\n"),
			"{}",
			text
		);
		assert_eq!(text.matches("<line ").count(), 6);
	}
}
