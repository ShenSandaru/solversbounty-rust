import { BountyClientError } from "@/features/bounties/bounty-api";

export type BountyAnalytics = {
  participantCount: number;
  correctCount: number;
  incorrectCount: number;
  attemptCount: number;
  averageCompletionTimeMs: number | null;
  accuracy: string | null;
};

type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

export async function fetchBountyAnalytics(
  apiUrl: string,
  bountyId: string,
  fetchImpl: FetchLike = fetch,
): Promise<BountyAnalytics> {
  let response: Response;
  try {
    response = await fetchImpl(`${apiUrl}/bounties/${bountyId}/analytics`, {
      method: "GET",
      credentials: "include",
      cache: "no-store",
    });
  } catch {
    throw new BountyClientError("The API is unavailable.", "API_UNAVAILABLE");
  }
  if (!response.ok) {
    if (response.status === 401) {
      throw new BountyClientError("Sign in before reading owner analytics.", "UNAUTHORIZED");
    }
    if (response.status === 403) {
      throw new BountyClientError("Only the bounty owner can read these analytics.", "FORBIDDEN");
    }
    if (response.status === 404) {
      throw new BountyClientError("That bounty was not found.", "NOT_FOUND");
    }
    throw new BountyClientError("Owner analytics could not be loaded.", "API_FAILURE");
  }
  return parseBountyAnalytics(await response.json());
}

export function parseBountyAnalytics(value: unknown): BountyAnalytics {
  if (!isRecord(value)) {
    throw new BountyClientError("The API returned incomplete analytics.");
  }
  if (
    typeof value.participantCount !== "number" ||
    typeof value.correctCount !== "number" ||
    typeof value.incorrectCount !== "number" ||
    typeof value.attemptCount !== "number"
  ) {
    throw new BountyClientError("The API returned incomplete analytics.");
  }
  if ("walletAddress" in value || "submittedAnswer" in value || "expectedAnswer" in value) {
    throw new BountyClientError("The API returned incomplete analytics.");
  }
  const averageCompletionTimeMs =
    value.averageCompletionTimeMs === null || value.averageCompletionTimeMs === undefined
      ? null
      : typeof value.averageCompletionTimeMs === "number"
        ? value.averageCompletionTimeMs
        : null;
  if (value.averageCompletionTimeMs !== null && value.averageCompletionTimeMs !== undefined && averageCompletionTimeMs === null) {
    throw new BountyClientError("The API returned incomplete analytics.");
  }
  const accuracy =
    value.accuracy === null || value.accuracy === undefined
      ? null
      : typeof value.accuracy === "string"
        ? value.accuracy
        : null;
  if (value.accuracy !== null && value.accuracy !== undefined && accuracy === null) {
    throw new BountyClientError("The API returned incomplete analytics.");
  }
  return {
    participantCount: value.participantCount,
    correctCount: value.correctCount,
    incorrectCount: value.incorrectCount,
    attemptCount: value.attemptCount,
    averageCompletionTimeMs,
    accuracy,
  };
}

export function accuracyDisplay(accuracy: string | null): string {
  if (accuracy === null) {
    return "—";
  }
  const ratio = Number(accuracy);
  if (!Number.isFinite(ratio)) {
    return "—";
  }
  return `${(ratio * 100).toFixed(1)}%`;
}

export function formatAverageCompletionMs(ms: number | null): string {
  if (ms === null) {
    return "—";
  }
  if (ms < 1000) {
    return `${ms} ms`;
  }
  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)} s`;
  }
  const minutes = Math.floor(seconds / 60);
  const rem = Math.round(seconds % 60);
  return `${minutes}m ${rem}s`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
