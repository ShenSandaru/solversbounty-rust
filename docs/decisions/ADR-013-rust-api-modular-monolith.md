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
4. NestJS `apps/api` has been **removed**. The sole API deployable is `apps/api-rs` on port `3001`. Schema and migration history live under [`apps/api-rs/db`](../../apps/api-rs/db/README.md).
5. `packages/shared` and `apps/web` stay TypeScript.

## Consequences

- Agents and humans treat [RUST_BACKEND_PARITY.md](../architecture/RUST_BACKEND_PARITY.md) as the HTTP/behavior contract for the Rust API.
- Requirement statuses are not marked `implemented` solely because of the language port.
- New money, auth, or privacy changes still need their own ADR.
- Dev tooling uses `pnpm dev:api` / `pnpm test:api` (`cargo`) for the API and pnpm/turbo for web and shared.
- Open hardening (not blockers for Nest removal): Phantom browser smoke, deeper DB HTTP parity tests, and SystemProgram+Memo treasury broadcast byte construction when payout broadcast is enabled.

## Alternatives

| Alternative | Why it was not selected |
| --- | --- |
| Keep Nest forever and add Rust only for one module | Duplicates the monolith and drifts contracts |
| Rewrite web and shared in Rust | Out of scope; web already couples to the HTTP/cookie contract |
| Microservices or a queue cluster | Forbidden for the hackathon MVP; ADR-001 stays |
