import { describe, expect, it } from "vitest";
import { accuracyDisplay, formatAverageCompletionMs, parseBountyAnalytics } from "./analytics-api";

describe("parseBountyAnalytics", () => {
  it("accepts ADR-009 aggregate fields", () => {
    expect(
      parseBountyAnalytics({
        participantCount: 2,
        correctCount: 1,
        incorrectCount: 3,
        attemptCount: 4,
        averageCompletionTimeMs: 1500,
        accuracy: "0.2500",
      }),
    ).toEqual({
      participantCount: 2,
      correctCount: 1,
      incorrectCount: 3,
      attemptCount: 4,
      averageCompletionTimeMs: 1500,
      accuracy: "0.2500",
    });
  });

  it("rejects payloads that could leak private research rows", () => {
    expect(() =>
      parseBountyAnalytics({
        participantCount: 1,
        correctCount: 0,
        incorrectCount: 0,
        attemptCount: 0,
        averageCompletionTimeMs: null,
        accuracy: null,
        walletAddress: "abc",
      }),
    ).toThrow(/incomplete analytics/i);
  });
});

describe("analytics display helpers", () => {
  it("formats accuracy ratio as a percent", () => {
    expect(accuracyDisplay("0.5000")).toBe("50.0%");
    expect(accuracyDisplay(null)).toBe("—");
  });

  it("formats completion time", () => {
    expect(formatAverageCompletionMs(null)).toBe("—");
    expect(formatAverageCompletionMs(250)).toBe("250 ms");
    expect(formatAverageCompletionMs(1500)).toBe("1.5 s");
  });
});
