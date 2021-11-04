mod hash;
mod rpc;
mod util;

// --- std ---
use std::{
	fs::File,
	io::{Read, Write},
};
// --- crates.io ---
use serde_json::Value;
use subrpcer::chain;
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
	let mut mem_mmr = <MemMMR<Hash, Hasher>>::new(0, MemStore::default());
	let mut checklist = File::create("checklist.data").unwrap();
	let mut empty_nodes = File::create("empty-nodes.data").unwrap();

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

				let block_hash = if let Ok(msg) = ws.read_message() {
					if let Some(hash) = serde_json::from_slice::<Value>(&msg.into_data()).unwrap()
						["result"]
						.as_str()
					{
						hash.to_string()
					} else {
						continue 'l;
					};
				} else {
					continue 'l;
				};

				// if let Ok(msg) = ws.read_message() {
				// 	let node_hash = if let Some(hash) =
				// 		serde_json::from_slice::<Value>(&msg.into_data()).unwrap()["result"]
				// 			.as_str()
				// 	{
				// 		hash.to_string()
				// 	} else {
				// 		writeln!(empty_nodes, "{},{}", pos, util::offchain_key(*pos)).unwrap();

				// 		"".into()
				// 	};

				// if node_hash.is_empty() || hash != &node_hash {
				// 		if ws
				// 			.write_message(Message::Binary(
				// 				serde_json::to_vec(&rpc::insert_node_hash_payload(*pos, &hash))
				// 					.unwrap(),
				// 			))
				// 			.is_ok()
				// 		{
				// 			let check = format!("{},{}", pos, hash);

				// 			println!("{}", check);
				// 			writeln!(checklist, "{}", check).unwrap();
				// 		} else {
				// 			continue 'l;
				// 		}
				// 	}
				// } else {
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
			let (pos, hash) = line.split_once(',').unwrap();
			let pos = pos.parse().unwrap();

			(pos, hash)
		})
		.collect::<Vec<_>>();
	let mut checklist = File::create("checklist.data").unwrap();
	let mut empty_nodes = File::create("empty-nodes.data").unwrap();

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
					let node_hash = if let Some(hash) =
						serde_json::from_slice::<Value>(&msg.into_data()).unwrap()["result"]
							.as_str()
					{
						hash.to_string()
					} else {
						writeln!(empty_nodes, "{},{}", pos, util::offchain_key(*pos)).unwrap();

						"".into()
					};

					if node_hash.is_empty() || hash != &node_hash {
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

	// util::build_mmr_from_snap();

	// correct_node_hashes_live(uri);
	correct_node_hashes_snap(uri);
}
