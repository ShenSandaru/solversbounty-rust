"use client";

import { formatLamportsAsSol } from "@solversbounty/shared";
import Link from "next/link";
import { useEffect, useState } from "react";
import { PageContainer } from "@/components/page-container";
import { PageHeader } from "@/components/page-header";
import { Button } from "@/components/ui/button";
import { apiBaseUrl } from "@/features/auth/auth-client";
import { useAuthSession } from "@/features/auth/auth-session";
import { BountyClientError, readMyBounty, type BountyDraft } from "./bounty-api";
import { CreateBountyForm } from "./create-bounty-form";
import type { CreateBountyFormValues } from "./create-bounty-schema";

export function EditBountyPage({ id }: { id: string }) {
  const { session } = useAuthSession();
  const [draft, setDraft] = useState<BountyDraft | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session) {
      return;
    }
    let active = true;
    readMyBounty(apiBaseUrl(), id)
      .then((loaded) => {
        if (active) {
          setDraft(loaded);
        }
      })
      .catch((caught: unknown) => {
        if (active) {
          setError(caught instanceof BountyClientError ? caught.message : "The bounty could not be loaded.");
        }
      });
    return () => {
      active = false;
    };
  }, [id, session]);

  return (
    <PageContainer>
      <PageHeader
        eyebrow="Edit draft"
        title={draft?.title ?? "Your bounty"}
        description="Updates stay on the draft. Funding uses two separate SOL transfers to the custodial Devnet treasury."
        actions={
          <div className="flex flex-wrap items-center gap-2">
            {draft ? (
              <Button size="sm" asChild>
                <Link href={`/bounties/${draft.id}/fund`}>Continue to funding →</Link>
              </Button>
            ) : null}
            <Button size="sm" variant="outline" asChild>
              <Link href="/dashboard">My bounties</Link>
            </Button>
          </div>
        }
      />
      {!session ? <p className="mt-6 text-sm text-muted-foreground">Sign in to edit this draft.</p> : null}
      {error ? (
        <p className="mt-6 text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
      {session && !draft && !error ? <p className="mt-6 text-sm text-muted-foreground">Loading draft.</p> : null}
      {draft ? (
        <>
          <p className="mt-4 text-sm text-muted-foreground">
            Reward pool {formatLamportsAsSol(BigInt(draft.principalLamports))} SOL. Separate commission{" "}
            {formatLamportsAsSol(BigInt(draft.commissionLamports))} SOL. Status: {draft.status}.{" "}
            <Link href={`/bounties/${draft.id}/fund`} className="font-medium hover:underline">Fund this bounty</Link>.
          </p>
          <CreateBountyForm bountyId={draft.id} initial={formValues(draft)} />
        </>
      ) : null}
    </PageContainer>
  );
}

function localDateTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
function formValues(draft: BountyDraft): Partial<CreateBountyFormValues> {
  const reward = formatLamportsAsSol(BigInt(draft.rewardLamports));
  const deadline = localDateTime(draft.deadline);
  const questions =
    draft.questions && draft.questions.length > 0
      ? [...draft.questions]
          .sort((a, b) => a.orderIndex - b.orderIndex)
          .map((q) => ({
            id: q.id,
            orderIndex: q.orderIndex,
            questionType: q.questionType,
            prompt: q.prompt,
            ...(q.options ? { options: q.options } : {}),
            expectedAnswer: q.expectedAnswer,
            verificationMethod: q.verificationMethod,
          }))
      : [
          {
            orderIndex: 0,
            questionType: "short_answer" as const,
            prompt: draft.prompt,
            expectedAnswer: draft.expectedAnswer,
            verificationMethod: draft.verificationMethod ?? "exact",
          },
        ];
  return {
    title: draft.title,
    description: draft.description,
    instructions: draft.instructions,
    prompt: draft.prompt,
    category: draft.category,
    expectedAnswer: draft.expectedAnswer,
    verificationMethod: draft.verificationMethod,
    questions,
    rewardSol: reward,
    maxRewardedSolvers: draft.maxRewardedSolvers,
    attemptLimit: draft.attemptLimit,
    deadline,
    consentAcknowledged: true,
  };
}
