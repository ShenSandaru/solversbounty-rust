"use client";

import Link from "next/link";
import { useEffect, useId, useState } from "react";
import { Button } from "@/components/ui/button";
import type { OnboardingJourney } from "@/features/marketing/journey-preference";

type GuideStep = {
  title: string;
  detail: string;
  href: string;
  cta: string;
};

const creatorSteps: GuideStep[] = [
  {
    title: "1. Connect & sign in",
    detail: "Use the Phantom button in the top-right header. Signing a login message does not send SOL.",
    href: "/bounties/create",
    cta: "Go to Create",
  },
  {
    title: "2. Draft your challenge",
    detail: "Set the question, encrypted expected answer, reward per solver, and max solvers.",
    href: "/bounties/create",
    cta: "Create draft",
  },
  {
    title: "3. Fund & activate",
    detail: "Send two Devnet transfers (reward pool + separate 5% commission) to the custodial treasury, then activate.",
    href: "/dashboard",
    cta: "My bounties",
  },
  {
    title: "4. Review aggregate insights",
    detail: "Inspect participant count, correctness, attempt count, and completion time.",
    href: "/research",
    cta: "Research",
  },
];

const solverSteps: GuideStep[] = [
  {
    title: "1. Browse open challenges",
    detail: "Explore funded cognitive challenges in the marketplace. No participation fee.",
    href: "/bounties",
    cta: "Marketplace",
  },
  {
    title: "2. Sign in & consent",
    detail: "Connect Phantom and accept the research recording consent before starting a session.",
    href: "/bounties",
    cta: "Pick a challenge",
  },
  {
    title: "3. Submit & earn SOL",
    detail: "The backend verifies your answer and pays eligible correct solvers on Solana Devnet.",
    href: "/dashboard",
    cta: "My activity",
  },
];

export function SidebarGuideCard({
  mode,
  isSignedIn,
  onOpenGuide,
}: {
  mode: OnboardingJourney | null;
  isSignedIn: boolean;
  onOpenGuide: () => void;
}) {
  const isCreator = mode === "creator";
  const nextAction = !mode
    ? { label: "Choose your mode", href: "/get-started", note: "Pick Solver or Creator to focus navigation." }
    : isCreator
      ? isSignedIn
        ? { label: "Create a challenge", href: "/bounties/create", note: "Draft, fund on Devnet, and activate." }
        : { label: "Start a draft", href: "/bounties/create", note: "Sign in with Phantom in the header to save drafts." }
      : { label: "Browse challenges", href: "/bounties", note: "Solve open challenges and earn Devnet SOL." };

  return (
    <div className="mt-auto rounded-xl border border-border bg-background/60 p-3.5">
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs font-semibold tracking-[0.12em] text-brand-green uppercase">Quick guide</p>
        <button
          type="button"
          onClick={onOpenGuide}
          className="text-xs font-medium text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
        >
          How it works
        </button>
      </div>
      <p className="mt-2 text-xs leading-5 text-muted-foreground">{nextAction.note}</p>
      <Button size="sm" variant="outline" className="mt-3 w-full" asChild>
        <Link href={nextAction.href}>{nextAction.label}</Link>
      </Button>
    </div>
  );
}

export function SystemGuideModal({
  open,
  mode,
  onClose,
}: {
  open: boolean;
  mode: OnboardingJourney | null;
  onClose: () => void;
}) {
  const titleId = useId();
  const [selectedTab, setSelectedTab] = useState<"creator" | "solver" | null>(null);
  const tab = selectedTab ?? (mode === "solver" ? "solver" : "creator");

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose, open]);

  if (!open) {
    return null;
  }

  const handleClose = () => {
    setSelectedTab(null);
    onClose();
  };

  const steps = tab === "creator" ? creatorSteps : solverSteps;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4" role="dialog" aria-modal="true" aria-labelledby={titleId}>
      <button
        type="button"
        className="absolute inset-0 bg-black/70 backdrop-blur-xs"
        aria-label="Close guide"
        onClick={handleClose}
      />
      <div className="surface relative z-10 w-full max-w-lg p-6 shadow-2xl">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-xs font-semibold tracking-[0.14em] text-brand-green uppercase">System guide</p>
            <h2 id={titleId} className="mt-1 text-xl font-semibold">
              How SolversBounty works
            </h2>
          </div>
          <Button type="button" variant="ghost" size="sm" onClick={handleClose}>
            Close
          </Button>
        </div>

        <div className="mt-4 inline-flex rounded-lg border border-border bg-secondary/40 p-1" role="tablist" aria-label="Guide perspective">
          <button
            type="button"
            role="tab"
            aria-selected={tab === "creator"}
            onClick={() => setSelectedTab("creator")}
            className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
              tab === "creator" ? "bg-brand-green/15 text-foreground" : "text-muted-foreground hover:text-foreground"
            }`}
          >
            Creator flow (4 steps)
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === "solver"}
            onClick={() => setSelectedTab("solver")}
            className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
              tab === "solver" ? "bg-brand-green/15 text-foreground" : "text-muted-foreground hover:text-foreground"
            }`}
          >
            Solver flow (3 steps)
          </button>
        </div>

        <ol className="mt-5 grid gap-3">
          {steps.map((step) => (
            <li key={step.title} className="flex items-start justify-between gap-3 rounded-xl border border-border bg-background/50 p-3.5">
              <div className="min-w-0">
                <p className="text-sm font-medium">{step.title}</p>
                <p className="mt-1 text-xs leading-5 text-muted-foreground">{step.detail}</p>
              </div>
              <Button size="sm" variant="outline" asChild className="shrink-0">
                <Link href={step.href} onClick={handleClose}>
                  {step.cta}
                </Link>
              </Button>
            </li>
          ))}
        </ol>
      </div>
    </div>
  );
}
