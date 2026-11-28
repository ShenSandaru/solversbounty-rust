import {
  createQuestionInputSchema,
  type CreateQuestionInput,
  type QuestionType,
  type VerificationMethod,
} from "@solversbounty/shared";
import { selectableVerificationMethods } from "@/lib/labels";

export const SAMPLE_QUESTION_CSV = [
  "prompt,questionType,verificationMethod,expectedAnswer,options",
  '"What is 2 + 2?",multiple_choice,exact,4,"2|3|4|5"',
  '"What is the capital of Japan?",short_answer,case_insensitive,Tokyo,',
  '"Find the next number: 2, 4, 6, ?",numerical_pattern,numeric_exact,8,',
].join("\n");

export type CsvParsedRow = {
  rowNumber: number;
  prompt: string;
  questionType: string;
  verificationMethod: string;
  valid: boolean;
  errors: string[];
  question?: CreateQuestionInput;
};

export type CsvImportResult = {
  headerError?: string;
  rows: CsvParsedRow[];
};

const REQUIRED_HEADERS = ["prompt", "questionType", "verificationMethod", "expectedAnswer"] as const;

export function parseCsvRows(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let inQuotes = false;
  let i = 0;

  while (i < text.length) {
    const ch = text[i];
    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"';
          i += 2;
          continue;
        }
        inQuotes = false;
        i += 1;
        continue;
      }
      field += ch;
      i += 1;
      continue;
    }

    if (ch === '"') {
      inQuotes = true;
      i += 1;
      continue;
    }
    if (ch === ",") {
      row.push(field);
      field = "";
      i += 1;
      continue;
    }
    if (ch === "\r" || ch === "\n") {
      if (ch === "\r" && text[i + 1] === "\n") {
        i += 1;
      }
      row.push(field);
      if (row.some((cell) => cell.trim() !== "")) {
        rows.push(row);
      }
      row = [];
      field = "";
      i += 1;
      continue;
    }
    field += ch;
    i += 1;
  }

  row.push(field);
  if (row.some((cell) => cell.trim() !== "")) {
    rows.push(row);
  }
  return rows;
}

export function parseAndValidateQuestionCsv(
  csvText: string,
  existingPrompts: string[] = [],
): CsvImportResult {
  const rawRows = parseCsvRows(csvText);
  if (rawRows.length === 0) {
    return { headerError: "The CSV file is empty.", rows: [] };
  }

  const headers = (rawRows[0] ?? []).map((h) => h.trim());
  const missing = REQUIRED_HEADERS.filter((req) => !headers.includes(req));
  if (missing.length > 0) {
    return {
      headerError: `Missing required CSV column(s): ${missing.join(", ")}`,
      rows: [],
    };
  }

  const idxOf = (name: string) => headers.indexOf(name);
  const promptIdx = idxOf("prompt");
  const typeIdx = idxOf("questionType");
  const methodIdx = idxOf("verificationMethod");
  const answerIdx = idxOf("expectedAnswer");
  const optionsIdx = idxOf("options");

  const seenPrompts = new Set(existingPrompts.map((p) => p.trim().toLowerCase()).filter(Boolean));
  const parsedRows: CsvParsedRow[] = [];

  for (let i = 1; i < rawRows.length; i += 1) {
    const cells = rawRows[i] ?? [];
    const prompt = (cells[promptIdx] ?? "").trim();
    const questionType = (cells[typeIdx] ?? "").trim();
    const verificationMethod = (cells[methodIdx] ?? "").trim();
    const expectedAnswer = (cells[answerIdx] ?? "").trim();
    const rawOptions = optionsIdx >= 0 ? (cells[optionsIdx] ?? "").trim() : "";
    const options =
      rawOptions.length > 0
        ? rawOptions.split("|").map((o) => o.trim())
        : undefined;

    const errors: string[] = [];

    if (
      !selectableVerificationMethods.includes(
        verificationMethod as (typeof selectableVerificationMethods)[number],
      )
    ) {
      errors.push("Unsupported verificationMethod.");
    }

    const normalizedPrompt = prompt.toLowerCase();
    if (normalizedPrompt && seenPrompts.has(normalizedPrompt)) {
      errors.push("Duplicate question prompt.");
    }

    const candidate = {
      questionType: questionType as QuestionType,
      prompt,
      ...(options ? { options } : {}),
      expectedAnswer,
      verificationMethod: verificationMethod as VerificationMethod,
    };

    const result = createQuestionInputSchema.safeParse(candidate);
    if (!result.success) {
      for (const issue of result.error.issues) {
        if (!errors.includes(issue.message)) {
          errors.push(issue.message);
        }
      }
    }

    if (errors.length === 0 && normalizedPrompt) {
      seenPrompts.add(normalizedPrompt);
    }

    parsedRows.push({
      rowNumber: i + 1,
      prompt,
      questionType,
      verificationMethod,
      valid: errors.length === 0,
      errors,
      ...(errors.length === 0 ? { question: result.success ? result.data : candidate } : {}),
    });
  }

  return { rows: parsedRows };
}
