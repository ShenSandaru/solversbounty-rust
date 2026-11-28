"use client";

import type { CreateQuestionInput } from "@solversbounty/shared";
import { Download, Upload, X } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  parseAndValidateQuestionCsv,
  SAMPLE_QUESTION_CSV,
  type CsvImportResult,
} from "./csv-question-parser";

export function CsvImportPanel({
  existingPrompts,
  onImport,
  onClose,
}: {
  existingPrompts: string[];
  onImport: (questions: CreateQuestionInput[]) => void;
  onClose: () => void;
}) {
  const [result, setResult] = useState<CsvImportResult | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);

  const handleFile = async (file: File | null) => {
    if (!file) return;
    setFileName(file.name);
    const text = await file.text();
    setResult(parseAndValidateQuestionCsv(text, existingPrompts));
  };

  const downloadTemplate = () => {
    const blob = new Blob([SAMPLE_QUESTION_CSV], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "solversbounty-questions-template.csv";
    link.click();
    URL.revokeObjectURL(url);
  };

  const validQuestions =
    result?.rows
      .filter((r) => r.valid && r.question)
      .map((r) => r.question as CreateQuestionInput) ?? [];

  return (
    <div className="rounded-xl border border-[#9945FF]/40 bg-[#22222B] p-4 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h4 className="text-sm font-semibold">Import questions from CSV</h4>
          <p className="text-xs text-muted-foreground">
            Required columns: prompt, questionType, verificationMethod, expectedAnswer. Optional: options (pipe-separated for MCQ).
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button type="button" variant="outline" size="sm" onClick={downloadTemplate}>
            <Download className="mr-1.5 h-3.5 w-3.5" aria-hidden="true" />
            Sample CSV
          </Button>
          <Button type="button" variant="ghost" size="sm" onClick={onClose} aria-label="Close CSV import">
            <X className="h-4 w-4" aria-hidden="true" />
          </Button>
        </div>
      </div>

      <label
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          void handleFile(e.dataTransfer.files?.[0] ?? null);
        }}
        className="flex cursor-pointer flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border bg-[#111116]/60 px-4 py-6 text-center hover:border-[#9945FF]"
      >
        <Upload className="h-5 w-5 text-muted-foreground" aria-hidden="true" />
        <span className="text-xs text-foreground">
          {fileName ? `Selected: ${fileName}` : "Drop a .csv file here or click to browse"}
        </span>
        <input
          type="file"
          accept=".csv,text/csv"
          className="sr-only"
          onChange={(e) => void handleFile(e.target.files?.[0] ?? null)}
        />
      </label>

      {result?.headerError ? (
        <p className="text-sm text-destructive" role="alert">
          {result.headerError}
        </p>
      ) : null}

      {result && result.rows.length > 0 ? (
        <div className="space-y-3">
          <div className="max-h-60 overflow-auto rounded-lg border border-border">
            <table className="w-full text-left text-xs">
              <thead className="bg-[#111116] text-muted-foreground">
                <tr>
                  <th className="px-3 py-2">Row</th>
                  <th className="px-3 py-2">Prompt</th>
                  <th className="px-3 py-2">Type</th>
                  <th className="px-3 py-2">Method</th>
                  <th className="px-3 py-2">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/60">
                {result.rows.map((row) => (
                  <tr key={row.rowNumber}>
                    <td className="px-3 py-2 font-mono">{row.rowNumber}</td>
                    <td className="px-3 py-2 max-w-[16rem] truncate">{row.prompt || "—"}</td>
                    <td className="px-3 py-2">{row.questionType || "—"}</td>
                    <td className="px-3 py-2">{row.verificationMethod || "—"}</td>
                    <td className="px-3 py-2">
                      {row.valid ? (
                        <span className="text-[#14F195]">Valid</span>
                      ) : (
                        <span className="text-destructive">{row.errors.join(" ")}</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="flex flex-wrap items-center justify-end gap-2">
            <Button type="button" variant="outline" size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              type="button"
              size="sm"
              disabled={validQuestions.length === 0}
              onClick={() => {
                onImport(validQuestions);
                onClose();
              }}
            >
              Import {validQuestions.length} valid {validQuestions.length === 1 ? "question" : "questions"}
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
