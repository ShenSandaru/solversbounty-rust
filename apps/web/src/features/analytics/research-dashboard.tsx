"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "next/navigation";
import { EmptyState } from "@/components/empty-state";
import { Button } from "@/components/ui/button";
import { StatisticCard } from "@/components/statistic-card";
import { apiBaseUrl } from "@/features/auth/auth-client";
import { useAuthSession } from "@/features/auth/auth-session";
import { BountyClientError, listMyBounties, type BountyDraft } from "@/features/bounties/bounty-api";
import {
  accuracyDisplay,
  fetchBountyAnalytics,
  formatAverageCompletionMs,
  type BountyAnalytics,
} from "./analytics-api";

export function ResearchDashboard() {
  const { session } = useAuthSession();
  const searchParams = useSearchParams();
  const requestedId = searchParams.get("bounty");
  const [bounties, setBounties] = useState<BountyDraft[] | null>(null);
  const [listError, setListError] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string>("");
  const [analytics, setAnalytics] = useState<BountyAnalytics | null>(null);
  const [analyticsError, setAnalyticsError] = useState<string | null>(null);
  const [analyticsLoading, setAnalyticsLoading] = useState(false);

  useEffect(() => {
    if (!session) {
      setBounties(null);
      return;
    }
    let active = true;
    listMyBounties(apiBaseUrl())
      .then((rows) => {
        if (!active) {
          return;
        }
        setBounties(rows);
        setListError(null);
      })
      .catch((caught: unknown) => {
        if (!active) {
          return;
        }
        setListError(caught instanceof BountyClientError ? caught.message : "Bounties could not be loaded.");
        setBounties([]);
      });
    return () => {
      active = false;
    };
  }, [session]);

  const selectable = useMemo(
    () => (bounties ?? []).filter((row) => row.status === "active" || row.status === "closed" || row.status === "funding_pending" || row.status === "draft"),
    [bounties],
  );

  useEffect(() => {
    if (!bounties) {
      return;
    }
    if (requestedId && bounties.some((row) => row.id === requestedId)) {
      setSelectedId(requestedId);
      return;
    }
    if (!selectedId && bounties[0]) {
      setSelectedId(bounties[0].id);
    }
  }, [bounties, requestedId, selectedId]);

  useEffect(() => {
    if (!session || !selectedId) {
      setAnalytics(null);
      return;
    }
    let active = true;
    setAnalyticsLoading(true);
    setAnalyticsError(null);
    fetchBountyAnalytics(apiBaseUrl(), selectedId)
      .then((payload) => {
        if (active) {
          setAnalytics(payload);
        }
      })
      .catch((caught: unknown) => {
        if (active) {
          setAnalytics(null);
          setAnalyticsError(
            caught instanceof BountyClientError ? caught.message : "Owner analytics could not be loaded.",
          );
        }
      })
      .finally(() => {
        if (active) {
          setAnalyticsLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, [session, selectedId]);

  if (!session) {
    return (
      <div className="mt-6">
        <EmptyState
          title="Sign in to read owner analytics"
          description="Aggregate research figures are available only for bounties you own. There is no public demo dataset and no separate admin analytics console in this MVP."
          action={
            <Button variant="outline" asChild>
              <Link href="/dashboard">Back to workspace</Link>
            </Button>
          }
        />
      </div>
    );
  }

  if (listError) {
    return (
      <p className="mt-6 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive" role="alert">
        {listError}
      </p>
    );
  }

  if (!bounties) {
    return (
      <div className="mt-6 h-32 animate-pulse rounded-xl border border-border bg-card" role="status">
        <span className="sr-only">Loading your bounties.</span>
      </div>
    );
  }

  if (selectable.length === 0) {
    return (
      <div className="mt-6">
        <EmptyState
          title="No owned bounties yet"
          description="Create and fund a challenge first. Owner analytics appear after solvers start consented sessions on a bounty you own."
          action={
            <Button asChild>
              <Link href="/bounties/create">Create a bounty</Link>
            </Button>
          }
        />
      </div>
    );
  }

  const selected = selectable.find((row) => row.id === selectedId) ?? selectable[0];

  return (
    <div className="mt-6 grid gap-6">
      <div className="rounded-xl border border-border bg-card/60 px-4 py-3 text-sm leading-6 text-muted-foreground" role="note">
        These figures are live aggregates for bounties you own: participants, correct and incorrect attempts,
        average completion time, and accuracy. Individual solver records and wallet addresses are not shown.
        Platform-wide admin analytics are not part of this screen.
      </div>
      <div>
        <p className="text-xs font-medium tracking-[0.12em] text-muted-foreground uppercase">Your bounties</p>
        <div className="mt-2 flex flex-wrap gap-2" role="group" aria-label="Owned bounty">
          {selectable.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-pressed={item.id === selected?.id}
              className={`rounded-md border px-3.5 py-1.5 text-sm transition-colors ${
                item.id === selected?.id
                  ? "border-brand-green bg-brand-green/10 font-medium text-foreground"
                  : "border-border bg-card text-muted-foreground hover:border-brand-green/50 hover:text-foreground"
              }`}
              onClick={() => setSelectedId(item.id)}
            >
              {item.title}
            </button>
          ))}
        </div>
      </div>
      {analyticsLoading ? (
        <div className="h-28 animate-pulse rounded-xl border border-border bg-card" role="status">
          <span className="sr-only">Loading analytics.</span>
        </div>
      ) : null}
      {analyticsError ? (
        <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive" role="alert">
          {analyticsError}
        </p>
      ) : null}
      {!analyticsLoading && analytics && analytics.participantCount === 0 ? (
        <EmptyState
          title="No solving sessions yet"
          description="Participants, accuracy, and completion time appear after solvers start sessions on this bounty."
        />
      ) : null}
      {!analyticsLoading && analytics && analytics.participantCount > 0 ? (
        <ResearchFigures analytics={analytics} />
      ) : null}
    </div>
  );
}

function ResearchFigures({ analytics }: { analytics: BountyAnalytics }) {
  const attempts = analytics.attemptCount;
  const incorrectShare = attempts === 0 ? 0 : (analytics.incorrectCount / attempts) * 100;
  const correctShare = attempts === 0 ? 0 : (analytics.correctCount / attempts) * 100;

  return (
    <>
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <StatisticCard label="Participants" value={analytics.participantCount} detail="Solving sessions started." />
        <StatisticCard
          label="Overall accuracy"
          value={accuracyDisplay(analytics.accuracy)}
          detail="Correct attempts divided by all attempts."
        />
        <StatisticCard label="Correct attempts" value={analytics.correctCount} detail="Server-verified correct submissions." />
        <StatisticCard
          label="Average completion time"
          value={formatAverageCompletionMs(analytics.averageCompletionTimeMs)}
          detail="Mean time to the first correct attempt."
        />
      </div>
      <section className="surface p-6">
        <h2 className="text-lg font-semibold">Attempt mix</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          {analytics.correctCount} correct and {analytics.incorrectCount} incorrect attempts ({analytics.attemptCount}{" "}
          total).
        </p>
        <div className="mt-5 h-4 overflow-hidden rounded-full bg-secondary" aria-hidden="true">
          <div className="flex h-full">
            <div className="bg-brand-green" style={{ width: `${correctShare}%` }} />
            <div className="bg-muted-foreground/40" style={{ width: `${incorrectShare}%` }} />
          </div>
        </div>
        <div className="mt-3 flex gap-5 text-xs text-muted-foreground">
          <span className="inline-flex items-center gap-1.5">
            <span className="size-2.5 rounded-full bg-brand-green" aria-hidden="true" />
            Correct {correctShare.toFixed(0)}%
          </span>
          <span className="inline-flex items-center gap-1.5">
            <span className="size-2.5 rounded-full bg-muted-foreground/40" aria-hidden="true" />
            Incorrect {incorrectShare.toFixed(0)}%
          </span>
        </div>
      </section>
    </>
  );
}
