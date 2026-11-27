# Database schema (PostgreSQL)

The Axum API in `apps/api-rs` talks to PostgreSQL with **sqlx**. Table and enum names match the Prisma schema that was originally applied from the Nest app.

| Path | Role |
|------|------|
| [`prisma/schema.prisma`](prisma/schema.prisma) | Canonical model reference (14 models) |
| [`prisma/migrations/`](prisma/migrations/) | Applied Neon development migration history |
| [`prisma/manual/partial-indexes.sql`](prisma/manual/partial-indexes.sql) | Partial unique index for one active auth nonce per wallet |

Runtime does not require the Prisma client. Use these files when applying or inspecting schema on a disposable database. Neon development database `neondb` already has the initial migration; do not re-apply it to production.

Connection: `DATABASE_URL` (pooled, `sslmode=require` for Neon). `DIRECT_URL` is only for Prisma CLI if you run it against this folder.
