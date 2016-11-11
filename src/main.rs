#![allow(non_snake_case)]
#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use serde_json::Map;
use std::env;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io;
use std::fs::File;

#[derive(Deserialize)]
struct GethAccount {
	balance: Option<String>,
	wei: Option<String>
}

#[derive(Deserialize)]
struct GethSpec {
	nonce: String,
	difficulty: String,
	mixhash: String,
	coinbase: String,
	timestamp: String,
	parentHash: String,
	extraData: String,
	gasLimit: String,
	alloc: Map<String, GethAccount>
}

#[derive(Serialize)]
struct ParityEthash {
	gasLimitBoundDivisor: String,
	minimumDifficulty: String,
	difficultyBoundDivisor: String,
	durationLimit: String,
	blockReward: String
}

#[derive(Serialize)]
struct ParityParams {
	accountStartNonce: String,
	maximumExtraDataSize: String,
	minGasLimit: String,
	networkID: String
}

#[derive(Serialize)]
struct ParitySeal {
	nonce: String,
	mixHash: String
}

#[derive(Serialize)]
struct ParityGenesis {
	seal: Map<String, ParitySeal>,
	difficulty: String,
	author: String,
	timestamp: String,
	parentHash: String,
	extraData: String,
	gasLimit: String
}

fn linear_pricing(base: u64, word: u64) -> Map<String, Map<String, u64>> {
	let mut linear = Map::new();
	linear.insert("base".into(), base);
	linear.insert("word".into(), word);
	let mut pricing = Map::new();
	pricing.insert("linear".into(), linear);
	pricing
}

#[derive(Serialize)]
struct ParityBuiltin {
	name: String,
	pricing: Map<String, Map<String, u64>>
}

#[derive(Serialize)]
struct ParityAccount {
	balance: String,
	nonce: String,
	#[serde(skip_serializing_if="Option::is_none")]
	builtin: Option<ParityBuiltin>
}

#[derive(Serialize)]
struct ParitySpec {
	name: String,
	engine: Map<String, Map<String, ParityEthash>>,
	params: ParityParams,
	genesis: ParityGenesis,
	accounts: Map<String, ParityAccount>
}

fn get_path() -> PathBuf {
	/// Read Geth chain spec.
	let mut args = env::args();
	args.next();
	PathBuf::from(args.next().expect("No file given."))
}

fn read_file(path: PathBuf) -> String {
	let mut f = File::open(path).expect("Could not open the file.");
	let mut s = String::new();
	f.read_to_string(&mut s).expect("Could not read the file.");
	s
}

fn ask() -> (String, String) {
	/// Ask for additional info which Geth does not include in the file.
	println!("Please enter Geth's --networkid option value (default 0):");
	let mut network_id = String::new();
	io::stdin().read_line(&mut network_id).expect("Could not read.");
	let network_id = network_id.trim().parse::<u64>().map(|n| format!("0x{:X}", n)).unwrap_or("0x0".into());

	println!("Please enter the start nonce (used for replay protection, default 0):");
	let mut start_nonce = String::new();
	io::stdin().read_line(&mut start_nonce).expect("Could not read.");
	let start_nonce = start_nonce.trim().parse::<u64>().map(|n| format!("0x{:X}", n)).unwrap_or("0x0".into());

	(network_id, start_nonce)
}

fn translate(geth_spec: GethSpec, network_id: String, start_nonce: String) -> ParitySpec {
	/// Construct Parity chain spec.
	let parity_ethash = ParityEthash {
		gasLimitBoundDivisor: "0x400".into(),
		minimumDifficulty: "0x20000".into(),
		difficultyBoundDivisor: "0x800".into(),
		durationLimit: "0xd".into(),
		blockReward: "0x4563918244F40000".into()
	};
	let mut engine = Map::new();
	engine.insert("Ethash".into(), Map::new());
	engine.get_mut("Ethash").unwrap().insert("params".into(), parity_ethash);

	let parity_params = ParityParams {
		accountStartNonce: start_nonce.clone(),
		maximumExtraDataSize: "0x20".into(),
		minGasLimit: "0x1388".into(),
		networkID: network_id
	};

	let mut parity_seal = Map::new();
	parity_seal.insert("ethereum".into(), ParitySeal {
		nonce: geth_spec.nonce,
		mixHash: geth_spec.mixhash
	});
	let parity_genesis = ParityGenesis {
		seal: parity_seal,
		difficulty: geth_spec.difficulty,
		author: geth_spec.coinbase,
		timestamp: geth_spec.timestamp,
		parentHash: geth_spec.parentHash,
		extraData: geth_spec.extraData,
		gasLimit: geth_spec.gasLimit
	};

	let parity_accounts = geth_spec.alloc.into_iter().map(|(address, acc)|
		(address.clone(), ParityAccount {
			balance: acc.balance.clone().unwrap_or_else(|| acc.wei.clone().expect("Each account has to have balance.")),
			nonce: start_nonce.clone(),
			builtin: match address.parse::<u8>() {
				Ok(1) => Some(ParityBuiltin {
					name: "ecrecover".into(),
					pricing: linear_pricing(3000, 0)
				}),
				Ok(2) => Some(ParityBuiltin {
					name: "sha256".into(),
					pricing: linear_pricing(60, 12)
				}),
				Ok(3) => Some(ParityBuiltin {
					name: "ripemd160".into(),
					pricing: linear_pricing(600, 120)
				}),
				Ok(4) => Some(ParityBuiltin {
					name: "identity".into(),
					pricing: linear_pricing(15, 3)
				}),
				_ => None,
			}})).collect();

	ParitySpec {
		name: "GethTranslation".into(),
		engine: engine,
		params: parity_params,
		genesis: parity_genesis,
		accounts: parity_accounts
	}
}

fn main() {
	let file_str = read_file(get_path());
	let geth_spec = serde_json::from_str(&file_str).expect("Invalid JSON file.");
	let (network_id, start_nonce) = ask();
	let parity_spec = translate(geth_spec, network_id, start_nonce);
	let serialized = serde_json::to_string_pretty(&parity_spec).expect("Could not serialize");
	println!("{}", serialized);
}

#[test]
fn check_translator() {
	let geth_str = read_file(PathBuf::from("example-geth.json"));
	let geth_spec = serde_json::from_str(&geth_str).expect("Invalid JSON file.");
	let parity_spec = translate(geth_spec, "0x0".into(), "0x0".into());
	let serialized = serde_json::to_string_pretty(&parity_spec).expect("Could not serialize");
	println!("{}", serialized);
	assert_eq!(serialized, read_file(PathBuf::from("example-parity.json")));
}
