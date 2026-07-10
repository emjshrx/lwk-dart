# LWK Dart

Dart/Flutter bindings for Liquid Wallet Kit, extended with Simplicity covenant support.

## Language

**Simplicity Stack**:
Internal Rust primitives for compiling Simplicity programs, building stateful Taproot addresses, and constructing covenant transactions. Foundation for the Lending Protocol API — not exposed as a public Dart API.
_Avoid_: smplx, generic Simplicity API

**Offer**:
A lending covenant on-chain, progressing through pending (pre-lock) → active → repaid or liquidated. Represented in Dart by a stateful `LendingOffer` that holds protocol parameters and on-chain storage (`is_active`, `current_debt`) and exposes lifecycle methods (`attachCreation`, `attachAcceptance`, etc.).
_Avoid_: Loan, contract

**Protocol Version**:
The `.simf` contract sources vendored in-repo, pinned to a specific `simplicity-lending` git rev recorded in the manifest. Upgrades are explicit: bump rev, re-copy sources, re-run integration tests.
_Avoid_: Submodule, runtime fetch

**Network (lending)**:
Liquid testnet is the supported and tested network for v1. Mainnet types compile but require explicit opt-in (`allowMainnet: true` or equivalent) before any lending or indexer call succeeds.
_Avoid_: Mainnet-first

**IssuanceFactory**:
A protocol-specific contract for issuing utility NFTs used in the lending setup flow. Exposed as its own stateful type alongside `LendingOffer`.
_Avoid_: Factory, issuance contract

**Lending Protocol API**:
Typed Dart methods for building and signing lending covenant transactions across the full offer lifecycle (utility NFT setup, pre-lock, accept, cancel, repay, liquidate, claim). Public types mirror upstream: `LendingOffer`, `IssuanceFactory`, `LendingTransaction`, `LendingIndexer`. Supporting contracts (`script_auth`, `asset_auth`, `asset_auth_vault`) remain internal. Does not include generic asset issuance — users supply principal and collateral assets themselves.
_Avoid_: Contracts API, Simplicity API

**LendingConfig**:
Module-level configuration set via `Lending.init(config: ...)` after `LibLwk.init()`. Holds `network`, `allowMainnet`, and `indexerBaseUrl`. Stored in Rust and shared by all lending types. Re-init replaces the previous config (production apps should still call once).
_Avoid_: Per-call config, per-instance config

**Indexer Client**:
Read-only HTTP client for the simplicity-lending indexer — listing offers and reading offer status. Implemented in Rust behind the FFI bridge, matching how payjoin and electrum calls work. Does not build, sign, or broadcast transactions.
_Avoid_: Backend API, remote API

**LendingTransaction**:
A lending-specific transaction builder wrapping the internal PSET machinery. Passed to `LendingOffer.attach*` methods; callers extract a partial PSET for signing via `Wallet` and broadcast via `Blockchain`.
_Avoid_: PsetBuilder (public), FinalTransaction

