# ADR-013 Rust API modular monolith

- Status: accepted
- Date: 2026-11-27

## Context

The MVP backend was implemented as a NestJS modular monolith in `apps/api` ([ADR-001](ADR-001-modular-monolith.md)). Product behavior, PostgreSQL schema, cookie sessions ([ADR-003](ADR-003-postgresql-server-side-sessions.md)), custodial Devnet treasury ([ADR-002](ADR-002-custodial-devnet-treasury.md)), E1, and the HTTP contract used by `apps/web` are already locked.

The repository is being converted so the API process is written in Rust while those behaviors stay the same. Nest remains a reference during the side-by-side period. The conversion contract is [RUST_BACKEND_PARITY.md](../architecture/RUST_BACKEND_PARITY.md).

This ADR does not change custody, authentication mechanism, research privacy, E1, or the ledger identity. It records the language and process cutover for the modular monolith.

## Decision

1. The API modular monolith runs as `apps/api-rs` (Axum + Tokio + sqlx against the same PostgreSQL schema).
2. Module boundaries from ADR-001 remain: `verification` is pure; only `rewards` and `solana` advance payout transfers; login signature checks stay local and must not require a healthy RPC.
3. HTTP routes, cookies (`sb_session`, `sb_csrf`), CSRF header, error envelope, lamport JSON strings, and payout worker gates stay compatible with Nest so `apps/web` does not change.
4. NestJS `apps/api` is retained as a behavioral reference until parity checks in the parity doc pass; then local/dev scripts and docs point at `apps/api-rs` on port `3001`. Nest may be deprecated or removed after that sign-off without a further ADR unless product behavior also changes.
5. `packages/shared` and `apps/web` stay TypeScript.

## Consequences

- Agents and humans treat [RUST_BACKEND_PARITY.md](../architecture/RUST_BACKEND_PARITY.md) as the port checklist.
- Requirement statuses are not marked `implemented` solely because of the language port.
- New money, auth, or privacy changes still need their own ADR.
- Dev tooling uses `cargo run` / `cargo test` for the API in addition to existing pnpm scripts for web and shared.

## Alternatives

| Alternative | Why it was not selected |
| --- | --- |
| Keep Nest forever and add Rust only for one module | Duplicates the monolith and drifts contracts |
| Rewrite web and shared in Rust | Out of scope; web already couples to the HTTP/cookie contract |
| Microservices or a queue cluster | Forbidden for the hackathon MVP; ADR-001 stays |
