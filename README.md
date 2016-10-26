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
    }
}
```
to Parity one, which should enable the two to connect to each other.

## Usage
Rust compiler is needed, you can get one [here](https://www.rustup.rs/).
```
git clone https://github.com/keorn/parity-spec.git
cd parity-spec
cargo run -- geth-spec.json
```

The resulting spec can be then used in Patity with `parity --chain parity-spec.json`.
