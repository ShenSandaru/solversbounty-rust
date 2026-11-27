"use client";

import type { OnboardingJourney } from "@/features/marketing/journey-preference";
import { writeOnboardingJourney } from "@/features/marketing/journey-preference";

const modes: { value: OnboardingJourney; label: string }[] = [
  { value: "solver", label: "Solver" },
  { value: "creator", label: "Creator" },
];

export function ModeSwitch({
  mode,
  className = "",
}: {
  mode: OnboardingJourney | null;
  className?: string;
}) {
  return (
    <div className={className}>
      <p className="mb-2 px-1 text-xs font-medium tracking-[0.12em] text-muted-foreground uppercase">Mode</p>
      <div
        className="grid grid-cols-2 gap-1 rounded-lg border border-border bg-secondary/40 p-1"
        role="group"
        aria-label="Workspace mode"
      >
        {modes.map((item) => {
          const selected = mode === item.value;
          return (
            <button
              key={item.value}
              type="button"
              aria-pressed={selected}
              className={`rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                selected
                  ? "bg-card text-foreground shadow-sm ring-1 ring-brand-green/50"
                  : "text-muted-foreground hover:bg-card/60 hover:text-foreground"
              }`}
              onClick={() => writeOnboardingJourney(item.value)}
            >
              {item.label}
            </button>
          );
        })}
      </div>
      {!mode ? (
        <p className="mt-2 px-1 text-xs leading-5 text-muted-foreground">
          Choose Solver or Creator to focus the navigation.
        </p>
      ) : null}
    </div>
  );
}
