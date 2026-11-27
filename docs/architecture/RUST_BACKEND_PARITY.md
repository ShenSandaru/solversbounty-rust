# Rust backend parity inventory

Status: **authoritative HTTP and domain contract** for `apps/api-rs`. NestJS has been removed. Do not change product logic without updating this contract and the requirement IDs. Originally derived from the Nest controllers and services.

Related: [BACKEND_ARCHITECTURE.md](BACKEND_ARCHITECTURE.md), [DATABASE_DESIGN.md](DATABASE_DESIGN.md), [BLOCKCHAIN_DESIGN.md](BLOCKCHAIN_DESIGN.md), [ADR-001](../decisions/ADR-001-modular-monolith.md).

## Purpose

`apps/api-rs` (Axum + Tokio + sqlx) is the only API. `apps/web` depends on these routes, cookies, CSRF, error envelope, lamport JSON strings, E1, payout worker gates, and answer secrecy.

## Eligibility rule E1 (canonical)

Copy verbatim from [README.md](../../README.md):

> A reward allocation is created only inside one database transaction when the submission attempt is verified correct, the attempt belongs to the authenticated solver's solving session, a consent record exists, the attempt number is within the bounty attempt limit, the bounty is active, the server time is at or before the deadline, that solver has no allocation on that bounty, the number of allocations is below max rewarded solvers, and refundable lamports are at least the reward. The same transaction moves the reward from refundable to reserved and inserts one payout in `pending`. The chain transfer happens after the transaction commits. A correct attempt that fails only the capacity checks (allocation cap or refundable balance) is stored with an ineligibility reason and creates no payout. Requests that fail consent, ownership, attempt limit, bounty activity, or deadline are rejected and store no attempt.

Capacity reasons stored on attempts: `pool_exhausted`, `insufficient_refundable`. Those outcomes are **200** with the attempt recorded, not 5xx.

---

## 1. HTTP surface

No global path prefix. Swagger lives at `/docs` in Nest (optional in Rust). Errors use:

```json
{
  "statusCode": 401,
  "code": "UNAUTHENTICATED",
  "message": "...",
  "requestId": "...",
  "issues": [{ "path": "...", "message": "..." }],
  "checks": { "database": "down" }
}
```

Stable codes: `VALIDATION_ERROR`, `UNAUTHENTICATED`, `AUTH_REJECTED`, `FORBIDDEN`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `NOT_READY`, `INTERNAL`.

BigInt / lamport fields serialize as **decimal strings**.

### Auth (`AuthController`)

| Method | Path | Guards | Body | Success |
|--------|------|--------|------|---------|
| POST | `/auth/nonce` | — | `{ walletAddress }` (32–44) | `{ nonce, message, expiresAt }` |
| POST | `/auth/login` | — | `{ walletAddress, signature }` | `{ walletAddress, role, csrfToken }` + `Set-Cookie` `sb_session`, `sb_csrf` |
| GET | `/auth/session` | cookie | — | `{ id, walletAddress, role, csrfToken }` (may refresh CSRF cookie) |
| POST | `/auth/logout` | CSRF in service | cookie + `x-csrf-token` | `{ revoked: true }` + clear cookies |

### Bounties (`BountiesController`)

| Method | Path | Guards | Notes |
|--------|------|--------|-------|
| POST | `/bounties` | Session + CSRF | Create draft; encrypted answer |
| GET | `/bounties` | — | Public list; query `q?`, `category?` |
| GET | `/bounties/:id/public` | — | One public view |
| GET | `/bounties/mine` | Session | Creator private list (decrypted answer) |
| GET | `/bounties/:id` | Session | Creator private one; else 404 |
| PATCH | `/bounties/:id` | Session + CSRF | Draft only |
| GET | `/bounties/:id/funding` | Session | FundingView |
| POST | `/bounties/:id/funding/:purpose` | Session + CSRF | `purpose`: `reward-pool` \| `commission`; body `{ signature }` |
| POST | `/bounties/:id/activate` | Session + CSRF | Both fundings confirmed |

### Submissions / analytics (`SubmissionsController`, root)

| Method | Path | Guards | Notes |
|--------|------|--------|-------|
| POST | `/bounties/:id/consent` | Session + CSRF | `{ accepted: true }` |
| POST | `/bounties/:id/sessions` | Session + CSRF | Start or return existing |
| GET | `/sessions/:id` | Session | SessionView; no answers |
| GET | `/sessions/:id/result` | Session | `{ status, attemptsUsed, remainingAttempts, latest }` |
| POST | `/sessions/:id/submissions` | Session + CSRF | `{ answer }`; ignore `isCorrect` / `submittedAt` |
| GET | `/bounties/:id/analytics` | Session | Creator ADR-009 aggregates + `accuracy` |

### Payouts (`PayoutsController`, root)

| Method | Path | Guards | Notes |
|--------|------|--------|-------|
| GET | `/payouts/mine` | Session | Solver payouts |
| GET | `/bounties/:id/payouts` | Session | Creator summary (no wallets) |
| GET | `/admin/payouts` | Session + admin | Admin list |
| POST | `/admin/payouts/run` | Session + CSRF + admin | `{ processed: true }` |

### Health

| Method | Path | Success |
|--------|------|---------|
| GET | `/health` | `{ status: "ok", service: "SolversBounty" }` |
| GET | `/health/ready` | 200 `{ status: "ready", checks: { database } }` or 503 `not_ready` when DB down |

---

## 2. Auth contract

Source (historical Nest; now `apps/api-rs/src/modules/auth/*`):

| Constant | Value |
|----------|-------|
| `NONCE_TTL_MS` | 10 minutes |
| `SESSION_TTL_MS` | 8 hours (working default, not an ADR) |
| `NONCE_RATE_LIMIT` | 10 / 60s per wallet |
| `SESSION_COOKIE` | `sb_session` |
| `CSRF_COOKIE` | `sb_csrf` |
| `CSRF_HEADER` | `x-csrf-token` |
| `LOGIN_CHAIN` | `solana:devnet` |

### Login message (byte-compatible)

Plain text lines joined by `\n` (not SIWS preamble):

```
SolversBounty login
Wallet: {walletAddress}
This signature does not approve a SOL transfer.
Domain: {domain}
URI: {uri}
Nonce: {nonce}
Issued At: {issuedAt ISO}
Expiration Time: {expiresAt ISO}
Chain: solana:devnet
```

`domain` / `uri` come from the first `CORS_ORIGIN` entry via `URL.host` / `URL.origin`.

### Session

- Opaque random token in HttpOnly cookie; store **SHA-256 hex** of token in `auth_sessions.refresh_token_hash`.
- Cookies: `HttpOnly`, `SameSite=Lax`, `Path=/`, `Max-Age=…`; `Secure` when `NODE_ENV=production`.
- CSRF: timing-safe compare of cookie vs header; both HttpOnly cookies.
- Login: find active nonce → rebuild message → Ed25519 verify (64-byte base64url sig) → consume nonce → upsert user (admin from allowlist) → revoke other sessions → create session.
- `AUTH_TOKEN_SECRET` is configured but **unused** for session signing (ADR-003).

### Rate limits

| Key | Limit | Window |
|-----|-------|--------|
| `auth-nonce:{wallet}` | 10 | 60s |
| `submit:{userId}` | 30 | 60s |

In-process `InMemoryRateLimit` (no Redis).

---

## 3. Bounty + funding

### Draft / edit

- Validate create body (shared Zod rules): title, description, instructions, prompt, category, expectedAnswer, verificationMethod, rewardSol, maxRewardedSolvers, attemptLimit, deadline **after server now**.
- Principal = `rewardLamports * maxRewardedSolvers`; caps per Nest (`max` principal 1M SOL, max solvers 100_000).
- Status `draft` until funding; edit only while `draft`.
- Encrypt expected answer before store; decrypt only for creator private reads and inside submit verification.

### Answer cipher

AES-256-GCM. Key: 32-byte hex or base64url from `ANSWER_ENCRYPTION_KEY`. `encryptionKeyId = "primary"`.

Format: `v1.{nonce_b64url}.{ciphertext_b64url}.{tag_b64url}` (12-byte nonce).

### Ledger identity

```
funded === reserved + paid + refundable + refunded
```

Entry effects (`funding-ledger.ts`):

| entryType | Effect |
|-----------|--------|
| `deposit` | +funded, +refundable |
| `reserve` | +reserved, −refundable |
| `settle` | −reserved, +paid |
| `release` | −reserved, +refundable |
| `refund` | −refundable, +refunded |

### Funding (ADR-012)

- Obligations: `reward_pool` = principal; `commission` = 5% of principal.
- Deposit memo / reference: `sb:deposit:{bountyId}:{purpose}` where purpose is `reward_pool` or `commission`.
- Path segments use hyphens: `reward-pool`, `commission`.
- Submit: RPC lookup at `confirmed`/`finalized`; fee payer = creator wallet; destination = treasury; exact lamports; matching memo.
- Only **reward_pool** confirmation posts ledger `deposit`. Commission confirms obligation only.
- First confirmed funding moves bounty to `funding_pending`.
- Activate: `SELECT … FOR UPDATE` bounty; both obligations `confirmed` → `active` + `publishedAt`.

### Public DTO omissions

Public views omit: `expectedAnswer`, ciphertext, `encryptionKeyId`, `verificationMethod`, `closeReason`, creator id, funding internals. Marketplace excludes `draft` and `funding_pending`. Auto-close active past deadline with reason `deadline` on read paths that observe it.

---

## 4. Submissions + verification

### Flow

1. Consent (`research-consent-v1` / `RESEARCH_CONSENT_VERSION`): active bounty + server deadline; idempotent unique `(userId, bountyId)`.
2. Start session: requires consent; unique `(bountyId, solverId)` returns existing; `startedAt` = server now; check active/deadline/cap.
3. Submit: server `submittedAt`; decrypt expected inside tx; `answersMatch`; store attempt; on eligible correct apply E1; client flags ignored.

### `answersMatch`

| Method | Rule |
|--------|------|
| `exact` | `===` |
| `case_insensitive` | lowercased equality |
| `normalized_text` | trim + collapse whitespace |
| `numeric_exact` | trim; both must match `/^-?\d+(\.\d+)?$/`; compare trimmed strings (invalid → no match) |

### Capacity (`capacityDecision`)

```
if allocationCount >= maxRewardedSolvers → pool_exhausted
if refundable < rewardLamports → insufficient_refundable
else reserve
```

On reserve (same TX, bounty row locked): `RewardAllocation`, `Payout` with `idempotencyKey = payout:{bountyId}:{solverId}`, ledger `reserve`, maybe close `fully_allocated`, session `completed`, audit. After commit, nudge payout worker (non-blocking).

---

## 5. Payout worker

Sources: `payout-worker.ts`, `prisma-payout.repository.ts`, `treasury-signer.ts`.

| Setting | Value |
|---------|-------|
| Tick | 15_000 ms |
| Batch | 10 |
| Broadcast | only if `PAYOUT_BROADCAST_ENABLED === true` |
| Memo | `sb:payout:{payoutId}` |

State machine:

1. Skip terminal `confirmed` / `failed`.
2. If open chain tx `commitment === "submitted"` → reconcile via RPC.
3. Else if `pending` or `retryable` and broadcast enabled and treasury configured → sign SystemProgram transfer + Memo, `recordSend` (status `submitted`), then broadcast exact bytes.
4. Reconcile: `unavailable` / `not_found` / non-confirmed leave `submitted`. Confirmed failure → `retryable`. Confirmed success matching treasury, memo, single transfer amount/addresses → settle (ledger `settle`, allocation `paid`, payout `confirmed`).
5. Never sign a second transfer while an open `submitted` signature exists. `processed` does not settle.

Match rules: cluster `devnet`, fee payer = treasury, memo exact, one transfer treasury → solver for `amountLamports`.

---

## 6. Analytics

`GET /bounties/:id/analytics` — creator only (404 otherwise).

Fields: `participantCount`, `correctCount`, `incorrectCount`, `attemptCount`, `averageCompletionTimeMs`, `accuracy` (4-decimal string or null). No solver wallets, no raw answers. (`AnalyticsModule` in Nest is empty; logic lives in bounty repository.)

---

## 7. Infra

### Env (same names)

| Name | Notes |
|------|-------|
| `NODE_ENV` | development \| test \| production; Secure cookies in production |
| `PORT` | default 3001 |
| `DATABASE_URL` | required in production; optional otherwise |
| `DIRECT_URL` | migrations / CLI only |
| `SOLANA_RPC_URL` | Devnet HTTP(S) |
| `CORS_ORIGIN` | comma list; reject `*` at boot |
| `JSON_BODY_LIMIT` | default `64kb` |
| `AUTH_TOKEN_SECRET` | present, unused for sessions |
| `ANSWER_ENCRYPTION_KEY` | AES key |
| `TREASURY_SECRET_KEY` / `TREASURY_PUBLIC_KEY` | custodial Devnet |
| `PAYOUT_BROADCAST_ENABLED` | default false |
| `ADMIN_WALLET_ALLOWLIST` | comma wallets |

CORS: credentials true; methods GET HEAD POST PUT PATCH DELETE OPTIONS; headers `Content-Type`, `X-CSRF-Token`.

Logging: structured with `requestId`, method, path (no query), status. Redact answers, cookies, secrets, treasury material. Log chain signatures (public).

Missing treasury / answer key must not crash public reads or `/health`. Auth without DB → 503 `NOT_READY`.

### Schema

PostgreSQL models from [`apps/api-rs/db/prisma/schema.prisma`](../../apps/api-rs/db/prisma/schema.prisma) (14 models) plus partial unique index:

```sql
CREATE UNIQUE INDEX auth_nonces_one_active_per_wallet
  ON auth_nonces (user_wallet)
  WHERE used_at IS NULL AND retired_at IS NULL;
```

---

## 8. Nest → Rust module map

| Nest module | Rust module (`apps/api-rs`) | Owns |
|-------------|-----------------------------|------|
| config / bootstrap | `config`, `http` | Env, CORS, errors |
| health | `http::health` | Liveness / readiness |
| anti-abuse | `modules::anti_abuse` | In-memory rate limit |
| solana | `modules::solana` | Verify, RPC, treasury sign |
| auth | `modules::auth` | Nonce, login, session, CSRF |
| users | (via auth) | Empty Nest module; upsert on login |
| bounties | `modules::bounties` | Draft, funding, activate, public DTO |
| verification | `modules::verification` | Pure `answers_match` |
| submissions | `modules::submissions` | Consent, session, submit, analytics route |
| rewards | `modules::rewards` | Eligibility, payouts, worker |
| analytics | thin / under bounties | Aggregates |
| audit | write helpers | Append-only rows |
| database | `db` | sqlx pool |

Dependency direction matches [BACKEND_ARCHITECTURE.md](BACKEND_ARCHITECTURE.md): verification has no I/O; only rewards + solana advance payouts; no DB transaction held across RPC.

---

## 9. Test matrix (Nest → Rust)

| Nest spec | Rust target | Asserts |
|-----------|-------------|---------|
| `compare-answer.spec.ts` | `verification` unit | Four methods |
| `eligibility` via submissions | `capacity_decision` unit | Cap / refundable |
| `answer-cipher.spec.ts` | cipher unit | Round-trip + bad payload |
| `login-message.spec.ts` | login message unit | Exact lines |
| `in-memory-rate-limit.spec.ts` | rate limit unit | Window |
| `verify-wallet-signature.spec.ts` | solana unit | Accept / reject |
| `funding-math.spec.ts` / `observed-transfer.spec.ts` | funding unit | Principal, commission, match |
| `auth.http.spec.ts` / `auth.service.spec.ts` | auth HTTP | Nonce, login, CSRF logout |
| `auth.postgres.spec.ts` | auth + DB | Replay, expiry, 503 |
| `bounties.http.spec.ts` / postgres | bounties HTTP/DB | Draft, secrecy |
| `funding.http.spec.ts` | funding HTTP | Two transfers mocked RPC |
| `submissions.http.spec.ts` | submissions HTTP | Consent, E1, no leak |
| `payout-worker.spec.ts` / `payout.postgres.spec.ts` | worker | Broadcast gate, settle, no double-send |
| `health.controller.spec.ts` | health | ready / not_ready |
| `bootstrap.cors.spec.ts` | CORS boot | Reject `*` |

---

## 10. Invariants checklist (cutover gate)

Automated without live DB (`cargo test --manifest-path apps/api-rs/Cargo.toml`, 2026-11-27: 31 passed):

- [x] E1 capacity function (`pool_exhausted` / `insufficient_refundable`) ported; submit path uses it inside the bounty lock transaction
- [x] Ledger identity helper (`funded = reserved + paid + refundable + refunded`)
- [x] Answer cipher AES-GCM `v1.*` round-trip
- [x] Login message lines match Nest (no SIWS preamble)
- [x] CORS rejects `*`
- [x] `PAYOUT_BROADCAST_ENABLED=false` does not call RPC for new broadcasts
- [x] Devnet RPC host gate; memo formats `sb:deposit:` / `sb:payout:`
- [x] Health / readiness; auth without DB → `NOT_READY`
- [x] Cookies / CSRF helpers; Ed25519 wallet verify unit test

Remaining hardening (Nest removed; track in progress):

- [ ] Public / solver responses never include `expectedAnswer` (HTTP against DB)
- [ ] Client `isCorrect` / `submittedAt` ignored on write (HTTP)
- [ ] One reward per solver under concurrency
- [ ] Payout reconcile/settle against mocked or Devnet RPC with stored signature
- [ ] Web smoke: login → draft → marketplace list against Rust on `:3001`
- [ ] Live SystemProgram+Memo treasury broadcast byte construction when broadcast is enabled

Do not mark new product requirements `implemented` solely because of the language port.
