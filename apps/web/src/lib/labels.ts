import type { BountyCategory, BountyStatus, QuestionType, VerificationMethod } from "@solversbounty/shared";

export const questionTypeLabels: Record<QuestionType, string> = {
  multiple_choice: "Multiple choice",
  short_answer: "Short answer",
  logic_puzzle: "Logic puzzle",
  mathematics: "Mathematics",
  numerical_pattern: "Numerical pattern",
  cipher: "Cipher",
};

export const categoryLabels: Record<BountyCategory, string> = {
  logic: "Logic",
  mathematics: "Mathematics",
  numerical_patterns: "Numerical patterns",
  ciphers: "Ciphers",
};

export const verificationMethodLabels: Record<VerificationMethod, string> = {
  exact: "Exact match",
  case_insensitive: "Case-insensitive match",
  normalized_text: "Normalized text",
  numeric_exact: "Exact numeric",
  manual: "Manual review",
};

export const selectableVerificationMethods = [
  "exact",
  "case_insensitive",
  "normalized_text",
  "numeric_exact",
  "manual",
] as const satisfies readonly VerificationMethod[];

export const statusLabels: Record<BountyStatus, string> = {
  draft: "Draft",
  funding_pending: "Funding pending",
  active: "Active",
  closed: "Closed",
};

export const difficultyLevels = ["introductory", "intermediate", "advanced"] as const;

export type DifficultyLevel = (typeof difficultyLevels)[number];

export const difficultyLabels: Record<DifficultyLevel, string> = {
  introductory: "Introductory",
  intermediate: "Intermediate",
  advanced: "Advanced",
};

export const closeReasonLabels = {
  creator: "Closed by creator",
  deadline: "Expired",
  fully_allocated: "Fully allocated",
  admin: "Closed by an administrator",
} as const;
