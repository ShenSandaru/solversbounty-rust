import { z } from "zod";
import { parseSolToLamports } from "./lamports";

export const bountyCategorySchema = z.enum([
  "logic",
  "mathematics",
  "numerical_patterns",
  "ciphers",
]);

export const verificationMethodSchema = z.enum([
  "exact",
  "case_insensitive",
  "normalized_text",
  "numeric_exact",
  "manual",
]);

export const questionTypeSchema = z.enum([
  "multiple_choice",
  "short_answer",
  "logic_puzzle",
  "mathematics",
  "numerical_pattern",
  "cipher",
]);

export const bountyStatusSchema = z.enum([
  "draft",
  "funding_pending",
  "active",
  "closed",
]);

export const publicQuestionSchema = z
  .object({
    id: z.string().min(1),
    orderIndex: z.number().int().nonnegative(),
    questionType: questionTypeSchema,
    prompt: z.string().min(1),
    options: z.array(z.string().min(1)).nullable().optional(),
    verificationMethod: verificationMethodSchema.optional(),
  })
  .strict();

export const createQuestionInputSchema = z
  .object({
    orderIndex: z.number().int().nonnegative().optional(),
    questionType: questionTypeSchema.default("short_answer"),
    prompt: z.string().trim().min(1, "Enter the challenge question.").max(5000),
    options: z.array(z.string().trim().min(1)).optional(),
    expectedAnswer: z.string().trim().min(1, "Enter the expected answer.").max(500),
    verificationMethod: verificationMethodSchema,
  })
  .superRefine((value, context) => {
    if (value.questionType === "multiple_choice") {
      if (!value.options || value.options.length < 2) {
        context.addIssue({
          code: "custom",
          path: ["options"],
          message: "Multiple-choice questions require at least 2 options.",
        });
        return;
      }
      const unique = new Set(value.options);
      if (unique.size !== value.options.length) {
        context.addIssue({
          code: "custom",
          path: ["options"],
          message: "Multiple-choice options must be distinct.",
        });
      }
      if (value.verificationMethod !== "manual" && !value.options.includes(value.expectedAnswer)) {
        context.addIssue({
          code: "custom",
          path: ["expectedAnswer"],
          message: "Expected answer must match one of the multiple-choice options.",
        });
      }
    } else if (value.options && value.options.length > 0) {
      context.addIssue({
        code: "custom",
        path: ["options"],
        message: "Options are only allowed for multiple-choice questions.",
      });
    }
  });

export const publicBountySchema = z
  .object({
    id: z.string().min(1),
    title: z.string().min(1),
    description: z.string().min(1),
    instructions: z.string(),
    prompt: z.string().min(1).optional(),
    questions: z.array(publicQuestionSchema).min(1).optional(),
    category: bountyCategorySchema,
    status: bountyStatusSchema,
    rewardLamports: z.string().regex(/^[0-9]+$/),
    maxRewardedSolvers: z.number().int().positive(),
    rewardedCount: z.number().int().nonnegative(),
    attemptLimit: z.number().int().positive(),
    deadline: z.string().datetime({ offset: true }),
  })
  .strict();

export const createBountyInputSchema = (now: Date) =>
  z
    .object({
      title: z.string().trim().min(1, "Enter a title.").max(140),
      description: z.string().trim().min(1, "Enter a description.").max(5000),
      instructions: z.string().trim().max(5000),
      prompt: z.string().trim().min(1, "Enter the challenge question.").max(5000).optional(),
      category: bountyCategorySchema,
      expectedAnswer: z.string().trim().min(1, "Enter the expected answer.").max(500).optional(),
      verificationMethod: verificationMethodSchema.optional(),
      questions: z.array(createQuestionInputSchema).min(1).optional(),
      rewardSol: z.string().trim().min(1, "Enter a reward."),
      maxRewardedSolvers: z
        .number({ error: "Enter the maximum number of rewarded solvers." })
        .int("Enter a whole number of solvers.")
        .positive("Enter at least one rewarded solver."),
      attemptLimit: z
        .number({ error: "Enter an attempt limit." })
        .int("Enter a whole number of attempts.")
        .positive("Enter at least one attempt.")
        .max(10, "The proposed maximum is 10 attempts."),
      deadline: z.string().min(1, "Choose a deadline."),
    })
    .superRefine((value, context) => {
      const rewardLamports = parseSolToLamports(value.rewardSol);
      if (rewardLamports === null || rewardLamports <= 0n) {
        context.addIssue({
          code: "custom",
          path: ["rewardSol"],
          message: "Enter a positive SOL amount with at most 9 decimal places.",
        });
      }

      const hasQuestions = Boolean(value.questions && value.questions.length > 0);
      const hasLegacySingle = Boolean(value.prompt && value.expectedAnswer && value.verificationMethod);
      if (!hasQuestions && !hasLegacySingle) {
        context.addIssue({
          code: "custom",
          path: ["questions"],
          message: "Provide at least one question.",
        });
      }

      const deadline = new Date(value.deadline);
      if (Number.isNaN(deadline.getTime()) || deadline.getTime() <= now.getTime()) {
        context.addIssue({
          code: "custom",
          path: ["deadline"],
          message: "Choose a deadline in the future. The server clock is authoritative later.",
        });
      }
    });

export type BountyCategory = z.infer<typeof bountyCategorySchema>;
export type VerificationMethod = z.infer<typeof verificationMethodSchema>;
export type QuestionType = z.infer<typeof questionTypeSchema>;
export type BountyStatus = z.infer<typeof bountyStatusSchema>;
export type PublicQuestion = z.infer<typeof publicQuestionSchema>;
export type CreateQuestionInput = z.input<typeof createQuestionInputSchema>;
export type PublicBounty = z.infer<typeof publicBountySchema>;
export type CreateBountyInput = z.input<ReturnType<typeof createBountyInputSchema>>;
