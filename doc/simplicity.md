# Simplicity Smart-Contract Primitives (Internal Rust)

This document describes the **internal** Simplicity covenant APIs in `rust/src/contracts/simplicity/`. These primitives support building stateful Simplicity contracts (P2PK, lending, DEX, etc.) inside this repo. They are **not** exposed via `package:lwk/lwk.dart` — only Rust tests and future contract modules use them directly.

## Overview

| Primitive | Purpose |
|-----------|---------|
| `SimplicityProgram` | Compile `.simf` sources; sighash, finalize, run, P2PK sign |
| `StateTaprootBuilder` / `StateTaprootSpendInfo` | Stateful Taproot addresses (program + data leaves) |
| `SimplicityRunResult` | Program/witness byte split from an offline `run` |
| `lwk_wollet::ExternalUtxo` | Describe covenant UTXOs not tracked by the LWK wallet |
| `elements::pset::*` | PSET / transaction construction (upstream) |

Typed program parameters and witnesses use upstream `simplicityhl::{Arguments, WitnessValues, Value}` directly (no local wrappers).

## 1. Compiling a Program

Example P2PK program (same source as `integration_test.rs`):

```
fn main() {
    jet::bip_0340_verify(
        (param::ALICE_PUBLIC_KEY, jet::sig_all_hash()),
        witness::ALICE_SIGNATURE)
}
```

```rust
use std::collections::HashMap;
use lwk_simplicity::simplicityhl::{Arguments, Value};
use lwk_simplicity::simplicityhl::num::U256;
use lwk_simplicity::simplicityhl::parse::ParseFromStr;
use lwk_simplicity::simplicityhl::str::WitnessName;
use lwk_simplicity::simplicityhl::value::ValueConstructible;

let mut map = HashMap::new();
map.insert(
    WitnessName::parse_from_str("ALICE_PUBLIC_KEY").unwrap(),
    Value::u256(U256::from_byte_array(pubkey_bytes)),
);
let args = Arguments::from(map);

let program = SimplicityProgram::load_with_arguments(P2PK_SOURCE.to_string(), &args)?;
let cmr = program.cmr();
```

## 2. Building Stateful Taproot Addresses

For covenants that store on-chain state, build a Taproot tree with a **program leaf** and optional **data leaves** (32-byte storage slots):

```rust
let spend = StateTaprootBuilder::new()
    .add_simplicity_leaf(1, program.cmr())?
    .add_data_leaf(1, state_bytes)? // 32 bytes
    .finalize(&internal_key)?;

let script = spend.script_pubkey();
// IMPORTANT: use output_key for signing and finalization
let program_public_key = spend.output_key();
```

For a simple single-leaf P2PK covenant (no state slot), use depth `0` with only a program leaf, or `SimplicityProgram::create_p2tr_address()`.

### Unspendable internal key

Many protocols use the standard unspendable internal key (BIP-341 NUMS point):

```
50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
```

Available as `UNSPENDABLE_TAPROOT_PUBKEY`.

## 3. Which Public Key to Use

Taproot addresses have two related keys:

| Key | Meaning |
|-----|---------|
| **Internal key** | The key passed to `StateTaprootBuilder::finalize` (or to `create_p2tr_address`). Often a NUMS / unspendable point when only script-path spends are intended. |
| **Output key** (`Q`) | The tweaked key committed in the on-chain `scriptPubKey` after the Merkle tree tweak. |

| Scenario | Key for `get_sighash_all` / `finalize_transaction` |
|----------|-----------------------------------------------------|
| Simple P2TR (`create_p2tr_address`) | Internal x-only key passed when creating the address |
| Stateful taproot (`StateTaprootBuilder`) | **`StateTaprootSpendInfo::output_key()`** — not the internal key |

Using the wrong key produces invalid signatures or witness rejection.

## 4. Building and Spending a Covenant Transaction

Covenant outputs are typically **not tracked by the LWK wallet**. Build the spend with Elements PSET types (see `integration_test.rs`):

```rust
let mut input = Input::from_prevout(outpoint);
input.witness_utxo = Some(funding_out);

let mut pset = PartiallySignedTransaction::new_v2();
pset.add_input(input);
pset.add_output(recipient);
pset.add_output(change);
let tx_bytes = elements::encode::serialize(&pset.extract_tx()?);
```

For P2PK covenants, sign with the program key:

```rust
let keypair = derive_keypair(mnemonic, network, path)?;
let signature = program.create_p2pk_signature(
    &keypair,
    tx_bytes.clone(),
    utxos_hex.clone(),
    0,
    network,
)?;
```

## 5. Finalizing vs Running

**`finalize_transaction`** attaches the Simplicity witness stack to a transaction input and returns serialized tx bytes. The resulting input witness has **4 stack elements**:

```
[witness_bytes, program_bytes, cmr_bytes, control_block]
```

**`run`** executes the program offline in a transaction environment for verification / debugging. It does **not** mutate the transaction. Use `SimplicityRunResult` (and `simplicity_control_block`) to inspect or rebuild the witness stack.

```rust
let mut map = HashMap::new();
map.insert(
    WitnessName::parse_from_str("ALICE_SIGNATURE").unwrap(),
    Value::byte_array(signature),
);
let witness = WitnessValues::from(map);

let finalized = program.finalize_transaction(
    tx_bytes.clone(),
    program_public_key.to_string(),
    utxos_hex.clone(),
    0,
    &witness,
    network,
)?;

let run_result = program.run(
    tx_bytes,
    program_public_key.to_string(),
    utxos_hex,
    0,
    &witness,
    network,
)?;
```

## Example: Minimal P2PK Flow

```rust
// 1. Derive key and compile the P2PK program above with ALICE_PUBLIC_KEY
let keypair = derive_keypair(mnemonic, LiquidNetwork::Testnet, "m/86'/1'/0'/0/0")?;
let internal_key = keypair.x_only_public_key().0;
let program = SimplicityProgram::load_with_arguments(P2PK_SOURCE.into(), &p2pk_args(&internal_key))?;

// 2. Build a single-leaf Taproot address and fund its script_pubkey
let spend = StateTaprootBuilder::new()
    .add_simplicity_leaf(0, program.cmr())?
    .finalize(&internal_key)?;

// 3. Build a spend PSET with elements::pset, extract tx bytes
// 4. Sign and finalize
let signature = program.create_p2pk_signature(&keypair, tx_bytes.clone(), utxos_hex.clone(), 0, network)?;
let finalized = program.finalize_transaction(tx_bytes, internal_key.to_string(), utxos_hex, 0, &witness, network)?;
```

See `rust/src/contracts/simplicity/integration_test.rs` for a complete offline Rust test of this flow.
