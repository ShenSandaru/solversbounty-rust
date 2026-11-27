"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/empty-state";
import { useAuthSession } from "@/features/auth/auth-session";

export function SolverActivityPanel() {
  const { session } = useAuthSession();

  if (!session) {
    return (
      <EmptyState
        title="Sign in to track your attempts"
        description="Browsing the marketplace is public. Sign in with Phantom before you start a solving session. Your personal session and payout history will appear here once that list is available."
        action={
          <Button asChild>
            <Link href="/bounties">Explore marketplace</Link>
          </Button>
        }
      />
    );
  }

  return (
    <EmptyState
      title="No personal history in this workspace yet"
      description="Participation and reward history for your wallet is not listed here yet. Open the marketplace to find an active challenge, consent, and submit answers. Confirmed payouts still settle through the custodial Devnet treasury."
      action={
        <div className="flex flex-wrap justify-center gap-3">
          <Button asChild>
            <Link href="/bounties">Explore marketplace</Link>
          </Button>
          <Button variant="outline" asChild>
            <Link href="/get-started">Switch modes</Link>
          </Button>
        </div>
      }
    />
  );
}
