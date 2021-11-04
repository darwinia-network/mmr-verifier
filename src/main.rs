#![allow(dead_code)]

mod hash;
mod util;

// --- std ---
use std::{
	fs::File,
	io::{Read, Write},
	mem,
};
// --- crates.io ---
use serde_json::Value;
use subrpcer::{chain, offchain};
use tungstenite::Message;
// --- github.com ---
use mmr::{
	util::{MemMMR, MemStore},
	MMRStore,
};
// --- mmr-verifier ---
use hash::{Hash, Hasher};

// Use a live chain as the source of block hashes.
fn correct_node_hashes_live(uri: &str) {
	let mut mmr_size = 0;
	let mut mem_mmr = <MemMMR<Hash, Hasher>>::new(mmr_size, MemStore::default());
	let mut block_hashes = vec![];
	let mut block_hashes_file = File::create("block-hashes.livedata").unwrap();
	let mut mmr_file = File::create("mmr.livedata").unwrap();

	'l: loop {
		if let Ok((mut ws, _)) = tungstenite::connect(uri) {
			for block_number in 0u64.. {
				if ws
					.write_message(Message::Binary(
						serde_json::to_vec(&chain::get_block_hash(block_number)).unwrap(),
					))
					.is_err()
				{
					continue 'l;
				}

				if let Ok(msg) = ws.read_message() {
					if let Some(block_hash) = serde_json::from_slice::<Value>(&msg.into_data())
						.unwrap()["result"]
						.as_str()
					{
						mem_mmr
							.push(array_bytes::hex_into_unchecked(block_hash))
							.unwrap();
						block_hashes.extend_from_slice(
							format!("{},{}\n", block_number, block_hash).as_bytes(),
						);
					} else {
						continue 'l;
					};
				} else {
					continue 'l;
				};

				if block_number % 100 == 0 {
					block_hashes_file
						.write_all(&mem::take(&mut block_hashes))
						.unwrap();
					mmr_file
						.write_all(
							(mmr_size..mem_mmr.mmr_size)
								.into_iter()
								.map(|pos| {
									format!(
										"{},{}\n",
										pos,
										array_bytes::bytes2hex(
											"0x",
											mem_mmr.store().get_elem(pos).unwrap().unwrap().0,
										)
									)
									.into_bytes()
								})
								.flatten()
								.collect::<Vec<_>>()
								.as_slice(),
						)
						.unwrap();

					mmr_size = mem_mmr.mmr_size;

					println!("process: {}, {}", block_number, mmr_size);
				}

				// if ws
				// 	.write_message(Message::Binary(
				// 		serde_json::to_vec(&rpc::insert_node_hash_payload(*pos, &block_hash)).unwrap(),
				// 	))
				// 	.is_err()
				// {
				// 	continue 'l;
				// }
			}
		}
	}
}

// Use a state snapshot as the source of block hashes.
fn correct_node_hashes_snap(uri: &str) {
	let mut mmr_data = "".into();
	let mut read = File::open("mmr.data").unwrap();

	read.read_to_string(&mut mmr_data).unwrap();

	let mut mmr_data = mmr_data
		.lines()
		.map(|line| {
			let (pos, node_hash) = line.split_once(',').unwrap();
			let pos = pos.parse().unwrap();

			(pos, node_hash)
		})
		.collect::<Vec<_>>();

	'l: loop {
		if let Ok((mut ws, _)) = tungstenite::connect(uri) {
			while let Some((pos, node_hash)) = mmr_data.last() {
				if ws
					.write_message(Message::Binary(
						serde_json::to_vec(&offchain::local_storage_set(
							"PERSISTENT",
							util::offchain_key(*pos),
							&node_hash,
						))
						.unwrap(),
					))
					.is_err()
				{
					continue 'l;
				}

				if *pos % 1000 == 0 {
					println!("process: {}", pos);
				}

				mmr_data.pop().unwrap();
			}

			return;
		}
	}
}

fn main() {
	let uri = "wss://rpc-alt.darwinia.network";

	// util::build_mmr_from_snap();

	// correct_node_hashes_live(uri);
	correct_node_hashes_snap(uri);
}
