# Lending indexer client in Rust behind FRB

The read-only **Indexer Client** (`LendingIndexer`) calls the simplicity-lending indexer HTTP API from **Rust**, exposed to Dart through the same FRB bridge as `Wallet`, `Blockchain`, and payjoin — not from Dart via `http` or `dio`.

**Considered options**

- **Pure Dart HTTP** — idiomatic for REST, easy to mock. Rejected: inconsistent with how this repo handles external services (electrum and SideSwap payjoin both go through Rust).
- **Rust via FRB (chosen)** — HTTP client in Rust (`reqwest` or equivalent), serde DTOs aligned with indexer OpenAPI schemas, FRB-generated Dart bindings exported from `package:lwk/lending.dart`.

**Consequences**

- Indexer schema changes require Rust DTO updates and FRB regeneration.
- `dio` in `pubspec.yaml` remains unused by lending; only internal tooling (e.g. dylib download) uses Dart `http`.
- Indexer is discovery-only: list/search offers, read status. No tx construction, signing, or broadcast through the indexer.

See `docs/lending.md` for endpoints and usage.
