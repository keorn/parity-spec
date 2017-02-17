Converts Geth proof of work chain spec e.g.
```
{
	"nonce": "0x0000000000000042",
	"timestamp": "0x0",
	"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
	"extraData": "0x0",
	"gasLimit": "0x8000000",
	"difficulty": "0x400",
	"mixhash": "0x0000000000000000000000000000000000000000000000000000000000000000",
	"coinbase": "0x3333333333333333333333333333333333333333",
	"alloc": {
		"0000000000000000000000000000000000000001": {"balance": "1"},
		"0000000000000000000000000000000000000002": {"balance": "1"},
		"0000000000000000000000000000000000000003": {"balance": "1"},
		"0000000000000000000000000000000000000004": {"balance": "1"},
		"dbdbdb2cbd23b783741e8d7fcf51e459b497e4a6": {"balance": "1606938044258990275541962092341162602522202993782792835301376"},
		"e4157b34ea9615cfbde6b4fda419828124b70c78": {"balance": "1606938044258990275541962092341162602522202993782792835301376"}
	}
}
```
to Parity one, which should enable the two to connect to each other.
For best effects use the "config" field in Geth spec as seen in the [Ropsten testnet spec](https://dl.dropboxusercontent.com/u/4270001/testnet_genesis.json).

## Usage
```
git clone https://github.com/keorn/parity-spec.git
cd parity-spec
cargo run -- geth-spec.json
```

The resulting spec can be then used in Parity with `parity --chain parity-spec.json`.
