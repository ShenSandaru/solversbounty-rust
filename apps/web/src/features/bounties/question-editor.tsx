"use client";

import type { QuestionType, VerificationMethod } from "@solversbounty/shared";
import { ArrowDown, ArrowUp, Trash2 } from "lucide-react";
import { useWatch, type UseFormReturn } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import {
  questionTypeLabels,
  selectableVerificationMethods,
  verificationMethodLabels,
} from "@/lib/labels";
import type { CreateBountyFormValues } from "./create-bounty-schema";
import { McqOptionsEditor } from "./mcq-options-editor";

const questionTypes = Object.entries(questionTypeLabels) as [QuestionType, string][];

export function QuestionEditor({
  form,
  index,
  total,
  onMoveUp,
  onMoveDown,
  onRemove,
}: {
  form: UseFormReturn<CreateBountyFormValues>;
  index: number;
  total: number;
  onMoveUp: () => void;
  onMoveDown: () => void;
  onRemove: () => void;
}) {
  const questionType =
    useWatch({
      control: form.control,
      name: `questions.${index}.questionType`,
    }) ?? "short_answer";

  const errors = form.formState.errors.questions?.[index];

  const handleTypeChange = (nextType: QuestionType) => {
    form.setValue(`questions.${index}.questionType`, nextType, {
      shouldValidate: true,
      shouldDirty: true,
    });
    if (nextType === "multiple_choice") {
      const current = form.getValues(`questions.${index}.options`);
      if (!current || current.length < 2) {
        form.setValue(`questions.${index}.options`, ["", ""], {
          shouldValidate: false,
          shouldDirty: true,
        });
      }
    } else {
      form.setValue(`questions.${index}.options`, undefined, {
        shouldValidate: true,
        shouldDirty: true,
      });
    }
  };

  return (
    <div className="surface space-y-4 rounded-xl border border-border bg-[#22222B] p-4 sm:p-5">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border/60 pb-3">
        <div className="flex items-center gap-2">
          <span className="rounded-full border border-[#9945FF]/40 bg-[#9945FF]/15 px-2.5 py-0.5 text-xs font-medium text-[#d4b8ff]">
            Question {index + 1}
          </span>
          <span className="text-xs text-muted-foreground">
            {questionTypeLabels[questionType]}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onMoveUp}
            disabled={index === 0}
            aria-label={`Move question ${index + 1} up`}
          >
            <ArrowUp className="h-4 w-4" aria-hidden="true" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onMoveDown}
            disabled={index === total - 1}
            aria-label={`Move question ${index + 1} down`}
          >
            <ArrowDown className="h-4 w-4" aria-hidden="true" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onRemove}
            disabled={total <= 1}
            aria-label={`Remove question ${index + 1}`}
          >
            <Trash2 className="h-4 w-4 text-destructive" aria-hidden="true" />
          </Button>
        </div>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5">
          <Label htmlFor={`q-${index}-type`}>Question type</Label>
          <Select
            id={`q-${index}-type`}
            value={questionType}
            onChange={(e) => handleTypeChange(e.target.value as QuestionType)}
          >
            {questionTypes.map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </Select>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor={`q-${index}-method`}>Verification method</Label>
          <Select
            id={`q-${index}-method`}
            {...form.register(`questions.${index}.verificationMethod`)}
          >
            {selectableVerificationMethods.map((method: VerificationMethod) => (
              <option key={method} value={method}>
                {verificationMethodLabels[method]}
              </option>
            ))}
          </Select>
          {errors?.verificationMethod?.message ? (
            <p className="text-xs text-destructive" role="alert">
              {errors.verificationMethod.message}
            </p>
          ) : null}
        </div>
      </div>

      <div className="space-y-1.5">
        <Label htmlFor={`q-${index}-prompt`}>Question prompt</Label>
        <Textarea
          id={`q-${index}-prompt`}
          rows={3}
          placeholder="Enter the challenge question shown to solvers."
          {...form.register(`questions.${index}.prompt`)}
        />
        {errors?.prompt?.message ? (
          <p className="text-xs text-destructive" role="alert">
            {errors.prompt.message}
          </p>
        ) : null}
      </div>

      {questionType === "multiple_choice" ? (
        <McqOptionsEditor form={form} questionIndex={index} />
      ) : null}

      <div className="space-y-1.5">
        <Label htmlFor={`q-${index}-answer`}>Expected answer</Label>
        <Input
          id={`q-${index}-answer`}
          placeholder={
            questionType === "multiple_choice"
              ? "Must match one of the options above"
              : "Protected expected answer (never exposed publicly)"
          }
          {...form.register(`questions.${index}.expectedAnswer`)}
        />
        {errors?.expectedAnswer?.message ? (
          <p className="text-xs text-destructive" role="alert">
            {errors.expectedAnswer.message}
          </p>
        ) : null}
      </div>
    </div>
  );
}
