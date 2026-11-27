import { APP_NAME } from "@solversbounty/shared";
import { cn } from "cn";
import Link from "next/link";

export function Brand({
  className,
  subtitle = "Cognitive bounty network",
}: {
  className?: string;
  subtitle?: string;
}) {
  return (
    <Link href="/" className={cn("flex items-center gap-3 rounded-lg", className)}>
      <span className="grid size-9 shrink-0 place-items-center rounded-xl bg-brand-gradient p-[1.5px] shadow-[0_0_20px_rgb(153_69_255/0.25)]">
        <span className="grid size-full place-items-center rounded-[10.5px] bg-[#111116]">
          <svg viewBox="0 0 28 28" fill="none" className="size-6" aria-hidden="true">
            <defs>
              <linearGradient id="sb-logo-grad" x1="3" y1="3" x2="25" y2="25" gradientUnits="userSpaceOnUse">
                <stop offset="0%" stopColor="#9945FF" />
                <stop offset="100%" stopColor="#14F195" />
              </linearGradient>
            </defs>
            <path
              d="M14 2.5 23.5 8v12L14 25.5 4.5 20V8L14 2.5Z"
              stroke="url(#sb-logo-grad)"
              strokeWidth="1.75"
              strokeLinejoin="round"
            />
            <path
              d="M17.8 9.8c-.9-.8-2.2-1.3-3.8-1.3-2.1 0-3.5 1-3.5 2.5 0 1.4 1.1 2.1 3.3 2.5l1 .2c2.2.4 3.4 1.3 3.4 2.9 0 1.8-1.6 3-4.1 3-1.8 0-3.3-.6-4.3-1.6"
              stroke="url(#sb-logo-grad)"
              strokeWidth="2.1"
              strokeLinecap="round"
            />
            <circle cx="14" cy="5.8" r="1.2" fill="#14F195" />
            <circle cx="14" cy="22.2" r="1.2" fill="#9945FF" />
          </svg>
        </span>
      </span>
      <span className="min-w-0">
        <span className="block text-sm font-semibold">{APP_NAME}</span>
        {subtitle ? <span className="hidden text-xs text-brand-green sm:block">{subtitle}</span> : null}
      </span>
    </Link>
  );
}
