use byond_scan::VersionRange;

pub const FIRST_515: u32 = 1637;

/// First build (`515.1641`) where type table entries store the typepath value,
/// adding 8 bytes (`0x94` bytes on Linux, `0x98` on Windows) and requiring new
/// types to be registered for `text2path`.
pub const TYPEPATHS: u32 = 1641;

/// First Linux build (`515.1644`) compiled without CPU frame pointers.
///
/// Paired globals swap order, `get_length` returns a BYOND value instead of a
/// plain count, misc interning copies the caller's words instead of taking
/// ownership of the buffer, and the interpreter holds the pointer to the DM
/// proc's frame in `ESI` instead of `EDI`.
pub const LINUX_FRAMELESS: u32 = 1644;

/// First supported BYOND 516 build (`516.1648`).
///
/// Introduces `/alist` support and adds `icon_w` and `icon_z` ahead of an
/// appearance's mouse pointers, shifting those pointers by 4 bytes.
pub const FIRST_516: u32 = 1648;

/// First Linux build (`516.1674`) that passes `remove_from_list`'s list
/// argument on the stack instead of in `eax:edx`.
pub const LINUX_REMOVE_FROM_LIST_ON_STACK: u32 = 1674;

/// First build (`516.1686`) where typepath registration checks for a missing
/// type, changing the function prologue matched by `register_typepath`.
pub const TYPEPATH_NONE_CHECK: u32 = 1686;

/// Newest verified BYOND build (`516.1689`).
///
/// Newer builds still attempt symbol recipes, but resource reload returns an
/// error and the `text2path` cache is left untouched.
pub const NEWEST_CHECKED: u32 = 1689;

/// Supported BYOND major versions paired with each series's minimum build
/// (`515.1637` and `516.1648`), consulted by `registry::accepts`.
pub const SUPPORTED: &[(u32, u32)] = &[(515, FIRST_515), (516, FIRST_516)];

/// Returns an open-ended version range from `first` onward with no upper bound.
///
/// Builds newer than [`NEWEST_CHECKED`] match recipes using this range.
pub const fn since(first: u32) -> VersionRange {
	VersionRange::from(first)
}

/// Returns a half-open version range covering builds from `first` up to
/// `next - 1`.
pub const fn until(first: u32, next: u32) -> VersionRange {
	VersionRange { min: first, max: next - 1 }
}
