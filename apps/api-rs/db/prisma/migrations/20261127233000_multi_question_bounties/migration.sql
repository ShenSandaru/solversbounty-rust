-- Phase 1: Multi-question bounties & question-level submissions.
-- Preserves existing single-question bounties by defaulting order_index = 0 and question_type = 'short_answer'.

CREATE TYPE "question_type" AS ENUM (
  'multiple_choice',
  'short_answer',
  'logic_puzzle',
  'mathematics',
  'numerical_pattern',
  'cipher'
);

ALTER TYPE "verification_method" ADD VALUE IF NOT EXISTS 'manual';

CREATE TYPE "manual_review_status" AS ENUM (
  'pending',
  'approved',
  'rejected'
);

DROP INDEX IF EXISTS "bounty_questions_bounty_id_key";

ALTER TABLE "bounty_questions"
  ADD COLUMN "order_index" INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN "question_type" "question_type" NOT NULL DEFAULT 'short_answer',
  ADD COLUMN "options" JSONB;

CREATE UNIQUE INDEX "bounty_questions_bounty_id_order_index_key"
  ON "bounty_questions"("bounty_id", "order_index");

CREATE INDEX "bounty_questions_bounty_id_idx"
  ON "bounty_questions"("bounty_id");

ALTER TABLE "submission_attempts"
  ADD COLUMN "question_id" UUID,
  ADD COLUMN "manual_review_status" "manual_review_status";

CREATE INDEX "submission_attempts_question_id_idx"
  ON "submission_attempts"("question_id");

ALTER TABLE "submission_attempts"
  ADD CONSTRAINT "submission_attempts_question_id_fkey"
  FOREIGN KEY ("question_id") REFERENCES "bounty_questions"("id")
  ON DELETE RESTRICT ON UPDATE CASCADE;
