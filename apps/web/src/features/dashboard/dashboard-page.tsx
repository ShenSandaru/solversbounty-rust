"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useSyncExternalStore } from "react";
import { Button } from "@/components/ui/button";
import { PageContainer } from "@/components/page-container";
import { PageHeader } from "@/components/page-header";
import { useAuthSession } from "@/features/auth/auth-session";
import { WalletBalance } from "@/features/auth/wallet-balance";
import {
  readOnboardingJourney,
  subscribeOnboardingJourney,
  writeOnboardingJourney,
} from "@/features/marketing/journey-preference";
import { CreatorBountyList } from "./creator-bounty-list";
import { SolverActivityPanel } from "./solver-activity-panel";

export function DashboardPage() {
  const { session } = useAuthSession();
  const searchParams = useSearchParams();
  const activeTab = searchParams.get("tab") === "profile" ? "profile" : "overview";
  const mode = useSyncExternalStore(subscribeOnboardingJourney, readOnboardingJourney, () => null);
  const walletCard = session ? (
    <div className="mt-6">
      <WalletBalance walletAddress={session.walletAddress} role={session.role} variant="card" />
    </div>
  ) : null;

  const tabs = (
    <div className="mt-5 inline-flex rounded-lg border border-border bg-secondary/40 p-1" role="tablist" aria-label="Dashboard sections">
      <Link
        href="/dashboard"
        role="tab"
        aria-selected={activeTab === "overview"}
        className={`rounded-md px-3.5 py-1.5 text-xs font-medium transition-colors ${
          activeTab === "overview" ? "bg-brand-green/15 text-foreground" : "text-muted-foreground hover:text-foreground"
        }`}
      >
        {mode === "creator" ? "My bounties" : mode === "solver" ? "My activity" : "Overview"}
      </Link>
      <Link
        href="/dashboard?tab=profile"
        role="tab"
        aria-selected={activeTab === "profile"}
        className={`rounded-md px-3.5 py-1.5 text-xs font-medium transition-colors ${
          activeTab === "profile" ? "bg-brand-green/15 text-foreground" : "text-muted-foreground hover:text-foreground"
        }`}
      >
        Profile
      </Link>
    </div>
  );

  if (activeTab === "profile") {
    return (
      <PageContainer>
        <PageHeader
          eyebrow="Profile"
          title="Wallet & account"
          description="Your connected Solana wallet balance, role, and active workspace mode."
        />
        {tabs}
        {session ? (
          <div className="mt-6 grid gap-4">
            <WalletBalance walletAddress={session.walletAddress} role={session.role} variant="card" />
            <section className="surface grid gap-3 p-5 text-sm sm:grid-cols-2">
              <div>
                <p className="text-xs text-muted-foreground">Connected wallet</p>
                <p className="mt-1 break-all font-mono text-xs">{session.walletAddress}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">Account role</p>
                <p className="mt-1 font-medium capitalize">{session.role}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">Workspace mode</p>
                <p className="mt-1 font-medium capitalize">{mode ?? "Not selected"}</p>
              </div>
              <div className="flex flex-wrap items-end gap-2">
                <Button size="sm" variant="outline" asChild>
                  <Link href="/bounties">Marketplace</Link>
                </Button>
                <Button size="sm" variant="outline" asChild>
                  <Link href="/bounties/create">Create bounty</Link>
                </Button>
              </div>
            </section>
          </div>
        ) : (
          <p className="surface mt-6 p-5 text-sm text-muted-foreground">
            Connect and sign in with Phantom in the top-right header to view your wallet balance and profile.
          </p>
        )}
      </PageContainer>
    );
  }

  if (mode === "solver") {
    return (
      <PageContainer>
        <PageHeader
          eyebrow="Solver mode"
          title="My activity"
          description="Your solving sessions and payouts will appear here. Until then, use the marketplace to find open challenges."
        />
        {tabs}
        {walletCard}
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
        {tabs}
        {walletCard}
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
