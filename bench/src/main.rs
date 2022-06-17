use env_logger::Builder;
use futures::future::join_all;
use std::str::FromStr;

use eigen_trust::{Keypair, LevelFilter, Multiaddr, Node};

const INTERVAL: u64 = 2;
const NUM_CONNECTIONS: usize = 12;

pub fn init_logger() {
	let mut builder = Builder::from_default_env();

	builder
		.filter(None, LevelFilter::Info)
		.format_timestamp(None)
		.init();
}

#[tokio::main]
async fn main() {
	init_logger();

	let mut local_keys = Vec::new();
	let mut local_addresses = Vec::new();
	let mut bootstrap_nodes = Vec::new();
	let starting_port = 58400;

	for i in 0..(NUM_CONNECTIONS + 1) {
		let local_key = Keypair::generate_ed25519();
		let peer_id = local_key.public().to_peer_id();
		let addr = format!("/ip4/127.0.0.1/tcp/{}", starting_port + i);
		let local_address = Multiaddr::from_str(&addr).unwrap();

		local_keys.push(local_key);
		local_addresses.push(local_address.clone());
		bootstrap_nodes.push((peer_id, local_address));
	}

	let mut tasks = Vec::new();
	for i in 0..(NUM_CONNECTIONS + 1) {
		let local_key = local_keys[i].clone();
		let local_address = local_addresses[i].clone();
		let bootstrap_nodes = bootstrap_nodes.clone();

		let join_handle = tokio::spawn(async move {
			let mut node =
				Node::new(local_key, local_address, bootstrap_nodes.clone(), INTERVAL)
					.unwrap();

			let peer = node.get_swarm_mut().behaviour_mut().get_peer_mut();
			for (peer_id, ..) in bootstrap_nodes {
				peer.set_score(peer_id, 5);
			}

			node.main_loop(Some(10)).await.unwrap();
		});
		tasks.push(join_handle);
	}

	let _ = join_all(tasks)
		.await
		.iter()
		.map(|r| r.as_ref().unwrap())
		.collect::<Vec<_>>();
	println!("Done");
}
