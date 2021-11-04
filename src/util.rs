// --- std ---
use std::{
	fs::File,
	io::{Read, Write},
};
// --- crates.io ---
use mmr::{
	util::{MemMMR, MemStore},
	MMRStore,
};
use parity_scale_codec::Encode;
// --- mmr-verifier ---
use crate::hash::{Hash, Hasher};

pub fn offchain_key(pos: u64) -> String {
	const PREFIX: &[u8] = b"header-mmr-";

	let offchain_key = array_bytes::bytes2hex("0x", (PREFIX, pos).encode());

	// dbg!((pos, &offchain_key));

	offchain_key
}

pub fn build_mmr_from_snap() {
	let mut block_hashes_data = "".into();
	let mut read = File::open("block-hashes.rawdata").unwrap();

	read.read_to_string(&mut block_hashes_data).unwrap();

	let mut block_hashes = vec![];

	for line in block_hashes_data.lines() {
		let (block_number, block_hash) = line.split_once(',').unwrap();
		let block_number = block_number.parse::<u64>().unwrap();
		let block_hash = block_hash.to_string();

		block_hashes.push((block_number, block_hash));
	}

	block_hashes.sort_by_key(|(n, _)| *n);

	let mut mem_mmr = <MemMMR<Hash, Hasher>>::new(0, MemStore::default());
	let mut block_hashes_store = vec![];

	for (i, (block_number, hex_block_hash)) in block_hashes.into_iter().enumerate() {
		if i as u64 != block_number {
			panic!("{}", i);
		}

		block_hashes_store
			.extend_from_slice(format!("{},{}\n", block_number, hex_block_hash).as_bytes());

		let block_hash = array_bytes::hex_into_unchecked(&hex_block_hash);

		mem_mmr.push(block_hash).unwrap();
	}

	let mut mmr_store = vec![];

	for pos in 0..mem_mmr.mmr_size {
		let node_hash = mem_mmr.store().get_elem(pos).unwrap().unwrap();

		mmr_store.extend_from_slice(
			format!("{},{}\n", pos, array_bytes::bytes2hex("0x", node_hash.0)).as_bytes(),
		);
	}

	let mut block_hashes_file = File::create("block-hashes.data").unwrap();

	block_hashes_file.write_all(&block_hashes_store).unwrap();
	block_hashes_file.sync_all().unwrap();

	let mut mmr_file = File::create("mmr.data").unwrap();

	mmr_file.write_all(&mmr_store).unwrap();
	mmr_file.sync_all().unwrap();
}
