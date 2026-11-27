# SolversBounty API (Rust)

Sole backend process: Axum + Tokio + sqlx. Behavior contract: [RUST_BACKEND_PARITY.md](../../docs/architecture/RUST_BACKEND_PARITY.md). Stack: [ADR-013](../../docs/decisions/ADR-013-rust-api-modular-monolith.md). Schema: [`db/`](db/README.md).

`apps/web` calls this API on port `3001` (cookies `sb_session` / `sb_csrf`, header `X-CSRF-Token`).

## Run

```bash
cp .env.example .env
# set DATABASE_URL to Neon/Postgres (sslmode=require for Neon)
cargo run
# or from repo root: pnpm dev:api
```

```bash
cargo test
# or: pnpm test:api
```

Unit and smoke tests do not need a live database. With `DATABASE_URL` unset, product routes return `503 NOT_READY`; `GET /health` stays `ok`.

## Env

`DATABASE_URL`, `PORT`, `CORS_ORIGIN`, `SOLANA_RPC_URL`, `ANSWER_ENCRYPTION_KEY`, `TREASURY_*`, `PAYOUT_BROADCAST_ENABLED`, `ADMIN_WALLET_ALLOWLIST`.

`PAYOUT_BROADCAST_ENABLED` defaults to `false`. The worker still reconciles stored signatures.
