"use client";

import type { CreateQuestionInput } from "@solversbounty/shared";
import { Plus, Upload } from "lucide-react";
import { useState } from "react";
import { useFieldArray, type UseFormReturn } from "react-hook-form";
import { Button } from "@/components/ui/button";
import type { CreateBountyFormValues } from "./create-bounty-schema";
import { CsvImportPanel } from "./csv-import-panel";
import { QuestionEditor } from "./question-editor";

export function QuestionListEditor({
  form,
}: {
  form: UseFormReturn<CreateBountyFormValues>;
}) {
  const [showCsvImport, setShowCsvImport] = useState(false);
  const { fields, append, remove, swap } = useFieldArray({
    control: form.control,
    name: "questions",
    keyName: "fieldKey",
  });

  const syncOrderIndexes = (count: number) => {
    for (let idx = 0; idx < count; idx += 1) {
      form.setValue(`questions.${idx}.orderIndex`, idx, {
        shouldDirty: true,
        shouldValidate: false,
      });
    }
  };

  const handleAdd = () => {
    append({
      orderIndex: fields.length,
      questionType: "short_answer",
      prompt: "",
      expectedAnswer: "",
      verificationMethod: "exact",
    });
  };

  const handleMoveUp = (index: number) => {
    if (index <= 0) return;
    swap(index, index - 1);
    syncOrderIndexes(fields.length);
  };

  const handleMoveDown = (index: number) => {
    if (index >= fields.length - 1) return;
    swap(index, index + 1);
    syncOrderIndexes(fields.length);
  };

  const handleRemove = (index: number) => {
    if (fields.length <= 1) return;
    remove(index);
    syncOrderIndexes(fields.length - 1);
  };

  const rootError = form.formState.errors.questions?.message;

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h3 className="text-base font-semibold">Challenge questions</h3>
          <p className="text-xs text-muted-foreground">
            Add one or more questions. Order is preserved for solvers.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => setShowCsvImport((prev) => !prev)}
          >
            <Upload className="mr-1.5 h-4 w-4" aria-hidden="true" />
            Import CSV
          </Button>
          <Button type="button" variant="outline" size="sm" onClick={handleAdd}>
            <Plus className="mr-1.5 h-4 w-4" aria-hidden="true" />
            Add Question
          </Button>
        </div>
      </div>

      {showCsvImport ? (
        <CsvImportPanel
          existingPrompts={(form.getValues("questions") ?? []).map((q) => q.prompt ?? "")}
          onImport={(imported: CreateQuestionInput[]) => {
            const startIndex = fields.length;
            imported.forEach((q, offset) => {
              append({
                ...q,
                orderIndex: startIndex + offset,
              });
            });
          }}
          onClose={() => setShowCsvImport(false)}
        />
      ) : null}

      {typeof rootError === "string" ? (
        <p className="text-sm text-destructive" role="alert">
          {rootError}
        </p>
      ) : null}

      <div className="space-y-4">
        {fields.map((field, index) => (
          <QuestionEditor
            key={field.fieldKey}
            form={form}
            index={index}
            total={fields.length}
            onMoveUp={() => handleMoveUp(index)}
            onMoveDown={() => handleMoveDown(index)}
            onRemove={() => handleRemove(index)}
          />
        ))}
      </div>
    </div>
  );
}
