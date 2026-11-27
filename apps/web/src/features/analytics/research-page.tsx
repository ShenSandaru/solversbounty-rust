import { Suspense } from "react";
import { PageContainer } from "@/components/page-container";
import { PageHeader } from "@/components/page-header";
import { ResearchDashboard } from "./research-dashboard";

export function ResearchPage() {
  return (
    <PageContainer>
      <PageHeader
        eyebrow="Creator mode"
        title="Owner research analytics"
        description="Aggregate activity for bounties you own. This is not a demonstration dataset and not a platform-wide admin console."
      />
      <Suspense
        fallback={
          <div className="mt-6 h-32 animate-pulse rounded-xl border border-border bg-card" role="status">
            <span className="sr-only">Loading analytics.</span>
          </div>
        }
      >
        <ResearchDashboard />
      </Suspense>
    </PageContainer>
  );
}
