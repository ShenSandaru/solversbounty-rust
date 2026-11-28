"use client";

import { Plus, Trash2 } from "lucide-react";
import { useWatch, type UseFormReturn } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { CreateBountyFormValues } from "./create-bounty-schema";

export function McqOptionsEditor({
  form,
  questionIndex,
}: {
  form: UseFormReturn<CreateBountyFormValues>;
  questionIndex: number;
}) {
  const options =
    useWatch({
      control: form.control,
      name: `questions.${questionIndex}.options`,
    }) ?? [];

  const fieldError = form.formState.errors.questions?.[questionIndex]?.options;
  const errorMessage =
    typeof fieldError?.message === "string"
      ? fieldError.message
      : Array.isArray(fieldError)
        ? fieldError.find((item) => item && typeof item.message === "string")?.message
        : undefined;

  const updateOption = (optIndex: number, value: string) => {
    const next = [...options];
    next[optIndex] = value;
    form.setValue(`questions.${questionIndex}.options`, next, {
      shouldValidate: true,
      shouldDirty: true,
    });
  };

  const addOption = () => {
    form.setValue(`questions.${questionIndex}.options`, [...options, ""], {
      shouldValidate: true,
      shouldDirty: true,
    });
  };

  const removeOption = (optIndex: number) => {
    const next = options.filter((_, idx) => idx !== optIndex);
    form.setValue(`questions.${questionIndex}.options`, next, {
      shouldValidate: true,
      shouldDirty: true,
    });
  };

  return (
    <div className="space-y-2 rounded-lg border border-border/70 bg-[#111116]/60 p-3">
      <div className="flex items-center justify-between gap-2">
        <Label className="text-xs font-medium text-muted-foreground">
          Multiple-choice options (at least 2 distinct options)
        </Label>
        <Button type="button" variant="outline" size="sm" onClick={addOption}>
          <Plus className="mr-1 h-3.5 w-3.5" aria-hidden="true" />
          Add option
        </Button>
      </div>
      <div className="space-y-2">
        {options.map((opt, optIndex) => (
          <div key={optIndex} className="flex items-center gap-2">
            <Input
              value={opt}
              onChange={(e) => updateOption(optIndex, e.target.value)}
              placeholder={`Option ${optIndex + 1}`}
              aria-label={`Question ${questionIndex + 1} option ${optIndex + 1}`}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => removeOption(optIndex)}
              disabled={options.length <= 2}
              aria-label={`Remove option ${optIndex + 1}`}
            >
              <Trash2 className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
            </Button>
          </div>
        ))}
      </div>
      {errorMessage ? (
        <p className="text-xs text-destructive" role="alert">
          {errorMessage}
        </p>
      ) : null}
    </div>
  );
}
