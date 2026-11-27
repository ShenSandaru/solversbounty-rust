"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useId, useState, useSyncExternalStore } from "react";
import { useAuthSession } from "@/features/auth/auth-session";
import { WalletAuth } from "@/features/auth/wallet-auth";
import {
  readOnboardingJourney,
  subscribeOnboardingJourney,
} from "@/features/marketing/journey-preference";
import { Brand } from "./brand";
import { MobileTabBar } from "./mobile-tab-bar";
import { ModeSwitch } from "./mode-switch";
import { isNavActive, navItemsForMode, pageContext } from "./navigation";
import { SiteFooter } from "./site-footer";
import { SidebarGuideCard, SystemGuideModal } from "./system-guide";
import { ThemeToggle } from "@/components/theme-toggle";

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [openOnPath, setOpenOnPath] = useState<string | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [guideOpen, setGuideOpen] = useState(false);
  const menuOpen = openOnPath === pathname;
  const menuId = useId();
  const { session } = useAuthSession();
  const journey = useSyncExternalStore(subscribeOnboardingJourney, readOnboardingJourney, () => null);
  const context = pageContext(pathname, journey);
  const isAdmin = session?.role === "admin";
  const items = navItemsForMode(journey, { isAdmin });

  useEffect(() => {
    if (!menuOpen) {
      return;
    }
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpenOnPath(null);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [menuOpen]);

  return (
    <div
      className={`min-h-full lg:grid ${
        sidebarCollapsed ? "lg:grid-cols-[4.25rem_minmax(0,1fr)]" : "lg:grid-cols-[18rem_minmax(0,1fr)]"
      }`}
    >
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-50 focus:rounded-lg focus:bg-card focus:px-3 focus:py-2 focus:text-sm focus:ring-2 focus:ring-brand-green"
      >
        Skip to content
      </a>
      <aside
        className={`sticky top-0 hidden h-screen flex-col border-r border-border bg-card/80 py-6 backdrop-blur lg:flex ${
          sidebarCollapsed ? "items-center px-2" : "px-4"
        }`}
      >
        {sidebarCollapsed ? (
          <button
            type="button"
            onClick={() => setSidebarCollapsed(false)}
            aria-label="Expand sidebar"
            title="Expand sidebar"
            className="mb-6 inline-flex size-9 items-center justify-center rounded-lg border border-border bg-background/60 text-muted-foreground transition-colors hover:border-brand-green/60 hover:text-foreground"
          >
            <SidebarIcon />
          </button>
        ) : (
          <div className="mb-8 flex items-center justify-between gap-2">
            <Brand />
            <button
              type="button"
              onClick={() => setSidebarCollapsed(true)}
              aria-label="Collapse sidebar"
              title="Collapse sidebar"
              className="inline-flex size-8 shrink-0 items-center justify-center rounded-lg border border-transparent text-muted-foreground transition-colors hover:border-border hover:bg-secondary hover:text-foreground"
            >
              <SidebarIcon />
            </button>
          </div>
        )}
        {!sidebarCollapsed ? <ModeSwitch mode={journey} className="mb-6" /> : null}
        <Navigation
          pathname={pathname}
          items={items}
          collapsed={sidebarCollapsed}
          onNavigate={() => setOpenOnPath(null)}
        />
        {sidebarCollapsed ? (
          <button
            type="button"
            onClick={() => setGuideOpen(true)}
            aria-label="How it works"
            title="How it works"
            className="mt-auto inline-flex size-9 items-center justify-center rounded-lg border border-border bg-background/60 text-muted-foreground transition-colors hover:border-brand-green/60 hover:text-foreground"
          >
            <GuideIcon />
          </button>
        ) : (
          <SidebarGuideCard mode={journey} isSignedIn={Boolean(session)} onOpenGuide={() => setGuideOpen(true)} />
        )}
      </aside>
      <div className="flex min-h-full min-w-0 flex-col">
        <header className="sticky top-0 z-20 border-b border-border bg-background/85 backdrop-blur-md">
          <div className="flex flex-wrap items-center justify-between gap-3 px-4 py-3 sm:px-6 lg:px-8">
            <div className="flex min-w-0 items-center gap-3">
              <button
                type="button"
                className="inline-flex h-10 items-center rounded-lg border border-border bg-card px-3 text-sm font-medium transition-colors hover:border-brand-green/60 lg:hidden"
                aria-expanded={menuOpen}
                aria-controls={menuId}
                onClick={() => setOpenOnPath(menuOpen ? null : pathname)}
              >
                Menu
              </button>
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">{context.section}</p>
                <p className="hidden truncate text-xs text-muted-foreground sm:block">{context.detail}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <ThemeToggle />
              <button
                type="button"
                onClick={() => setGuideOpen(true)}
                className="inline-flex h-9 items-center rounded-lg border border-border bg-card px-3 text-xs font-medium text-muted-foreground transition-colors hover:border-brand-green/60 hover:text-foreground"
              >
                Guide
              </button>
              <WalletAuth />
            </div>
          </div>
        </header>
        {menuOpen ? (
          <div className="fixed inset-0 z-30 lg:hidden" id={menuId}>
            <button
              type="button"
              className="absolute inset-0 bg-black/60"
              aria-label="Close menu"
              onClick={() => setOpenOnPath(null)}
            />
            <div
              className="relative flex h-full w-72 flex-col bg-card px-4 py-6 shadow-xl"
              role="dialog"
              aria-modal="true"
              aria-label="Navigation"
            >
              <Brand className="mb-8" />
              <ModeSwitch mode={journey} className="mb-6" />
              <Navigation pathname={pathname} items={items} onNavigate={() => setOpenOnPath(null)} />
            </div>
          </div>
        ) : null}
        <div className="flex flex-1 flex-col pb-[calc(4.5rem+env(safe-area-inset-bottom))] lg:pb-0">
          <div id="main-content" className="flex-1" tabIndex={-1}>
            {children}
          </div>
          <SiteFooter />
        </div>
      </div>
      <MobileTabBar mode={journey} isAdmin={isAdmin} />
      <SystemGuideModal open={guideOpen} mode={journey} onClose={() => setGuideOpen(false)} />
    </div>
  );
}

function Navigation({
  pathname,
  items,
  collapsed = false,
  onNavigate,
}: {
  pathname: string;
  items: ReturnType<typeof navItemsForMode>;
  collapsed?: boolean;
  onNavigate?: () => void;
}) {
  return (
    <nav aria-label="Primary" className={`flex flex-col gap-1 ${collapsed ? "items-center" : ""}`}>
      {items.map((item) => {
        const active = isNavActive(pathname, item.href);
        return (
          <Link
            key={`${item.href}-${item.label}`}
            href={item.href}
            aria-current={active ? "page" : undefined}
            aria-label={collapsed ? item.label : undefined}
            title={collapsed ? item.label : undefined}
            onClick={onNavigate}
            className={`flex items-center rounded-lg transition-colors ${
              collapsed
                ? `size-10 justify-center border ${
                    active
                      ? "border-brand-green/60 bg-brand-green/15 text-foreground"
                      : "border-transparent text-muted-foreground hover:bg-secondary hover:text-foreground"
                  }`
                : `gap-2.5 border-l-2 px-3 py-2 text-sm whitespace-nowrap ${
                    active
                      ? "border-brand-green bg-brand-green/10 font-medium text-foreground"
                      : "border-transparent text-muted-foreground hover:bg-secondary hover:text-foreground"
                  }`
            }`}
          >
            <NavItemIcon href={item.href} />
            {collapsed ? null : <span>{item.label}</span>}
          </Link>
        );
      })}
    </nav>
  );
}

function NavItemIcon({ href }: { href: string }) {
  if (href === "/bounties") {
    return (
      <svg viewBox="0 0 20 20" fill="none" className="size-4 shrink-0" aria-hidden="true">
        <rect x="3" y="3" width="5.5" height="5.5" rx="1.2" stroke="currentColor" strokeWidth="1.5" />
        <rect x="11.5" y="3" width="5.5" height="5.5" rx="1.2" stroke="currentColor" strokeWidth="1.5" />
        <rect x="3" y="11.5" width="5.5" height="5.5" rx="1.2" stroke="currentColor" strokeWidth="1.5" />
        <rect x="11.5" y="11.5" width="5.5" height="5.5" rx="1.2" stroke="currentColor" strokeWidth="1.5" />
      </svg>
    );
  }
  if (href === "/bounties/create") {
    return (
      <svg viewBox="0 0 20 20" fill="none" className="size-4 shrink-0" aria-hidden="true">
        <circle cx="10" cy="10" r="7" stroke="currentColor" strokeWidth="1.5" />
        <path d="M10 6.5v7M6.5 10h7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    );
  }
  if (href === "/dashboard") {
    return (
      <svg viewBox="0 0 20 20" fill="none" className="size-4 shrink-0" aria-hidden="true">
        <path d="M3.5 5.5h13M3.5 10h13M3.5 14.5h8.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    );
  }
  if (href === "/research") {
    return (
      <svg viewBox="0 0 20 20" fill="none" className="size-4 shrink-0" aria-hidden="true">
        <path d="M4 15.5V10m4 5.5V6.5m4 9V9m4 6.5V4.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    );
  }
  if (href === "/admin") {
    return (
      <svg viewBox="0 0 20 20" fill="none" className="size-4 shrink-0" aria-hidden="true">
        <path d="M10 3l6 2.5v4.5c0 3.5-2.5 5.8-6 7-3.5-1.2-6-3.5-6-7V5.5L10 3z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
      </svg>
    );
  }
  return (
    <svg viewBox="0 0 20 20" fill="none" className="size-4 shrink-0" aria-hidden="true">
      <path d="M10 3.5l2 4.2 4.5.7-3.2 3.2.8 4.5L10 14l-4.1 2.1.8-4.5-3.2-3.2 4.5-.7L10 3.5z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
    </svg>
  );
}

function GuideIcon() {
  return (
    <svg viewBox="0 0 20 20" fill="none" className="size-4" aria-hidden="true">
      <circle cx="10" cy="10" r="7" stroke="currentColor" strokeWidth="1.5" />
      <path d="M8.3 7.8a1.8 1.8 0 0 1 3.4.8c0 1.2-1.7 1.5-1.7 2.6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      <circle cx="10" cy="14" r="0.8" fill="currentColor" />
    </svg>
  );
}

function SidebarIcon() {
  return (
    <svg viewBox="0 0 20 20" fill="none" className="size-4" aria-hidden="true">
      <rect x="2.5" y="3.5" width="15" height="13" rx="2" stroke="currentColor" strokeWidth="1.5" />
      <path d="M7.5 3.5v13" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}
