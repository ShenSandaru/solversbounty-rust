"use client";

import { useEffect, useRef, useState } from "react";

const comicStates = [
  { text: "Wanna earn some SOL? 👀", accent: "text-brand-green" },
  { text: "Teach me how you think! 🧠", accent: "text-brand-purple" },
  { text: "Beep boop... need human brain!", accent: "text-foreground" },
  { text: "Solve it right → get paid ⚡", accent: "text-brand-green" },
  { text: "I'm still learning... help me out!", accent: "text-brand-purple" },
] as const;

export function HeroArt() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [eyeOffset, setEyeOffset] = useState({ x: 0, y: 0 });
  const [bubbleIndex, setBubbleIndex] = useState(0);

  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      return;
    }
    const timer = window.setInterval(() => {
      setBubbleIndex((current) => (current + 1) % comicStates.length);
    }, 2400);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      return;
    }
    function onPointerMove(event: PointerEvent) {
      const node = containerRef.current;
      if (!node) {
        return;
      }
      const rect = node.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const dx = Math.max(-1, Math.min(1, (event.clientX - cx) / 260));
      const dy = Math.max(-1, Math.min(1, (event.clientY - cy) / 260));
      setEyeOffset({ x: dx * 5.5, y: dy * 3.5 });
    }
    window.addEventListener("pointermove", onPointerMove, { passive: true });
    return () => window.removeEventListener("pointermove", onPointerMove);
  }, []);

  const currentBubble = comicStates[bubbleIndex] ?? comicStates[0];

  return (
    <div
      ref={containerRef}
      className="relative flex aspect-square w-full max-w-[22rem] items-center justify-center select-none"
      aria-hidden="true"
    >
      {/* Ambient glow */}
      <div className="pointer-events-none absolute -inset-6 rounded-full bg-[radial-gradient(circle_at_35%_30%,rgb(153_69_255/0.26),transparent_55%),radial-gradient(circle_at_70%_75%,rgb(20_241_149/0.2),transparent_55%)] blur-2xl" />

      {/* Frameless robot stage blending with background */}
      <div className="relative flex size-full items-center justify-center p-2">
        {/* Comic status speech bubble */}
        <div className="absolute top-3.5 right-3.5 z-10 max-w-[13.5rem] rounded-2xl rounded-bl-xs border border-border bg-card/95 px-3.5 py-2 shadow-lg backdrop-blur">
          <span className={`block text-xs font-semibold leading-snug ${currentBubble.accent}`}>
            {currentBubble.text}
          </span>
        </div>

        {/* Floating puzzle badge */}
        <div
          className="absolute top-6 left-5 z-10 rounded-lg border border-brand-purple/40 bg-brand-purple/15 px-2.5 py-1 font-mono text-[11px] font-semibold text-foreground animate-bounce"
          style={{ animationDuration: "3.4s" }}
        >
          ? Logic
        </div>

        {/* Floating SOL coin badge */}
        <div
          className="absolute bottom-5 right-5 z-10 rounded-lg border border-brand-green/40 bg-brand-green/15 px-2.5 py-1 font-mono text-[11px] font-semibold text-brand-green animate-bounce"
          style={{ animationDuration: "2.9s", animationDelay: "0.4s" }}
        >
          ◎ SOL
        </div>

        {/* Animated Comic Solver Robot SVG */}
        <svg
          viewBox="0 0 260 260"
          fill="none"
          className="relative z-10 h-full w-full drop-shadow-[0_14px_28px_rgb(0_0_0/0.35)]"
        >
          <defs>
            <linearGradient id="bot-grad" x1="40" y1="30" x2="220" y2="230" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#9945FF" />
              <stop offset="100%" stopColor="#14F195" />
            </linearGradient>
            <linearGradient id="bot-body" x1="130" y1="70" x2="130" y2="220" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#2a2a3a" />
              <stop offset="100%" stopColor="#171722" />
            </linearGradient>
          </defs>

          {/* Subtle orbit ring */}
          <circle
            cx="130"
            cy="134"
            r="96"
            stroke="url(#bot-grad)"
            strokeOpacity="0.25"
            strokeWidth="1.5"
            strokeDasharray="6 8"
          />

          {/* Ground shadow */}
          <ellipse cx="130" cy="228" rx="50" ry="9" fill="#000000" fillOpacity="0.28" />

          {/* Floating robot group */}
          <g className="animate-[hero-shift_3.2s_ease-in-out_infinite_alternate]">
            {/* Antenna */}
            <path d="M130 56V36" stroke="url(#bot-grad)" strokeWidth="4" strokeLinecap="round" />
            <circle cx="130" cy="30" r="8" fill="url(#bot-grad)" />
            <circle cx="130" cy="30" r="14" stroke="#14F195" strokeOpacity="0.35" strokeWidth="2" />

            {/* Left ear */}
            <rect x="58" y="84" width="12" height="28" rx="6" fill="url(#bot-grad)" />
            {/* Right ear */}
            <rect x="190" y="84" width="12" height="28" rx="6" fill="url(#bot-grad)" />

            {/* Head */}
            <rect
              x="70"
              y="56"
              width="120"
              height="82"
              rx="26"
              fill="url(#bot-body)"
              stroke="url(#bot-grad)"
              strokeWidth="2.5"
            />

            {/* Face visor */}
            <rect x="84" y="72" width="92" height="50" rx="16" fill="#0c0c12" stroke="#2d2d3d" strokeWidth="1.5" />

            {/* Cursor-tracking glowing eyes */}
            <g transform={`translate(${eyeOffset.x}, ${eyeOffset.y})`}>
              <rect x="100" y="86" width="18" height="20" rx="9" fill="#14F195" />
              <circle cx="106" cy="92" r="3" fill="#ffffff" />
              <rect x="142" y="86" width="18" height="20" rx="9" fill="#14F195" />
              <circle cx="148" cy="92" r="3" fill="#ffffff" />
            </g>

            {/* Friendly comic smile */}
            <path
              d="M118 112c4 4 20 4 24 0"
              stroke="#9945FF"
              strokeWidth="2.5"
              strokeLinecap="round"
            />

            {/* Neck */}
            <rect x="116" y="138" width="28" height="10" rx="4" fill="#232332" />

            {/* Torso */}
            <rect
              x="86"
              y="146"
              width="88"
              height="62"
              rx="22"
              fill="url(#bot-body)"
              stroke="url(#bot-grad)"
              strokeWidth="2.2"
            />

            {/* Chest core emblem */}
            <rect x="108" y="162" width="44" height="26" rx="10" fill="#0c0c12" stroke="#2d2d3d" strokeWidth="1.5" />
            <path
              d="M118 175h6l3-6 5 12 3-6h7"
              stroke="url(#bot-grad)"
              strokeWidth="2.2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />

            {/* Left arm */}
            <path
              d="M84 162c-16 4-24 14-24 28"
              stroke="url(#bot-grad)"
              strokeWidth="8"
              strokeLinecap="round"
            />
            {/* Right waving arm */}
            <path
              d="M176 162c16-4 24-14 26-26"
              stroke="url(#bot-grad)"
              strokeWidth="8"
              strokeLinecap="round"
            />
            <circle cx="203" cy="133" r="6" fill="#14F195" />
          </g>
        </svg>
      </div>
    </div>
  );
}
