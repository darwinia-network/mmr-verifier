// --- crates.io ---
use serde_json::Value;
use subrpcer::client::u;
// --- mmr-verifier ---
use crate::*;

pub fn get_node_hash_payload(pos: u64) -> Value {
	subrpcer::rpc(
		0,
		"offchain_localStorageGet",
		serde_json::json!(["PERSISTENT", util::offchain_key(pos)]),
	)
}
pub fn get_node_hash(uri: impl AsRef<str>, pos: u64) -> String {
	let uri = uri.as_ref();
	let rpc = get_node_hash_payload(pos);

	loop {
		if let Ok(response) = u::send_rpc(uri, &rpc) {
			let hash = response.into_json::<Value>().unwrap()["result"]
				.as_str()
				.unwrap()
				.to_string();

			// dbg!((pos, &hash));

			return hash;
		}
	}
}

pub fn insert_node_hash_payload(pos: u64, hash: &str) -> Value {
	subrpcer::rpc(
		0,
		"offchain_localStorageSet",
		serde_json::json!(["PERSISTENT", util::offchain_key(pos), hash]),
	)
}
pub fn insert_node_hash(uri: impl AsRef<str>, pos: u64, hash: String) {
	let uri = uri.as_ref();
	let rpc = insert_node_hash_payload(pos, &hash);

	loop {
		if let Ok(response) = u::send_rpc(uri, &rpc) {
			let result = &response.into_json::<Value>().unwrap()["result"];

			dbg!(result);

			break;
		}
	}
}
