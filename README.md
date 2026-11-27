# SolversBounty

**Solve Problems. Earn Rewards. Unlock Human Insights.**

SolversBounty is a wallet-authenticated cognitive micro-task marketplace and human reasoning research platform. Creators publish predefined challenges, fund a fixed SOL reward pool on Solana Devnet, pay a separate 5% commission on that pool, and study how people attempt those challenges. Solvers connect a wallet, submit answers, and receive the same micro-reward when the backend judges them eligible. Solvers do not pay a participation fee.

**Document status:** Product implementation in progress. Sole API is Rust `apps/api-rs` ([ADR-013](docs/decisions/ADR-013-rust-api-modular-monolith.md)). NestJS has been removed. Requirement statuses live in [REQUIREMENTS.md](docs/product/REQUIREMENTS.md). Live Phantom login, live Devnet funding, and live payout broadcast are not verified yet.

The hackathon MVP uses a **backend-managed custodial Devnet treasury**. Wallet login and SOL transfers are on Solana. Custody of reward funds is not trustless escrow. See [ADR-002](docs/decisions/ADR-002-custodial-devnet-treasury.md).

## Product objectives

SolversBounty has two equal purposes:

1. **Human intelligence marketplace.** Creators publish cognitive challenges and fund rewards. Eligible correct solvers receive a fixed micro-payment, one reward per wallet per bounty.
2. **Human reasoning research.** With explicit consent, the MVP records four observed metrics: correctness, attempt count, completion time, and participant count ([ADR-009](docs/decisions/ADR-009-minimal-research-analytics.md)). Creators see aggregates for their own bounties. Administrators may see platform aggregates. Neither sees solver wallet addresses or individual research records in analytics ([ADR-010](docs/decisions/ADR-010-role-based-analytics-visibility.md)). Strategy surveys, written explanations, answer-change history, and research export are not in the MVP.

The product is privacy-conscious. It does not promise anonymity. A wallet address is a pseudonym, and Solana transactions are public. The MVP does not collect email addresses, passwords, or KYC data.

The demo story is one complete path: a researcher creates a challenge, funds it, solvers connect their wallets and submit answers, the backend verifies those answers, eligible solvers are paid on Devnet, and the researcher reads the resulting analytics.

## Tech stack

Confirmed choices:

| Area | Choice |
| --- | --- |
| Monorepo | pnpm workspaces and Turborepo |
| Frontend | Next.js App Router, TypeScript, Tailwind CSS, shadcn/ui |
| Client state | Zustand |
| Server state | TanStack Query |
| Forms | React Hook Form and Zod |
| Backend | Rust Axum modular monolith (`apps/api-rs`, [ADR-013](docs/decisions/ADR-013-rust-api-modular-monolith.md)) |
| API contract | HTTP JSON + cookies/CSRF; see [RUST_BACKEND_PARITY.md](docs/architecture/RUST_BACKEND_PARITY.md) |
| Database | PostgreSQL (local / Neon); sqlx at runtime; schema under [`apps/api-rs/db`](apps/api-rs/db/README.md) |
| Blockchain | Solana Devnet (RPC + Ed25519 wallet verify; web uses `@solana/web3.js` / wallet adapter) |
| Authentication | Wallet signature verification (Phantom); server-side sessions |
| Tests | Vitest for the web app; `cargo test` for the API |

Details and the proposal register: [Implementation plan](docs/development/IMPLEMENTATION_PLAN.md).

## Repository overview

Monorepo with Next.js web, Rust Axum API, shared TypeScript contracts, and product docs. Schema and migrations live under `apps/api-rs/db` (sqlx at runtime). License is [MIT](LICENSE).

```text
.
├── AGENTS.md
├── LICENSE
├── README.md
├── apps/
│   ├── api-rs/       Rust Axum API (sqlx + db schema)
│   └── web/          Next.js frontend
├── packages/
│   ├── shared/       Shared Zod contracts and constants
│   └── typescript-config/
├── docs/
│   ├── architecture/
│   ├── decisions/
│   ├── development/
│   └── product/
├── package.json
├── pnpm-workspace.yaml
└── turbo.json
```

`apps/web/AGENTS.md` is a Next.js local note. The root [AGENTS.md](AGENTS.md) remains the product and repository authority.

## Development workflow

Install and run from the repository root:

```bash
pnpm install
pnpm --filter @solversbounty/shared build
pnpm dev:web          # Next.js on :3000
pnpm dev:api          # Rust API on :3001
pnpm test:api         # cargo test for apps/api-rs
pnpm build
pnpm lint
pnpm test
pnpm typecheck
```

`GET /health` reports process liveness. `GET /health/ready` reports whether a configured PostgreSQL connection is up. Copy `apps/api-rs/.env.example` and `apps/web` env as needed. Development can start without `DATABASE_URL`; a bad URL fails readiness, not liveness. Production requires `DATABASE_URL` with `sslmode=require` on Neon.

| Route | Role |
| --- | --- |
| `/` | Home |
| `/bounties` | Marketplace (API-backed active list) |
| `/bounties/create` | Create draft (requires signed-in session) |
| `/bounties/[id]` | Challenge details / fund / solve |
| `/dashboard` | Creator and solver overview |
| `/research` | Research analytics (ADR-009 aggregates when authorized) |

API-proven pieces include nonce auth, draft create/edit, marketplace listing, consent/session/submit/verify with E1 payout reservation, and creator analytics aggregates. Live ADR-012 two-transfer funding and activate succeeded via [`apps/api-rs/scripts/devnet-funding-smoke.mjs`](apps/api-rs/scripts/devnet-funding-smoke.mjs) (BOUNTY-007, BOUNTY-016, PAY-008, PAY-018). Still open: live Phantom browser smoke (AUTH-001 and related), and live treasury payout broadcast (`PAYOUT_BROADCAST_ENABLED` defaults off; SystemProgram+Memo signing is implemented). Manual wallet check: [MANUAL_PHANTOM_AUTH.md](docs/development/MANUAL_PHANTOM_AUTH.md).

Read [AGENTS.md](AGENTS.md) before changing product behavior. Update [progress](docs/development/PROGRESS.md) when a milestone actually changes. Do not mark a requirement `implemented` until its acceptance criteria pass.

Local and Neon PostgreSQL share the schema under [`apps/api-rs/db`](apps/api-rs/db/README.md). The Neon development database already has the initial migration; do not re-apply it to production.

## Documentation

| Document | Purpose |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Repository-wide instructions for people and coding agents |
| [Product requirements](docs/product/PRD.md) | Product scope, journeys, priorities, and reconciliations |
| [Requirements catalogue](docs/product/REQUIREMENTS.md) | Stable IDs, acceptance criteria, and implementation status |
| [User stories](docs/product/USER_STORIES.md) | Creator, solver, and administrator stories |
| [System architecture](docs/architecture/SYSTEM_ARCHITECTURE.md) | Boundaries, trust, and data flow |
| [Backend architecture](docs/architecture/BACKEND_ARCHITECTURE.md) | API modules, transactions, and errors |
| [Rust API parity](docs/architecture/RUST_BACKEND_PARITY.md) | HTTP contract and module map for `apps/api-rs` |
| [Database design](docs/architecture/DATABASE_DESIGN.md) | Proposed entities, constraints, and sensitive data |
| [Blockchain design](docs/architecture/BLOCKCHAIN_DESIGN.md) | Deposits, treasury, payouts, refunds, and failures |
| [Implementation plan](docs/development/IMPLEMENTATION_PLAN.md) | 40-hour plan, proposals, open questions, conflicts |
| [Progress](docs/development/PROGRESS.md) | Current milestone and verification |
| [Coding standards](docs/development/CODING_STANDARDS.md) | TypeScript, structure, validation, logging |
| [Testing strategy](docs/development/TESTING_STRATEGY.md) | Unit, API, payment safety, and demo-path tests |
| [ADR index](docs/decisions/README.md) | When and how to record a decision |
| [ADR-001 Modular monolith](docs/decisions/ADR-001-modular-monolith.md) | Why the backend is one modular process |
| [ADR-002 Custodial treasury](docs/decisions/ADR-002-custodial-devnet-treasury.md) | Why MVP custody is a Devnet hot wallet |
| [Threat model](docs/security/THREAT_MODEL.md) | Assets, boundaries, and abuse scenarios |
| [Security review](docs/security/SECURITY_REVIEW.md) | Findings, auth requirements, and open decisions |
| [Security checklist](docs/security/SECURITY_CHECKLIST.md) | Gates for later milestones |

## Glossary

Use these terms in copy, code, and docs.

| Term | Meaning |
| --- | --- |
| Bounty | The funded challenge record. In the MVP it has one question. |
| Challenge | Solver-facing name for a bounty. |
| Creator | Authenticated wallet that owns a bounty. |
| Solver | Authenticated wallet that attempts a bounty. |
| Administrator | Allowlisted wallet with the `admin` role (PD-04). May read platform aggregated analytics. May not read individual solver research records through analytics ([ADR-010](docs/decisions/ADR-010-role-based-analytics-visibility.md)). |
| Expected answer | Creator-defined answer. Hidden from public APIs. |
| Submitted answer | Text the solver sent for one attempt. |
| Solving session | One solver's server-timed attempt window on one bounty. |
| Submission attempt | One submitted answer inside a solving session. |
| Reward allocation | The single reserved reward for one solver on one bounty. |
| Payout | The on-chain transfer attempt for one allocation. |
| Treasury | Backend-managed custodial Devnet wallet that holds reward SOL. |
| Refundable balance | Unallocated reward-pool lamports not reserved, paid, or refunded. The name is an accounting balance, not a creator refund entitlement. After the deadline or the participant cap, [ADR-008](docs/decisions/ADR-008-unused-reward-pool-retention.md) retains this remainder as platform revenue and keeps the lamports in this balance. They are not a second asset and are not added to `paid`. |
| Max rewarded solvers | Maximum number of wallets that may receive the reward. |
| Pseudonymous | Identified by wallet address, without a claim of real-world uniqueness or anonymity. |

Ledger identity used throughout the design:

```text
funded = reserved + paid + refundable + refunded
```

`funded` is cumulative reward-pool deposits. It does not include the platform commission ([ADR-007](docs/decisions/ADR-007-company-platform-commission.md)). `reserved` is held for an unconfirmed payout. `paid` is confirmed solver payouts only. `refundable` can still be reserved while the bounty can pay solvers. After the deadline or the maximum rewarded solvers, the unallocated remainder is retained by SolversBounty ([ADR-008](docs/decisions/ADR-008-unused-reward-pool-retention.md)) and is not a creator refund. `refunded` remains in the identity. Cancelled, never-activated, and failed-funding balances are not decided. See [Blockchain design](docs/architecture/BLOCKCHAIN_DESIGN.md).

## Eligibility rule E1

This paragraph is canonical. Related docs repeat it verbatim.

A reward allocation is created only inside one database transaction when the submission attempt is verified correct, the attempt belongs to the authenticated solver's solving session, a consent record exists, the attempt number is within the bounty attempt limit, the bounty is active, the server time is at or before the deadline, that solver has no allocation on that bounty, the number of allocations is below max rewarded solvers, and refundable lamports are at least the reward. The same transaction moves the reward from refundable to reserved and inserts one payout in `pending`. The chain transfer happens after the transaction commits. A correct attempt that fails only the capacity checks (allocation cap or refundable balance) is stored with an ineligibility reason and creates no payout. Requests that fail consent, ownership, attempt limit, bounty activity, or deadline are rejected and store no attempt.

## License

[MIT](LICENSE)
