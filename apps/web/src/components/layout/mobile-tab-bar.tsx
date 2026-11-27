"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { isNavActive, navItems } from "./navigation";

const shortLabels: Record<string, string> = {
  "/bounties": "Market",
  "/dashboard": "Dash",
  "/bounties/create": "Create",
  "/research": "Research",
};

export function MobileTabBar() {
  const pathname = usePathname();

  return (
    <nav
      aria-label="Mobile primary"
      className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-card/95 pb-[env(safe-area-inset-bottom)] backdrop-blur-md lg:hidden"
    >
      <ul className="mx-auto grid max-w-lg grid-cols-4 gap-1 px-2 pt-2 pb-2">
        {navItems.map((item) => {
          const active = isNavActive(pathname, item.href);
          return (
            <li key={item.href}>
              <Link
                href={item.href}
                aria-current={active ? "page" : undefined}
                className={`flex min-h-12 flex-col items-center justify-center gap-1 rounded-lg px-1 text-[10px] font-medium tracking-wide transition-colors ${
                  active ? "bg-brand-purple/15 text-foreground" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <span
                  className={`size-1.5 rounded-full ${active ? "bg-brand-green" : "bg-border"}`}
                  aria-hidden="true"
                />
                {shortLabels[item.href] ?? item.label}
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
