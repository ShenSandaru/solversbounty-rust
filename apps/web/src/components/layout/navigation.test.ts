import { describe, expect, it } from "vitest";
import { isNavActive, navItemsForMode, pageContext } from "./navigation";

describe("navItemsForMode", () => {
  it("shows marketplace and get started when mode is unset", () => {
    expect(navItemsForMode(null).map((item) => item.label)).toEqual(["Marketplace", "Get started"]);
  });

  it("focuses solver tools in solver mode", () => {
    expect(navItemsForMode("solver").map((item) => item.label)).toEqual([
      "Marketplace",
      "My activity",
      "Get started",
    ]);
  });

  it("focuses creator tools in creator mode", () => {
    expect(navItemsForMode("creator").map((item) => item.label)).toEqual([
      "My bounties",
      "Create bounty",
      "Research",
      "Marketplace",
    ]);
  });
});

describe("isNavActive", () => {
  it("treats challenge detail routes as marketplace", () => {
    expect(isNavActive("/bounties/abc", "/bounties")).toBe(true);
    expect(isNavActive("/bounties/create", "/bounties")).toBe(false);
    expect(isNavActive("/bounties/create", "/bounties/create")).toBe(true);
  });
});

describe("pageContext", () => {
  it("labels dashboard by mode", () => {
    expect(pageContext("/dashboard", "solver").section).toBe("My activity");
    expect(pageContext("/dashboard", "creator").section).toBe("My bounties");
    expect(pageContext("/research", null).detail).toContain("Owner aggregates");
  });
});
