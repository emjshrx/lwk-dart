# Source lending `.simf` files via git submodule

The lending port needs upstream Simplicity contract sources from [simplicity-lending](https://github.com/BlockstreamResearch/simplicity-lending). ADR-0001 chose to port orchestration onto the internal stack rather than depend on `lending-contracts` / `smplx-std`, but left contract delivery as "vendor with a pinned rev."

**Considered options**

- **Copy `.simf` into the repo** — simple `include_str!`, but duplicates upstream sources and requires manual re-copy on every bump.
- **Cargo git dependency on `lending-contracts`** — pins rev in `Cargo.toml`, but the crate is not on crates.io and requires `smplx` artifact generation to compile; would pull `smplx-std` into the build graph.
- **Git submodule (chosen)** — `vendor/simplicity-lending` at a pinned commit; Rust reads `crates/contracts/simf/*.simf` via `include_str!`. Rev is the submodule gitlink in the parent repo. No `smplx-std` dependency.
- **Runtime fetch from GitHub** — fragile for offline/CI builds and harder to audit.

**Consequences**

- Cloners must run `git submodule update --init --recursive` (document in PR / README as needed).
- Upgrades: bump the submodule commit, re-run integration tests. No separate manifest file.
- Orchestration still ports onto the internal Simplicity stack (ADR-0001 unchanged).

See `docs/lending.md` and `CONTEXT.md`.
