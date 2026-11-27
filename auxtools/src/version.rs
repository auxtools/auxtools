pub static mut BYOND_VERSION_MAJOR: u32 = 0;
pub static mut BYOND_VERSION_MINOR: u32 = 0;

pub(crate) fn init() -> Result<(), String> {
	let (major, minor) = byond_scan::version().map_err(|err| format!("could not read the BYOND version ({err})"))?;

	unsafe {
		BYOND_VERSION_MAJOR = major;
		BYOND_VERSION_MINOR = minor;
	}

	Ok(())
}

/// Returns the running BYOND `(version, build)` pair, such as `(516, 1689)`.
///
/// Returns `(0, 0)` until the library's first initialization has read them. Use
/// `registry::byond_build` to query the build before initialization.
pub fn get() -> (u32, u32) {
	unsafe { (BYOND_VERSION_MAJOR, BYOND_VERSION_MINOR) }
}
