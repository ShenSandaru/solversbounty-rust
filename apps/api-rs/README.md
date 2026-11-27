# SolversBounty API (Rust)

Axum modular monolith that mirrors NestJS `apps/api`. Behavior contract: [RUST_BACKEND_PARITY.md](../../docs/architecture/RUST_BACKEND_PARITY.md). Stack decision: [ADR-013](../../docs/decisions/ADR-013-rust-api-modular-monolith.md).

Nest `apps/api` remains the reference until cutover. `apps/web` keeps calling the same routes, cookies, and CSRF header on port `3001`.

## Run

```bash
cp .env.example .env
# set DATABASE_URL to the same Neon/Postgres schema Nest uses
cargo run
```

```bash
cargo test
```

Unit and smoke tests do not need a live database or Devnet RPC. With `DATABASE_URL` unset, auth and product routes return `503 NOT_READY`; `GET /health` stays `ok`.

## Env

Same names as `apps/api/.env.example`: `DATABASE_URL`, `PORT`, `CORS_ORIGIN`, `SOLANA_RPC_URL`, `ANSWER_ENCRYPTION_KEY`, `TREASURY_*`, `PAYOUT_BROADCAST_ENABLED`, `ADMIN_WALLET_ALLOWLIST`.

`PAYOUT_BROADCAST_ENABLED` defaults to `false`. The worker still reconciles stored signatures.
