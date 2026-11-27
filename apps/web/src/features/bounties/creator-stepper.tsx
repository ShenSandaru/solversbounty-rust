"use client";

import Link from "next/link";

export type CreatorStepId = "sign_in" | "draft" | "fund" | "activate";

type StepDef = {
  id: CreatorStepId;
  number: number;
  label: string;
  hint: string;
  href?: string;
};

export function CreatorStepper({
  current,
  isSignedIn,
  bountyId,
  isActive = false,
}: {
  current: CreatorStepId;
  isSignedIn: boolean;
  bountyId?: string;
  isActive?: boolean;
}) {
  const steps: StepDef[] = [
    {
      id: "sign_in",
      number: 1,
      label: "Sign in",
      hint: "Phantom wallet",
    },
    {
      id: "draft",
      number: 2,
      label: "Define challenge",
      hint: "Question, answer & reward",
      href: bountyId ? `/bounties/${bountyId}/edit` : "/bounties/create",
    },
    {
      id: "fund",
      number: 3,
      label: "Fund on Devnet",
      hint: "Pool + 5% commission",
      href: bountyId ? `/bounties/${bountyId}/fund` : undefined,
    },
    {
      id: "activate",
      number: 4,
      label: "Activate & live",
      hint: "Open to solvers",
      href: isActive && bountyId ? `/bounties/${bountyId}` : undefined,
    },
  ];

  const order: CreatorStepId[] = ["sign_in", "draft", "fund", "activate"];
  const effectiveCurrent: CreatorStepId = !isSignedIn ? "sign_in" : current;
  const currentIndex = order.indexOf(effectiveCurrent);

  return (
    <nav aria-label="Challenge creation progress" className="mb-6">
      <ol className="grid gap-2 sm:grid-cols-4">
        {steps.map((step, index) => {
          const isDone = isActive ? true : index < currentIndex || (step.id === "sign_in" && isSignedIn && currentIndex > 0);
          const isCurrent = !isActive && index === currentIndex;

          const cardClass = isCurrent
            ? "border-brand-green/60 bg-brand-green/10 text-foreground"
            : isDone
              ? "border-brand-green/30 bg-card/90 text-foreground"
              : "border-border bg-card/40 text-muted-foreground";

          const badgeClass = isCurrent
            ? "bg-brand-gradient text-[#111116]"
            : isDone
              ? "bg-brand-green/20 text-brand-green"
              : "bg-secondary text-muted-foreground";

          const content = (
            <div className={`flex items-start gap-3 rounded-xl border px-3.5 py-3 transition-colors ${cardClass}`}>
              <span
                className={`mt-0.5 grid size-6 shrink-0 place-items-center rounded-full font-mono text-xs font-bold ${badgeClass}`}
                aria-hidden="true"
              >
                {isDone ? "✓" : step.number}
              </span>
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">{step.label}</p>
                <p className="truncate text-xs text-muted-foreground">{step.hint}</p>
              </div>
            </div>
          );

          return (
            <li key={step.id} aria-current={isCurrent ? "step" : undefined}>
              {step.href && !isCurrent && (isDone || bountyId) ? (
                <Link href={step.href} className="block rounded-xl focus-visible:outline-2 focus-visible:outline-brand-green">
                  {content}
                </Link>
              ) : (
                content
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
