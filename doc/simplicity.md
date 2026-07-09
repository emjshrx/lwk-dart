# Simplicity Smart-Contract Primitives (Internal Rust)

This document describes the **internal** generic Simplicity covenant APIs in `rust/src/contracts/`. These primitives support building stateful Simplicity contracts (P2PK, lending, DEX, etc.) inside this repo. They are **not** exposed via `package:lwk/lwk.dart` — only Rust tests and future contract modules use them directly.

Dart examples below illustrate the intended API shape when a thin public contract layer is added later.

## Overview

| Primitive | Purpose |
|-----------|---------|
| `SimplicityProgram` | Compile and run `.simf` sources |
| `SimplicityArguments` / `SimplicityWitnessValues` | Typed program parameters and witnesses |
| `SimplicityTypedValue` / `SimplicityType` | Build and parse typed Simplicity values |
| `StateTaprootBuilder` | Stateful Taproot addresses (program + data leaves) |
| `ExternalUtxo` | Describe covenant UTXOs not owned by the wallet |
| `PsetBuilder` | PSET / transaction construction |

## 1. Compiling a Program

```dart
final pubkey = await simplicityDeriveXonlyPubkey(
  signer: signer,
  derivationPath: "m/86'/1'/0'/0/0",
);

final args = SimplicityArguments()
    .addValue(
      name: 'ALICE_PUBLIC_KEY',
      value: SimplicityTypedValue.u256(bytes: pubkey.toBytes()),
    );

final program = await SimplicityProgram.loadWithArguments(
  source: p2pkSource,
  arguments: args,
);

final cmr = program.cmr();
```

In Rust:

```rust
let program = SimplicityProgram::load_with_arguments(
    P2PK_SOURCE.to_string(),
    &SimplicityArguments::new().add_value(
        "ALICE_PUBLIC_KEY".into(),
        SimplicityTypedValue::u256(pubkey_bytes).unwrap(),
    ),
)?;
```

## 2. Building Stateful Taproot Addresses

For covenants that store on-chain state, build a Taproot tree with a **program leaf** and optional **data leaves** (32-byte storage slots):

```dart
final builder = StateTaprootBuilder()
    .addSimplicityLeaf(depth: 1, cmr: program.cmr())
    .addDataLeaf(depth: 1, data: stateBytes); // 32 bytes

final spendInfo = builder.finalize(internalKey: internalKey);

// Address / script for receiving funds
final script = spendInfo.scriptPubkey();

// IMPORTANT: use output_key for signing and finalization
final programPublicKey = spendInfo.outputKey();
```

For a simple single-leaf P2PK covenant (no state slot), use depth `0` with only a program leaf, or `SimplicityProgram.createP2trAddress()`.

### Unspendable internal key

Many protocols use the standard unspendable internal key (BIP-341 NUMS point):

```
50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
```

In Rust this is `UNSPENDABLE_TAPROOT_PUBKEY`; in Dart pass it to `XOnlyPublicKey.fromString`.

## 3. Deriving Covenant Script Hashes

When a contract references another program's script hash (e.g. for cross-covenant checks):

```dart
final scriptHash = hashScript(script: spendInfo.scriptPubkey());
// 32-byte SHA256 of the script consensus bytes
```

## 4. Building Witness Values for `Either` Paths

Parse witness expressions against a declared type:

```dart
final eitherType = SimplicityType.fromString(s: 'Either<u32, u32>');

// Left branch
final leftWitness = SimplicityTypedValue.parse(
  valueStr: 'Left(42)',
  ty: eitherType,
);

final witness = SimplicityWitnessValues()
    .addValue(name: 'PATH', value: leftWitness);
```

Construct values programmatically:

```dart
final rightValue = SimplicityTypedValue.right(
  leftType: SimplicityType.u32(),
  value: SimplicityTypedValue.u32(value: 99),
);
```

## 5. Building and Spending a Covenant Transaction

Covenant outputs are typically **not** in your LWK wallet. Build the spend with `PsetBuilder` (see `integration_test.rs`):

```dart
final inputBuilder = PsetInputBuilder.fromPrevout(outpoint: fundingOutpoint);
inputBuilder.witnessUtxo(utxo: fundingOutput);
final input = inputBuilder.build();

final pset = PsetBuilder.newV2();
pset.addInput(input: input);
pset.addOutput(output: recipient);
pset.addOutput(output: change);
final txBytes = await pset.build().extractTxBytes();
```

`ExternalUtxo` wraps outpoint + txout + unblinded secrets for future wallet/`TxBuilder` integration; the offline P2PK test builds directly from `PsetBuilder`.

For P2PK covenants, sign with the program key:

```dart
final signature = await program.createP2pkSignature(
  signer: signer,
  derivationPath: path,
  txBytes: txBytes,
  utxosHex: [fundingOutputHex],
  inputIndex: 0,
  network: LiquidNetwork.testnet,
);
```

## 6. Finalizing the Transaction

Attach the Simplicity witness stack:

```dart
final witness = SimplicityWitnessValues()
    .addValue(
      name: 'ALICE_SIGNATURE',
      value: SimplicityTypedValue.byteArray(bytes: signature),
    );

final finalizedBytes = await program.finalizeTransactionWithValues(
  txBytes: txBytes,
  programPublicKeyHex: programPublicKey.toStringRepr(),
  utxosHex: [fundingOutputHex],
  inputIndex: 0,
  witnessValues: witness,
  network: LiquidNetwork.testnet,
  logLevel: SimplicityLogLevel.none,
);
```

The resulting input witness has **4 stack elements**:

```
[witness_bytes, program_bytes, cmr_bytes, control_block]
```

You can verify this with `program.run()` and `simplicityControlBlock()`.

## 7. Which Public Key to Use

| Scenario | Key for `getSighashAll` / `finalizeTransaction` |
|----------|------------------------------------------------|
| Simple P2TR (`createP2trAddress`) | Internal x-only key passed to `finalize()` |
| Stateful taproot (`StateTaprootBuilder`) | **`StateTaprootSpendInfo.outputKey()`** — not the internal key |

Using the wrong key produces invalid signatures or witness rejection.

## 8. PSET Building (Low-Level)

When you need full control:

```dart
final pset = PsetBuilder.newV2();
pset.addInput(input: inputBuilder.build());
pset.addOutput(output: outputBuilder.build());
pset.setFallbackLocktime(height: 1000);
final built = pset.build();
```

Sequence constants for PSET inputs (`ZERO`, `ENABLE_LOCKTIME_NO_RBF`, `MAX`) can be inlined where needed (see Bitcoin/Elements `nSequence` consensus rules).

## Example: Minimal P2PK Flow

1. Derive x-only pubkey from signer
2. `SimplicityProgram.loadWithArguments` with pubkey parameter
3. `StateTaprootBuilder` → `finalize` → fund the `scriptPubkey()`
4. `PsetBuilder` to construct the spend tx
5. `createP2pkSignature` → `SimplicityWitnessValues` → `finalizeTransactionWithValues`
6. Broadcast finalized tx bytes

See `rust/src/contracts/simplicity/integration_test.rs` for a complete offline Rust test of steps 2–6.
