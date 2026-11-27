pub const LONGEST_KEY: usize = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unusable {
	NoLetterOrDigit,
	TooLong,
	Character(char)
}

impl std::fmt::Display for Unusable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Unusable::NoLetterOrDigit => write!(f, "a key must contain at least one letter or digit"),
			Unusable::TooLong => write!(f, "a key must be at most {} characters long", LONGEST_KEY),
			Unusable::Character(refused) => write!(f, "a key cannot contain {:?}", refused)
		}
	}
}

pub fn unusable(key: &str) -> Option<Unusable> {
	if let Some(refused) = key.chars().find(|c| !c.is_ascii() || c.is_ascii_control() || "<>\"\\|".contains(*c)) {
		return Some(Unusable::Character(refused));
	}
	if key.len() > LONGEST_KEY {
		return Some(Unusable::TooLong);
	}
	if !key.chars().any(|c| c.is_ascii_alphanumeric()) {
		return Some(Unusable::NoLetterOrDigit);
	}
	None
}

#[cfg(test)]
mod tests {
	use super::{unusable, Unusable};

	#[test]
	fn keys_byond_takes_and_keys_it_does_not() {
		for key in ["Ada Lovelace", "ada", "A", "Dev %u", "x-1_2.3", &"k".repeat(30)] {
			assert_eq!(unusable(key), None, "{}", key);
		}
		assert_eq!(unusable(&"k".repeat(31)), Some(Unusable::TooLong));
		assert_eq!(unusable(""), Some(Unusable::NoLetterOrDigit));
		assert_eq!(unusable(" - "), Some(Unusable::NoLetterOrDigit));
		for (key, refused) in [
			("a<b", '<'),
			("a>b", '>'),
			("a\"b", '"'),
			("a\\b", '\\'),
			("a|male", '|'),
			("a\tb", '\t'),
			("Ada é", 'é')
		] {
			assert_eq!(unusable(key), Some(Unusable::Character(refused)), "{}", key);
		}
	}
}
