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

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [openOnPath, setOpenOnPath] = useState<string | null>(null);
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
    <div className="min-h-full lg:grid lg:grid-cols-[18rem_minmax(0,1fr)]">
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-50 focus:rounded-lg focus:bg-card focus:px-3 focus:py-2 focus:text-sm focus:ring-2 focus:ring-brand-green"
      >
        Skip to content
      </a>
      <aside className="sticky top-0 hidden h-screen flex-col border-r border-border bg-card/80 px-4 py-6 backdrop-blur lg:flex">
        <Brand className="mb-8" />
        <ModeSwitch mode={journey} className="mb-6" />
        <Navigation pathname={pathname} items={items} onNavigate={() => setOpenOnPath(null)} />
        <SidebarGuideCard mode={journey} isSignedIn={Boolean(session)} onOpenGuide={() => setGuideOpen(true)} />
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
  onNavigate,
}: {
  pathname: string;
  items: ReturnType<typeof navItemsForMode>;
  onNavigate?: () => void;
}) {
  return (
    <nav aria-label="Primary" className="flex flex-col gap-1">
      {items.map((item) => {
        const active = isNavActive(pathname, item.href);
        return (
          <Link
            key={`${item.href}-${item.label}`}
            href={item.href}
            aria-current={active ? "page" : undefined}
            onClick={onNavigate}
            className={`rounded-lg border-l-2 px-3 py-2 text-sm whitespace-nowrap transition-colors ${
              active
                ? "border-brand-green bg-brand-green/10 font-medium text-foreground"
                : "border-transparent text-muted-foreground hover:bg-secondary hover:text-foreground"
            }`}
          >
            {item.label}
          </Link>
        );
      })}
    </nav>
  );
}
