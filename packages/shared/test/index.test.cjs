const assert = require("node:assert/strict");
const test = require("node:test");
const {
  APP_NAME,
  LAMPORTS_PER_SOL,
  createBountyInputSchema,
  createQuestionInputSchema,
  formatLamportsAsSol,
  parseSolToLamports,
  principalLamports,
  publicBountySchema,
  publicQuestionSchema,
} = require("../dist/index.js");

test("shared package exports the app identity and lamport scale", () => {
  assert.equal(APP_NAME, "SolversBounty");
  assert.equal(LAMPORTS_PER_SOL, 1_000_000_000n);
});

test("0.01 SOL for 100 solvers is 1 SOL of principal lamports", () => {
  const reward = parseSolToLamports("0.01");
  assert.equal(reward, 10_000_000n);
  assert.equal(principalLamports(reward, 100), LAMPORTS_PER_SOL);
  assert.equal(formatLamportsAsSol(LAMPORTS_PER_SOL), "1");
  assert.equal(formatLamportsAsSol(reward), "0.01");
});

test("public bounty schema rejects an expected answer field", () => {
  const result = publicBountySchema.safeParse({
    id: "sample",
    title: "Sample",
    description: "A public description.",
    instructions: "",
    prompt: "What is shown to solvers?",
    category: "logic",
    status: "active",
    rewardLamports: "10000000",
    maxRewardedSolvers: 10,
    rewardedCount: 0,
    attemptLimit: 3,
    deadline: "2026-12-01T00:00:00.000Z",
    expectedAnswer: "hidden",
  });
  assert.equal(result.success, false);
});

test("publicQuestionSchema rejects expectedAnswer", () => {
  const result = publicQuestionSchema.safeParse({
    id: "q1",
    orderIndex: 0,
    questionType: "multiple_choice",
    prompt: "Pick one",
    options: ["A", "B"],
    expectedAnswer: "A",
  });
  assert.equal(result.success, false);
});

test("createBountyInputSchema supports legacy single-question and multi-question payloads", () => {
  const now = new Date("2026-01-01T00:00:00.000Z");
  const schema = createBountyInputSchema(now);
  const base = {
    title: "Multi-question test",
    description: "Description",
    instructions: "",
    category: "logic",
    rewardSol: "0.1",
    maxRewardedSolvers: 5,
    attemptLimit: 3,
    deadline: "2026-12-01T00:00:00.000Z",
  };

  const single = schema.safeParse({
    ...base,
    prompt: "What is 2+2?",
    expectedAnswer: "4",
    verificationMethod: "numeric_exact",
  });
  assert.equal(single.success, true);

  const multi = schema.safeParse({
    ...base,
    questions: [
      {
        orderIndex: 0,
        questionType: "multiple_choice",
        prompt: "Pick 4",
        options: ["3", "4"],
        expectedAnswer: "4",
        verificationMethod: "exact",
      },
      {
        orderIndex: 1,
        questionType: "short_answer",
        prompt: "Capital of France?",
        expectedAnswer: "Paris",
        verificationMethod: "case_insensitive",
      },
    ],
  });
  assert.equal(multi.success, true);
});

test("createQuestionInputSchema validates MCQ options and rejects invalid configs", () => {
  assert.equal(
    createQuestionInputSchema.safeParse({
      questionType: "multiple_choice",
      prompt: "Only one option",
      options: ["A"],
      expectedAnswer: "A",
      verificationMethod: "exact",
    }).success,
    false,
  );
  assert.equal(
    createQuestionInputSchema.safeParse({
      questionType: "multiple_choice",
      prompt: "Answer not in options",
      options: ["A", "B"],
      expectedAnswer: "C",
      verificationMethod: "exact",
    }).success,
    false,
  );
  assert.equal(
    createQuestionInputSchema.safeParse({
      questionType: "short_answer",
      prompt: "Short answer with options",
      options: ["A", "B"],
      expectedAnswer: "A",
      verificationMethod: "exact",
    }).success,
    false,
  );
});
