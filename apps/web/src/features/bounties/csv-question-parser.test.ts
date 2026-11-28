import { describe, expect, it } from "vitest";
import { parseAndValidateQuestionCsv, parseCsvRows } from "./csv-question-parser";

describe("parseCsvRows", () => {
  it("parses quoted fields with commas and escaped quotes", () => {
    const csv = 'prompt,questionType,verificationMethod,expectedAnswer,options\n"Hello, ""world""?",short_answer,exact,yes,';
    const rows = parseCsvRows(csv);
    expect(rows).toHaveLength(2);
    expect(rows[1]?.[0]).toBe('Hello, "world"?');
  });
});

describe("parseAndValidateQuestionCsv", () => {
  it("reports empty CSV files", () => {
    const res = parseAndValidateQuestionCsv("");
    expect(res.headerError).toBeTruthy();
  });

  it("reports missing required columns", () => {
    const res = parseAndValidateQuestionCsv("prompt,questionType\nQ1,short_answer");
    expect(res.headerError).toContain("verificationMethod");
  });

  it("validates valid, invalid MCQ, duplicate, and partial CSVs", () => {
    const csv = [
      "prompt,questionType,verificationMethod,expectedAnswer,options",
      '"What is 2+2?",multiple_choice,exact,4,"2|3|4|5"',
      '"Bad MCQ",multiple_choice,exact,9,"1|2"',
      '"What is 2+2?",short_answer,exact,4,',
      '"Unsupported",unknown_type,exact,x,',
      '"Manual",short_answer,manual,x,',
    ].join("\n");

    const res = parseAndValidateQuestionCsv(csv, ["Existing question?"]);
    expect(res.rows).toHaveLength(5);
    expect(res.rows[0]?.valid).toBe(true);
    expect(res.rows[1]?.valid).toBe(false);
    expect(res.rows[2]?.valid).toBe(false);
    expect(res.rows[3]?.valid).toBe(false);
    expect(res.rows[4]?.valid).toBe(false);
  });
});
