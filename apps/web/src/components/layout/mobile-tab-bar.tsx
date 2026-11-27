"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { OnboardingJourney } from "@/features/marketing/journey-preference";
import { isNavActive, navItemsForMode } from "./navigation";

export function MobileTabBar({
  mode,
  isAdmin = false,
}: {
  mode: OnboardingJourney | null;
  isAdmin?: boolean;
}) {
  const pathname = usePathname();
  const items = navItemsForMode(mode, { isAdmin });

  return (
    <nav
      aria-label="Mobile primary"
      className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-card/95 pb-[env(safe-area-inset-bottom)] backdrop-blur-md lg:hidden"
    >
      <ul
        className="mx-auto grid max-w-lg gap-1 px-2 pt-2 pb-2"
        style={{ gridTemplateColumns: `repeat(${Math.min(items.length, 4)}, minmax(0, 1fr))` }}
      >
        {items.slice(0, 4).map((item) => {
          const active = isNavActive(pathname, item.href);
          return (
            <li key={`${item.href}-${item.label}`}>
              <Link
                href={item.href}
                aria-current={active ? "page" : undefined}
                className={`flex min-h-12 flex-col items-center justify-center gap-1 rounded-lg px-1 text-[11px] font-medium tracking-wide transition-colors ${
                  active ? "bg-brand-green/10 text-foreground" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <span
                  className={`size-1.5 rounded-full ${active ? "bg-brand-green" : "bg-border"}`}
                  aria-hidden="true"
                />
                {item.shortLabel}
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
