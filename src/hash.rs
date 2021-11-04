// --- std ---
use std::fmt;
// --- crates.io ---
use blake2_rfc::blake2b;
// --- github.com ---
use mmr::Merge;

pub struct Hasher;
impl Merge for Hasher {
	type Item = Hash;

	fn merge(lhs: &Self::Item, rhs: &Self::Item) -> Self::Item {
		pub fn hash(data: &[u8]) -> [u8; 32] {
			array_bytes::dyn2array!(blake2b::blake2b(32, &[], data).as_bytes(), 32)
		}

		let mut data = vec![];

		data.extend_from_slice(&lhs.0);
		data.extend_from_slice(&rhs.0);

		Hash(hash(&data))
	}
}

#[derive(Clone, PartialEq)]
pub struct Hash(pub [u8; 32]);
impl From<[u8; 32]> for Hash {
	fn from(bytes: [u8; 32]) -> Self {
		Self(bytes)
	}
}
impl fmt::Display for Hash {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", array_bytes::bytes2hex("0x", self.0))
	}
}
impl fmt::Debug for Hash {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		<Self as fmt::Display>::fmt(&self, f)
	}
}
