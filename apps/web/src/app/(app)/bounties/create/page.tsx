import type { Metadata } from "next";
import { Suspense } from "react";
import { CreateBountyPage } from "@/features/bounties/create-bounty-page";

export const metadata: Metadata = {
  title: "Create bounty",
};

export default function Page({
  searchParams,
}: {
  searchParams: Promise<{ from?: string | string[] }>;
}) {
  return (
    <Suspense fallback={<p className="px-6 py-10 text-sm text-muted-foreground">Loading create form.</p>}>
      <CreateRoute searchParams={searchParams} />
    </Suspense>
  );
}

async function CreateRoute({
  searchParams,
}: {
  searchParams: Promise<{ from?: string | string[] }>;
}) {
  const params = await searchParams;
  const from = Array.isArray(params.from) ? params.from[0] : params.from;
  return <CreateBountyPage fromOnboarding={from === "onboarding"} />;
}
