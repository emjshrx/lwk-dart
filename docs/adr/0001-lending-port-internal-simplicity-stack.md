# Port simplicity-lending onto the internal Simplicity stack

lwk-dart already ships internal Simplicity primitives (`SimplicityProgram`, `StateTaprootBuilder`, `PsetBuilder`, etc. in `rust/src/contracts/simplicity/`). The lending integration **ports** orchestration logic from [simplicity-lending](https://github.com/BlockstreamResearch/simplicity-lending) `lending-contracts` onto those primitives instead of depending on `smplx-std` / `lending-contracts` as a crate.

**Considered options**

- **Wrap `lending-contracts`** — add `lending-contracts` + `smplx-std` as dependencies and expose types through FRB. Canonical upstream logic, but introduces a second transaction-building stack alongside the one we built for this purpose.
- **Port onto internal stack (chosen)** — reimplement `LendingOffer`, `IssuanceFactory`, and supporting orchestration using existing `rust/src/contracts/` primitives; vendor `.simf` sources with a pinned upstream rev.
- **Expose generic Simplicity API only** — publish `doc/simplicity.md` primitives to Dart and leave lending wiring to consumers. Too low-level for the target mobile-wallet use case.

**Consequences**

- Protocol edge cases (repayment phases, vault attachment, witness branches) must be kept in sync with upstream manually when bumping the pinned rev.
- Single native stack; no `smplx-std` in the FFI binary.
- Supporting contracts (`script_auth`, `asset_auth`, `asset_auth_vault`) stay internal — only `LendingOffer`, `IssuanceFactory`, `LendingTransaction`, and `LendingIndexer` are public.

See `docs/lending.md` and `CONTEXT.md` for the full design.
