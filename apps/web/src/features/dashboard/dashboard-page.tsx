"use client";

import Link from "next/link";
import { useSyncExternalStore } from "react";
import { Button } from "@/components/ui/button";
import { PageContainer } from "@/components/page-container";
import { PageHeader } from "@/components/page-header";
import {
  readOnboardingJourney,
  subscribeOnboardingJourney,
  writeOnboardingJourney,
} from "@/features/marketing/journey-preference";
import { CreatorBountyList } from "./creator-bounty-list";
import { SolverActivityPanel } from "./solver-activity-panel";

export function DashboardPage() {
  const mode = useSyncExternalStore(subscribeOnboardingJourney, readOnboardingJourney, () => null);

  if (mode === "solver") {
    return (
      <PageContainer>
        <PageHeader
          eyebrow="Solver mode"
          title="My activity"
          description="Your solving sessions and payouts will appear here. Until then, use the marketplace to find open challenges."
        />
        <div className="mt-6">
          <SolverActivityPanel />
        </div>
      </PageContainer>
    );
  }

  if (mode === "creator") {
    return (
      <PageContainer>
        <PageHeader
          eyebrow="Creator mode"
          title="My bounties"
          description="Live list of bounties you own. Draft, fund, activate, then open owner analytics. Organizations and individuals use this same workspace."
        />
        <div className="mt-4 flex flex-wrap gap-3">
          <Button asChild>
            <Link href="/bounties/create">Create bounty</Link>
          </Button>
          <Button variant="outline" asChild>
            <Link href="/research">Owner analytics</Link>
          </Button>
        </div>
        <section className="mt-6" aria-labelledby="owned-bounties">
          <h2 id="owned-bounties" className="text-xl font-semibold">
            Owned bounties
          </h2>
          <CreatorBountyList />
        </section>
      </PageContainer>
    );
  }

  return (
    <PageContainer>
      <PageHeader
        eyebrow="Workspace"
        title="Choose a mode"
        description="Pick Solver or Creator so the sidebar shows the tools for that job. You can switch any time."
      />
      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        <ModeCard
          title="Solver"
          body="Browse challenges, consent, submit answers, and earn the configured reward when eligible."
          href="/bounties"
          mode="solver"
          cta="Use solver tools"
        />
        <ModeCard
          title="Creator"
          body="Draft challenges, fund the custodial Devnet treasury, and read aggregate analytics for bounties you own."
          href="/dashboard"
          mode="creator"
          cta="Use creator tools"
        />
      </div>
    </PageContainer>
  );
}

function ModeCard({
  title,
  body,
  href,
  mode,
  cta,
}: {
  title: string;
  body: string;
  href: string;
  mode: "solver" | "creator";
  cta: string;
}) {
  return (
    <article className="surface flex h-full flex-col p-6">
      <h2 className="text-xl font-semibold">{title}</h2>
      <p className="mt-2 flex-1 text-sm leading-6 text-muted-foreground">{body}</p>
      <Button className="mt-6 w-fit" asChild>
        <Link
          href={href}
          onClick={() => {
            writeOnboardingJourney(mode);
          }}
        >
          {cta}
        </Link>
      </Button>
    </article>
  );
}
