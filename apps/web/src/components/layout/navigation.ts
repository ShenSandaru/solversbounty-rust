import type { OnboardingJourney } from "@/features/marketing/journey-preference";

export type NavItem = {
  href: string;
  label: string;
  shortLabel: string;
};

const marketplaceItem: NavItem = {
  href: "/bounties",
  label: "Marketplace",
  shortLabel: "Market",
};

const solverActivityItem: NavItem = {
  href: "/dashboard",
  label: "My activity",
  shortLabel: "Activity",
};

const creatorBountiesItem: NavItem = {
  href: "/dashboard",
  label: "My bounties",
  shortLabel: "Bounties",
};

const createItem: NavItem = {
  href: "/bounties/create",
  label: "Create bounty",
  shortLabel: "Create",
};

const researchItem: NavItem = {
  href: "/research",
  label: "Research",
  shortLabel: "Research",
};

const getStartedItem: NavItem = {
  href: "/get-started",
  label: "Get started",
  shortLabel: "Start",
};

const adminItem: NavItem = {
  href: "/admin",
  label: "Admin",
  shortLabel: "Admin",
};

/** @deprecated Prefer navItemsForMode — kept for any leftover imports. */
export const navItems = [
  marketplaceItem,
  solverActivityItem,
  createItem,
  researchItem,
] as const;

export function navItemsForMode(mode: OnboardingJourney | null, options?: { isAdmin?: boolean }): NavItem[] {
  const withAdmin = (items: NavItem[]) => (options?.isAdmin ? [...items, adminItem] : items);
  if (mode === "solver") {
    return withAdmin([marketplaceItem, solverActivityItem, getStartedItem]);
  }
  if (mode === "creator") {
    return withAdmin([creatorBountiesItem, createItem, researchItem, marketplaceItem]);
  }
  return withAdmin([marketplaceItem, getStartedItem]);
}

export function isNavActive(pathname: string, href: string): boolean {
  if (href === "/bounties") {
    return pathname === "/bounties" || /^\/bounties\/(?!create$).+/.test(pathname);
  }
  if (href === "/bounties/create") {
    return pathname === "/bounties/create";
  }
  return pathname === href;
}

export function pageContext(
  pathname: string,
  mode: OnboardingJourney | null = null,
): { section: string; detail: string } {
  if (pathname === "/") {
    return { section: "Home", detail: "Product overview" };
  }
  if (pathname === "/get-started") {
    return { section: "Get started", detail: "Choose Solver or Creator mode" };
  }
  if (pathname === "/bounties") {
    return { section: "Marketplace", detail: "Open challenges" };
  }
  if (pathname === "/bounties/create") {
    return { section: "Create bounty", detail: "Save a draft" };
  }
  if (pathname.startsWith("/bounties/") && pathname.endsWith("/fund")) {
    return { section: "Fund bounty", detail: "Two Devnet transfers" };
  }
  if (pathname.startsWith("/bounties/") && pathname.endsWith("/edit")) {
    return { section: "Edit bounty", detail: "Owner draft" };
  }
  if (pathname.startsWith("/bounties/")) {
    return { section: "Challenge", detail: "Details" };
  }
  if (pathname === "/dashboard") {
    if (mode === "solver") {
      return { section: "My activity", detail: "Solver workspace" };
    }
    if (mode === "creator") {
      return { section: "My bounties", detail: "Creator workspace" };
    }
    return { section: "Workspace", detail: "Pick a mode to focus tools" };
  }
  if (pathname === "/research") {
    return { section: "Research", detail: "Owner aggregates for your bounties" };
  }
  if (pathname === "/admin") {
    return { section: "Admin", detail: "Allowlisted wallet sign-in" };
  }
  return { section: "SolversBounty", detail: "App" };
}
