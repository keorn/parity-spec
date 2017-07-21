#![allow(non_snake_case)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::env;
use std::path::PathBuf;
use std::collections::BTreeMap as Map;
use std::io::prelude::*;
use std::io;
use std::fs::File;

#[derive(Deserialize, Debug)]
struct GethAccount {
	balance: Option<String>,
	wei: Option<String>
}

#[derive(Deserialize)]
struct GethConfig {
	chainId: Option<u64>,
	homesteadBlock: u64,
	eip150Block: Option<u64>,
	eip155Block: u64,
	eip158Block: u64,
	eip160Block: u64
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
	alloc: Map<String, GethAccount>,
	config: Option<GethConfig>
}

#[derive(Serialize)]
struct ParityEthash {
	gasLimitBoundDivisor: String,
	minimumDifficulty: String,
	difficultyBoundDivisor: String,
	durationLimit: String,
	blockReward: String,
	registrar: String,
	homesteadTransition: u64,
	eip150Transition: u64,
	eip155Transition: u64,
	eip160Transition: u64,
	eip161abcTransition: u64,
	eip161dTransition: u64
}

#[derive(Serialize)]
struct ParityParams {
	accountStartNonce: String,
	maximumExtraDataSize: String,
	minGasLimit: String,
	networkID: u64,
	eip98Transition: u64,
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
	#[serde(skip_serializing_if="Option::is_none")]
	nonce: Option<String>,
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

fn ask_network_id() -> u64 {
	/// Ask for additional info which Geth does not include in the file.
	println!("Please enter Geth's --networkid option value (required):");
	let mut network_id = String::new();
	io::stdin().read_line(&mut network_id).expect("Could not read.");
	network_id.trim().parse::<u64>().expect("Could not parse")
}

fn ask_start_nonce() -> Option<String> {
	println!("Please enter the start nonce (used for replay protection, default 0):");
	let mut start_nonce = String::new();
	io::stdin().read_line(&mut start_nonce).expect("Could not read.");
	start_nonce.trim().parse::<u64>().map(|n| format!("0x{:X}", n)).ok()
}

fn builtin_from_address(address: String) -> Option<ParityBuiltin> {
	let mut address = address;
	if address.starts_with("0x") {
		address = address[2..].into();
	}
	match u64::from_str_radix(&address, 16) {
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
	}
}

fn translate(geth_spec: GethSpec) -> ParitySpec {
	let geth_config = geth_spec.config.unwrap_or_else(|| GethConfig {
		chainId: None,
		homesteadBlock: 9223372036854775807,
		eip150Block: None,
		eip155Block: 9223372036854775807,
		eip158Block: 9223372036854775807,
		eip160Block: 9223372036854775807,
	});

	/// Construct Parity chain spec.
	let parity_ethash = ParityEthash {
		gasLimitBoundDivisor: "0x400".into(),
		minimumDifficulty: "0x20000".into(),
		difficultyBoundDivisor: "0x800".into(),
		durationLimit: "0xd".into(),
		blockReward: "0x4563918244F40000".into(),
		registrar: "0x81a4b044831c4f12ba601adb9274516939e9b8a2".into(),
		homesteadTransition: geth_config.homesteadBlock,
		eip150Transition: geth_config.eip150Block.unwrap_or(9223372036854775807),
		eip155Transition: geth_config.eip155Block,
		eip160Transition: geth_config.eip160Block,
		eip161abcTransition: geth_config.eip160Block,
		eip161dTransition: geth_config.eip160Block,
	};
	let mut engine = Map::new();
	engine.insert("Ethash".into(), Map::new());
	engine.get_mut("Ethash").unwrap().insert("params".into(), parity_ethash);

	let start_nonce = ask_start_nonce();

	let parity_params = ParityParams {
		accountStartNonce: start_nonce.clone().unwrap_or("0x0".into()),
		maximumExtraDataSize: "0x20".into(),
		minGasLimit: "0x1388".into(),
		networkID: geth_config.chainId.unwrap_or_else(ask_network_id),
		eip98Transition: 9223372036854775807,
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
		gasLimit: geth_spec.gasLimit,
	};

	let parity_accounts = geth_spec.alloc.into_iter().map(|(address, acc)| {
		(address.clone(), ParityAccount {
			balance: acc.balance.clone().unwrap_or_else(|| acc.wei.clone().expect("Each account has to have balance.")),
			nonce: start_nonce.clone(),
			builtin: builtin_from_address(address.clone())
		})}).collect();

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
	let parity_spec = translate(geth_spec);
	let serialized = serde_json::to_string_pretty(&parity_spec).expect("Could not serialize");
	println!("{}", serialized);
}
