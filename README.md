# Solana Transfer Extraction — CLR3 Ventures

Rust Substreams extractor for Solana blocks **385870151 through 385870156** (inclusive).

## Features
- Native SOL System Program `Transfer` and `TransferWithSeed` (lamports)
- SPL Token and Token-2022 `Transfer` and `TransferChecked` (raw token base units)
- Both top-level and inner (CPI) instructions
- Failed transactions filtered; malformed or unrelated instructions safely ignored
- Resolves account keys including lookup tables using the Substreams Solana instruction view
- One protobuf `Transfers` envelope per block, containing all individual transfers

**Address semantics:** SPL `from` and `to` refer to token-account addresses rather than necessarily wallet-owner addresses. Token amounts are raw integer units.

## Build and test

Install Rust, the wasm32 target and the Substreams CLI:

```sh
rustup target add wasm32-unknown-unknown
cargo test
cargo build --release --target wasm32-unknown-unknown
substreams pack substreams.yaml
substreams info substreams.yaml
```

Expected package name: `solana_transfer_extractor-v0.1.0.spkg`.

## Run the requested six blocks

Configure authentication with your Solana Substreams provider (for example, `substreams auth`). Do not commit secrets.

```sh
substreams run -e mainnet.sol.streamingfast.io:443 substreams.yaml map_transfers -s 385870151 -t 385870157
```

The stop block is **exclusive**; 385870157 is necessary to include block 385870156. Substitute your provider's Solana endpoint if different.

The map output is a protobuf envelope per block. The `scripts/export_jsonl.py` utility converts **structured JSON envelopes** to one JSON object per transfer in the required exact shape:

```json
{"from":"Eq1c3uNugn3FRpJ99K2q1ZyQrora3jntUqvkMnGroKkB","to":"4XpSNfoNfireSgdg8hY6Ng3dWrHjewZhmCancypjbcCN","amount":325253}
```

The exporter requires a structured JSON export from a compatible sink or CLI mode; human-readable terminal logs are not valid input.

## Tests and verification

Unit tests cover transfer variants, checked transfers, missing accounts, truncated instructions, unsupported program IDs, zero and maximum amounts. GitHub Actions runs unit tests and builds WebAssembly.

**Verification status:** A real provider stream, compiled SPKG, and final six-block output still require successful execution; no blockchain output counts are claimed without it.

[Assignment specification](https://github.com/CLR3-Ventures/substreams-assignment-1/blob/main/.unifiedhiring/task.md).
