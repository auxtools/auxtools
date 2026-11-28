use std::{
	collections::HashMap,
	path::{Component, Path, PathBuf},
	time::{Duration, Instant}
};

use dmasm::debug_files;

const STALE: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct WorldFiles {
	dme: Option<PathBuf>,
	names: Option<(HashMap<String, PathBuf>, HashMap<PathBuf, String>, Instant)>
}

impl WorldFiles {
	pub const fn new(dme: Option<PathBuf>) -> Self {
		WorldFiles { dme, names: None }
	}

	pub fn dme(&self) -> Option<&Path> {
		self.dme.as_deref()
	}

	fn root(&self) -> Option<&Path> {
		self.dme.as_deref()?.parent()
	}

	fn read(&mut self) {
		let Some(dme) = &self.dme else { return };
		let mut by_name = HashMap::new();
		let mut by_path = HashMap::new();
		for (name, path) in debug_files::names(dme, &|_| true) {
			if Path::new(&name) != path {
				by_name.entry(name.clone()).or_insert_with(|| path.clone());
			}
			by_path.entry(path).or_insert(name);
		}
		self.names = Some((by_name, by_path, Instant::now()));
	}

	pub fn path(&mut self, name: &str) -> PathBuf {
		let name = name.replace('\\', "/");
		let Some(root) = self.root().map(Path::to_owned) else {
			return PathBuf::from(name);
		};
		if is_absolute(&name) {
			let built = Path::new(&name);
			return match &self.dme {
				Some(dme) if !built.is_file() && name.rsplit('/').next() == dme.file_name().and_then(|file| file.to_str()) => dme.clone(),
				_ => built.to_owned()
			};
		}
		if self.names.is_none() {
			self.read();
		}
		if let Some(path) = self.names.as_ref().and_then(|(by_name, ..)| by_name.get(&name)) {
			return debug_files::normalize(&root.join(path));
		}
		let path = debug_files::normalize(&root.join(&name));
		if !path.is_file() && self.names.as_ref().is_some_and(|(.., read)| read.elapsed() >= STALE) {
			self.read();
			if let Some(found) = self.names.as_ref().and_then(|(by_name, ..)| by_name.get(&name)) {
				return debug_files::normalize(&root.join(found));
			}
		}
		path
	}

	pub fn name(&mut self, path: &str) -> String {
		let Some(root) = self.root().map(Path::to_owned) else {
			return path.to_owned();
		};
		if self.names.is_none() {
			self.read();
		}
		let included = |relative: &Path| self.names.as_ref().and_then(|(_, by_path, _)| by_path.get(relative)).cloned();
		match Path::new(path).strip_prefix(&root) {
			Ok(inside) => included(&debug_files::normalize(inside)).unwrap_or_else(|| inside.to_string_lossy().replace('\\', "/")),
			Err(_) => relative_to(&root, Path::new(path))
				.and_then(|outside| included(&outside))
				.unwrap_or_else(|| path.to_owned())
		}
	}
}

pub fn relative_to(root: &Path, path: &Path) -> Option<PathBuf> {
	let (root, path) = (debug_files::normalize(root), debug_files::normalize(path));
	if !root.is_absolute() || !path.is_absolute() {
		return None;
	}
	let shared = root.components().zip(path.components()).take_while(|(a, b)| a == b).count();
	if !root.components().take(shared).any(|part| matches!(part, Component::RootDir)) {
		return None;
	}
	let mut relative = PathBuf::new();
	for _ in root.components().skip(shared) {
		relative.push("..");
	}
	relative.extend(path.components().skip(shared));
	Some(relative)
}

fn is_absolute(name: &str) -> bool {
	let bytes = name.as_bytes();
	name.starts_with('/') || (bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/')
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn names_and_paths() {
		let dir = std::env::temp_dir().join(format!("auxtools-world-files-{}", std::process::id()));
		let _ = std::fs::remove_dir_all(&dir);
		let write = |path: &str, text: &str| {
			let path = dir.join(path);
			std::fs::create_dir_all(path.parent().unwrap()).unwrap();
			std::fs::write(path, text).unwrap();
		};
		write("world.dme", "#include \"code\\map.dm\"\n#include \"_std\\__std.dme\"\n");
		write("code/map.dm", "");
		write("_std/__std.dme", "#include \"map.dm\"\n#include \"macros\\dir.dm\"\n");
		write("_std/map.dm", "");
		write("_std/macros/dir.dm", "");
		let mut files = WorldFiles::new(Some(dir.join("world.dme")));

		assert_eq!(files.path("code/map.dm"), dir.join("code/map.dm"));
		assert_eq!(files.path("map.dm"), dir.join("_std/map.dm"));
		assert_eq!(files.path("macros/dir.dm"), dir.join("_std/macros/dir.dm"));
		assert_eq!(files.path("_std/__std.dme"), dir.join("_std/__std.dme"));
		assert_eq!(files.path("code\\gone.dm"), dir.join("code/gone.dm"));
		assert_eq!(files.path("code/sub/../map.dm"), dir.join("code/map.dm"));
		assert_eq!(files.path("/build/somewhere/world.dme"), dir.join("world.dme"));
		assert_eq!(files.path("C:\\build\\world.dme"), dir.join("world.dme"));
		assert_eq!(files.path("/build/somewhere/other.dme"), Path::new("/build/somewhere/other.dme"));

		let name = |files: &mut WorldFiles, path: &str| files.name(&dir.join(path).to_string_lossy());
		assert_eq!(name(&mut files, "code/map.dm"), "code/map.dm");
		assert_eq!(name(&mut files, "_std/map.dm"), "map.dm");
		assert_eq!(name(&mut files, "_std/macros/dir.dm"), "macros/dir.dm");
		assert_eq!(name(&mut files, "_std/__std.dme"), "_std/__std.dme");
		assert_eq!(files.name("/elsewhere/code/map.dm"), "/elsewhere/code/map.dm");

		assert_eq!(files.name(&dir.join("code/../code/map.dm").to_string_lossy()), "code/map.dm");

		let mut none = WorldFiles::default();
		assert_eq!(none.path("code\\map.dm"), Path::new("code/map.dm"));
		assert_eq!(none.name("/elsewhere/code/map.dm"), "/elsewhere/code/map.dm");
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn files_included_from_outside_the_project() {
		let dir = std::env::temp_dir().join(format!("auxtools-world-files-outside-{}", std::process::id()));
		let _ = std::fs::remove_dir_all(&dir);
		let write = |path: &str, text: &str| {
			let path = dir.join(path);
			std::fs::create_dir_all(path.parent().unwrap()).unwrap();
			std::fs::write(path, text).unwrap();
		};
		write(
			"game/world.dme",
			"#include \"code\\map.dm\"\n#include \"..\\testing\\testing.dm\"\n#include \"sub\\sub.dme\"\n"
		);
		write("game/code/map.dm", "");
		write("testing/testing.dm", "");
		write("game/sub/sub.dme", "#include \"..\\..\\shared\\tools.dm\"\n");
		write("shared/tools.dm", "");
		write("shared/unlisted.dm", "");
		let mut files = WorldFiles::new(Some(dir.join("game/world.dme")));
		let at = |path: &str| dir.join(path).to_string_lossy().into_owned();

		assert_eq!(files.name(&at("testing/testing.dm")), "../testing/testing.dm");
		assert_eq!(files.name(&at("game/../testing/testing.dm")), "../testing/testing.dm");
		assert_eq!(files.name(&at("shared/tools.dm")), "../../shared/tools.dm");
		assert_eq!(files.name(&at("shared/unlisted.dm")), at("shared/unlisted.dm"));
		assert_eq!(files.name(&at("game/code/map.dm")), "code/map.dm");

		assert_eq!(files.path("../testing/testing.dm"), dir.join("testing/testing.dm"));
		assert_eq!(files.path("..\\testing\\testing.dm"), dir.join("testing/testing.dm"));
		assert_eq!(files.path("../../shared/tools.dm"), dir.join("shared/tools.dm"));
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn relative_paths_between_absolute_ones() {
		let at = |path: &str| PathBuf::from(format!("{}{}", if cfg!(windows) { "C:\\" } else { "/" }, path));
		let relative = |root: &Path, path: &Path| relative_to(root, path);
		assert_eq!(relative(&at("a/game"), &at("a/testing/t.dm")), Some(PathBuf::from("../testing/t.dm")));
		assert_eq!(
			relative(&at("a/game"), &at("a/game/../testing/t.dm")),
			Some(PathBuf::from("../testing/t.dm"))
		);
		assert_eq!(relative(&at("a/game"), &at("a/game/code/t.dm")), Some(PathBuf::from("code/t.dm")));
		assert_eq!(relative(&at("a/b/game"), &at("c/t.dm")), Some(PathBuf::from("../../../c/t.dm")));
		assert_eq!(relative(&at("a/game"), Path::new("testing/t.dm")), None);
		assert_eq!(relative(Path::new("game"), &at("a/testing/t.dm")), None);
		#[cfg(windows)]
		assert_eq!(relative(Path::new("C:\\a\\game"), Path::new("D:\\a\\testing\\t.dm")), None);
	}
}
