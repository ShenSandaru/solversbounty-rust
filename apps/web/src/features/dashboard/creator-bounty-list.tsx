"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { apiBaseUrl } from "@/features/auth/auth-client";
import { useAuthSession } from "@/features/auth/auth-session";
import { BountyClientError, listMyBounties, type BountyDraft } from "@/features/bounties/bounty-api";
import { formatDeadline } from "@/lib/dates";
import { closeReasonLabels, statusLabels } from "@/lib/labels";

export function CreatorBountyList() {
  const { session } = useAuthSession();
  const [loaded, setLoaded] = useState<{ wallet: string; rows: BountyDraft[] } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session) {
      return;
    }
    const wallet = session.walletAddress;
    let active = true;
    listMyBounties(apiBaseUrl())
      .then((rows) => {
        if (active) {
          setLoaded({ wallet, rows });
        }
      })
      .catch((caught: unknown) => {
        if (active) {
          setError(caught instanceof BountyClientError ? caught.message : "Bounties could not be loaded.");
        }
      });
    return () => {
      active = false;
    };
  }, [session]);

  const rows = session && loaded?.wallet === session.walletAddress ? loaded.rows : null;

  if (!session) {
    return (
      <div className="mt-4 rounded-xl border border-dashed border-input bg-card/60 px-5 py-8">
        <p className="text-sm text-muted-foreground">Sign in with Phantom to manage your bounties.</p>
      </div>
    );
  }
  if (error) {
    return (
      <p
        className="mt-4 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
        role="alert"
      >
        {error}
      </p>
    );
  }
  if (!rows) {
    return (
      <div className="mt-4 h-24 animate-pulse rounded-xl border border-border bg-card" role="status">
        <span className="sr-only">Loading your bounties.</span>
      </div>
    );
  }
  if (rows.length === 0) {
    return (
      <div className="mt-4 rounded-xl border border-dashed border-input bg-card/60 px-5 py-8">
        <p className="text-sm text-muted-foreground">No bounties yet.</p>
        <Button className="mt-4" asChild>
          <Link href="/bounties/create">Create your first challenge</Link>
        </Button>
      </div>
    );
  }

  return (
    <div className="mt-4 surface overflow-x-auto">
      <table className="w-full min-w-[44rem] text-left text-sm">
        <caption className="sr-only">Your owned bounties</caption>
        <thead className="bg-secondary/60 text-xs tracking-wider text-muted-foreground uppercase">
          <tr>
            <th className="px-4 py-3 font-medium">Title</th>
            <th className="px-4 py-3 font-medium">Status</th>
            <th className="px-4 py-3 font-medium">Deadline</th>
            <th className="px-4 py-3 font-medium">Next step</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((bounty) => (
            <tr key={bounty.id} className="border-t border-border transition-colors hover:bg-card-elevated">
              <td className="px-4 py-3">
                <span className="font-medium">{bounty.title}</span>
              </td>
              <td className="px-4 py-3">
                <span className="rounded-full border border-input bg-secondary px-2.5 py-1 text-xs font-medium">
                  {bounty.closeReason && bounty.closeReason in closeReasonLabels
                    ? closeReasonLabels[bounty.closeReason as keyof typeof closeReasonLabels]
                    : statusLabels[bounty.status]}
                </span>
              </td>
              <td className="px-4 py-3">{formatDeadline(bounty.deadline)}</td>
              <td className="px-4 py-3">
                <div className="flex flex-wrap gap-x-3 gap-y-1">{nextActions(bounty)}</div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function nextActions(bounty: BountyDraft) {
  const links: { href: string; label: string }[] = [];
  if (bounty.status === "draft") {
    links.push({ href: `/bounties/${bounty.id}/edit`, label: "Edit" });
    links.push({ href: `/bounties/${bounty.id}/fund`, label: "Fund" });
  } else if (bounty.status === "funding_pending") {
    links.push({ href: `/bounties/${bounty.id}/fund`, label: "Continue funding" });
    links.push({ href: `/bounties/${bounty.id}/edit`, label: "Edit" });
  } else if (bounty.status === "active") {
    links.push({ href: `/bounties/${bounty.id}`, label: "View challenge" });
    links.push({ href: `/research?bounty=${bounty.id}`, label: "View analytics" });
  } else {
    links.push({ href: `/bounties/${bounty.id}`, label: "View challenge" });
    links.push({ href: `/research?bounty=${bounty.id}`, label: "View analytics" });
  }
  return links.map((link) => (
    <Link key={link.href + link.label} href={link.href} className="font-medium text-brand-green hover:underline">
      {link.label}
    </Link>
  ));
}
