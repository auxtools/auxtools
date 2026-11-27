use std::collections::BTreeMap;

pub fn hits_in(lcov: &str, file: &str) -> BTreeMap<u64, u64> {
	let record = lcov
		.split("end_of_record\n")
		.find(|record| record.lines().nth(1) == Some(format!("SF:{}", file).as_str()))
		.unwrap_or_else(|| panic!("no record of {} in:\n{}", file, lcov));
	let hits: BTreeMap<u64, u64> = record
		.lines()
		.filter_map(|line| line.strip_prefix("DA:"))
		.map(|pair| {
			let (line, hits) = pair.split_once(',').unwrap();
			(line.parse().unwrap(), hits.parse().unwrap())
		})
		.collect();
	let ran = hits.values().filter(|hits| **hits != 0).count();
	assert!(
		record.ends_with(&format!("LF:{}\nLH:{}\n", hits.len(), ran)),
		"unexpected totals in the record of {}:\n{}",
		file,
		record
	);
	hits
}
