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
		let (block_hash, block_number) = line.split_once(',').unwrap();
		let block_hash = block_hash.to_string();
		let block_number = block_number.parse::<u64>().unwrap();

		block_hashes.push((block_number, block_hash));
	}

	block_hashes.sort_by_key(|(n, _)| *n);

	let mut block_hashes_file = File::create("block-hashes.data").unwrap();
	let mut mmr_file = File::create("mmr.data").unwrap();
	let mut mmr_size = 0;
	let mut mem_mmr = <MemMMR<Hash, Hasher>>::new(mmr_size, MemStore::default());

	for (block_number, hex_block_hash) in block_hashes {
		writeln!(block_hashes_file, "{},{}", block_number, hex_block_hash).unwrap();

		let block_hash = array_bytes::hex_into_unchecked(&hex_block_hash);

		mem_mmr.push(block_hash).unwrap();

		for pos in mmr_size..mem_mmr.mmr_size {
			let node_hash = mem_mmr.store().get_elem(pos).unwrap().unwrap();

			writeln!(
				mmr_file,
				"{},{}",
				pos,
				array_bytes::bytes2hex("0x", node_hash.0)
			)
			.unwrap();
		}

		mmr_size = mem_mmr.mmr_size;
	}

	block_hashes_file.sync_all().unwrap();
	mmr_file.sync_all().unwrap();
}
