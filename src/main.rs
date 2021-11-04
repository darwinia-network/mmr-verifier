mod hash;
mod rpc;
mod util;

// --- std ---
use std::{
	collections::VecDeque,
	fs::File,
	io::{Read, Write},
};
// --- crates.io ---
use serde_json::Value;
use subrpcer::chain;
use tungstenite::Message;
// --- github.com ---
use mmr::{util::MemMMR, MMRStore};
// --- mmr-verifier ---
use hash::{Hash, Hasher};

// Use a live chain as the source of block hashes.
fn correct_node_hashes_live(
	uri: &str,
	mut file: File,
	mut hashes: VecDeque<String>,
	mut mem_mmr: MemMMR<Hash, Hasher>,
	start_at: u64,
) {
	if let Ok((mut ws, _)) = tungstenite::connect(uri) {
		for block_number in start_at.. {
			if ws
				.write_message(Message::Binary(
					serde_json::to_vec(&chain::get_block_hash(block_number)).unwrap(),
				))
				.is_err()
			{
				return correct_node_hashes_live(uri, file, hashes, mem_mmr, block_number);
			}

			if let Ok(msg) = ws.read_message() {
				let result =
					serde_json::from_slice::<Value>(&msg.into_data()).unwrap()["result"].take();
				let hex = result.as_str().unwrap();
				let hash = array_bytes::hex_into_unchecked(hex);

				hashes.push_back(hex.into());
				mem_mmr.push(hash).unwrap();
			} else {
				return correct_node_hashes_live(uri, file, hashes, mem_mmr, block_number);
			}

			let step: u64 = 1000;

			if block_number % step == 0 {
				let start_block_number = block_number - step;
				let end_block_number = block_number;
				let mut start_pos = mmr::leaf_index_to_mmr_size(start_block_number);

				for block_number in (start_block_number + 1)..=end_block_number {
					let block_hash = hashes.pop_front().unwrap();
					let end_pos = mmr::leaf_index_to_mmr_size(block_number);
					let node_hashes = (start_pos..end_pos)
						.into_iter()
						.map(|pos| {
							let node_hash = mem_mmr.store().get_elem(pos).unwrap().unwrap();

							format!(
								"pos:{}-node_hash:{}",
								pos,
								array_bytes::bytes2hex("0x", node_hash.0)
							)
						})
						.collect::<Vec<_>>()
						.join(",");

					writeln!(
						file,
						"block_number:{}-block_hash:{}-{}",
						block_number, block_hash, node_hashes
					)
					.unwrap();

					start_pos = end_pos;
				}

				dbg!(block_number);
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
			let (pos, hash) = line.split_once(',').unwrap();
			let pos = pos.parse().unwrap();

			(pos, hash)
		})
		.collect::<Vec<_>>();
	let mut checklist = File::create("checklist.data").unwrap();

	'l: loop {
		if let Ok((mut ws, _)) = tungstenite::connect(uri) {
			while let Some((pos, hash)) = mmr_data.last() {
				if ws
					.write_message(Message::Binary(
						serde_json::to_vec(&rpc::get_node_hash_payload(*pos)).unwrap(),
					))
					.is_err()
				{
					continue 'l;
				}

				if let Ok(msg) = ws.read_message() {
					let node_hash = serde_json::from_slice::<Value>(&msg.into_data()).unwrap()
						["result"]
						.as_str()
						.unwrap()
						.to_string();

					if hash != &node_hash {
						if ws
							.write_message(Message::Binary(
								serde_json::to_vec(&rpc::insert_node_hash_payload(*pos, &hash))
									.unwrap(),
							))
							.is_ok()
						{
							let check = format!("{},{}", pos, hash);

							println!("{}", check);
							writeln!(checklist, "{}", check).unwrap();
						} else {
							continue 'l;
						}
					}
				} else {
					continue 'l;
				}

				if *pos % 100 == 0 {
					println!("process: {}", pos);
				}

				mmr_data.pop().unwrap();
			}

			return;
		}
	}
}

fn main() {
	let uri = "ws://localhost:30000";

	// correct_node_hashes_live(
	// 	uri,
	// 	File::create("mmr.data").unwrap(),
	// 	VecDeque::new(),
	// 	<MemMMR<Hash, Hasher>>::new(0, MemStore::default()),
	// 	0,
	// );
	// correct_node_hashes_snap(uri);

	// util::build_mmr_from_snap();

	correct_node_hashes_snap(uri);
}
