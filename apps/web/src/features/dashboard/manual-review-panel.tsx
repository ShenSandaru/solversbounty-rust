"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/empty-state";
import { LoadingState } from "@/components/loading-state";
import { apiBaseUrl } from "@/features/auth/auth-client";
import { useAuthSession } from "@/features/auth/auth-session";

export type PendingReviewItem = {
  attemptId: string;
  bountyId: string;
  bountyTitle: string;
  questionPrompt: string;
  submittedAnswer: string;
  submittedAt: string;
  manualReviewStatus: "pending" | "approved" | "rejected";
};

export function ManualReviewPanel() {
  const { session } = useAuthSession();
  const [items, setItems] = useState<PendingReviewItem[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  useEffect(() => {
    if (!session) return;
    let active = true;
    fetch(`${apiBaseUrl()}/reviews/pending`, { credentials: "include" })
      .then(async (res) => {
        if (!res.ok) throw new Error("Could not load pending reviews.");
        return (await res.json()) as PendingReviewItem[];
      })
      .then((rows) => {
        if (active) setItems(rows);
      })
      .catch((err: unknown) => {
        if (active) setError(err instanceof Error ? err.message : "Could not load pending reviews.");
      });
    return () => {
      active = false;
    };
  }, [session]);

  const handleDecision = async (attemptId: string, decision: "approved" | "rejected") => {
    if (!session) return;
    const confirmed = window.confirm(
      decision === "approved"
        ? "Approve this submission? If eligible, this will reserve the solver's reward."
        : "Reject this submission? No reward will be allocated.",
    );
    if (!confirmed) return;

    setBusyId(attemptId);
    setError(null);
    try {
      const res = await fetch(`${apiBaseUrl()}/submissions/${attemptId}/review`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
          "X-CSRF-Token": session.csrfToken,
        },
        body: JSON.stringify({ decision }),
      });
      if (!res.ok) {
        const body = (await res.json().catch(() => null)) as { message?: string } | null;
        throw new Error(body?.message ?? "Review failed.");
      }
      setItems((prev) => (prev ? prev.filter((item) => item.attemptId !== attemptId) : prev));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Review failed.");
    } finally {
      setBusyId(null);
    }
  };

  if (!session) {
    return null;
  }

  return (
    <section className="mt-8 space-y-3" aria-labelledby="manual-reviews-heading">
      <h2 id="manual-reviews-heading" className="text-lg font-semibold">
        Pending manual reviews
      </h2>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
      {items === null ? (
        <LoadingState label="Loading pending reviews" />
      ) : items.length === 0 ? (
        <EmptyState
          title="No pending reviews"
          description="Submissions for manual-review questions will appear here."
        />
      ) : (
        <div className="space-y-3">
          {items.map((item) => (
            <div key={item.attemptId} className="surface rounded-xl border border-border bg-[#22222B] p-4 space-y-2">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <span className="text-sm font-semibold">{item.bountyTitle}</span>
                <span className="text-xs text-muted-foreground">
                  {new Date(item.submittedAt).toLocaleString()}
                </span>
              </div>
              <p className="text-xs text-muted-foreground">{item.questionPrompt}</p>
              <p className="rounded bg-[#111116] px-3 py-2 text-sm font-mono text-foreground">
                {item.submittedAnswer}
              </p>
              <div className="flex items-center justify-end gap-2 pt-1">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={busyId === item.attemptId}
                  onClick={() => void handleDecision(item.attemptId, "rejected")}
                >
                  Reject
                </Button>
                <Button
                  type="button"
                  size="sm"
                  disabled={busyId === item.attemptId}
                  onClick={() => void handleDecision(item.attemptId, "approved")}
                >
                  Approve
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
