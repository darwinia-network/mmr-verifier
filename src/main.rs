// --- std ---
use std::fmt;
// --- crates.io ---
use blake2_rfc::blake2b;
use csv::Reader;
use parity_scale_codec::Encode;
use serde::Deserialize;
use serde_json::Value;
use subrpcer::client::u;
// --- github.com ---
use mmr::{
	helper,
	util::{MemMMR, MemStore},
	Merge,
};

fn offchain_key(pos: u64) -> String {
	const PREFIX: &[u8] = b"header-mmr-";

	let offchain_key = array_bytes::bytes2hex("0x", (PREFIX, pos).encode());

	// dbg!((pos, &offchain_key));

	offchain_key
}

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
pub struct Hash([u8; 32]);
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

#[derive(Debug, Deserialize)]
struct Record {
	block_number: u64,
	parent_mmr_root: String,
	hash: String,
}
impl Record {
	fn read_csv() -> Vec<Self> {
		let mut reader = Reader::from_path("data.csv").unwrap();
		let mut v = reader
			.deserialize::<Record>()
			.filter_map(|r| r.ok())
			.collect::<Vec<_>>();

		v.sort_by_key(|r| r.block_number);

		v
	}
}

fn build_mem_store(start_at: u64) -> MemStore<Hash> {
	let mem_store = MemStore::default();
	let size = mmr::leaf_index_to_mmr_size(start_at);

	dbg!(&size);

	let peaks = helper::get_peaks(size);

	for pos in peaks {
		let k = offchain_key(pos);
		let rpc = subrpcer::rpc(
			0,
			"offchain_localStorageGet",
			serde_json::json!(["PERSISTENT", k]),
		);
		let hash = u::send_rpc("http://107.167.190.243:9977", rpc)
			.unwrap()
			.into_json::<Value>()
			.unwrap()["result"]
			.as_str()
			.unwrap()
			.to_string();

		dbg!((pos, &hash));

		let mut mem_store = mem_store.0.borrow_mut();

		mem_store.insert(pos, array_bytes::hex_into_unchecked(hash));
	}

	mem_store
}

/// > lock happened at 5_689_148
/// >
/// > get all **hash** and **parent mmr root** from 5_680_000
///
/// 1. build empty tree, index up to 5_730_720 with size 11_461_429
/// 2. insert peaks
///    1. get peaks state at 5_680_000
///    2. insert
/// 3. start append/merge process
/// 4. verify if 5_680_001 is correct (corner case)
fn main() {
	// let block_number = 5_730_720;
	// let mmr_size = mmr::leaf_index_to_mmr_size(block_number - 1);

	// assert_eq!(mmr_size, 1_1461_429);

	let start_at = 4_999_999;
	let mem_store = build_mem_store(start_at);
	let mut mem_mmr = <MemMMR<Hash, Hasher>>::new(mmr::leaf_index_to_mmr_size(start_at), mem_store);
	let records = Record::read_csv();

	for Record {
		block_number,
		parent_mmr_root: expected_root,
		hash,
	} in records
	{
		let root = array_bytes::bytes2hex("", mem_mmr.get_root().unwrap().0);

		dbg!((block_number, &expected_root, &root));
		assert_eq!(expected_root, root);

		mem_mmr.push(array_bytes::hex_into_unchecked(hash)).unwrap();
	}
}
