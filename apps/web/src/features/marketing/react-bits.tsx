"use client";

import { useEffect, useRef, type ReactNode } from "react";

/**
 * React Bits-style staggered blur + slide-up word reveal.
 */
export function BlurText({
  text,
  accentWord,
  className = "",
}: {
  text: string;
  accentWord?: string;
  className?: string;
}) {
  const words = text.split(" ");
  return (
    <span className={className}>
      {words.map((word, index) => {
        const isAccent = accentWord ? accentWord.split(" ").includes(word) : false;
        return (
          <span
            key={`${word}-${index}`}
            className={`rb-blur-word inline-block ${isAccent ? "gradient-text" : ""}`}
            style={{ animationDelay: `${index * 70}ms` }}
          >
            {word}
            {index < words.length - 1 ? "\u00A0" : ""}
          </span>
        );
      })}
    </span>
  );
}

/**
 * React Bits-style cursor-tracking radial spotlight card (60fps via CSS vars, no React re-renders).
 */
export function SpotlightCard({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  const cardRef = useRef<HTMLDivElement>(null);

  return (
    <div
      ref={cardRef}
      onMouseMove={(event) => {
        const node = cardRef.current;
        if (!node) {
          return;
        }
        const rect = node.getBoundingClientRect();
        node.style.setProperty("--spot-x", `${event.clientX - rect.left}px`);
        node.style.setProperty("--spot-y", `${event.clientY - rect.top}px`);
      }}
      className={`rb-spotlight relative overflow-hidden ${className}`}
    >
      {children}
    </div>
  );
}

/**
 * React Bits-style shiny shimmer badge.
 */
export function ShinyBadge({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <span
      className={`rb-shiny-badge relative inline-flex items-center overflow-hidden rounded-full border border-brand-green/35 bg-brand-green/10 px-3.5 py-1 text-xs font-medium tracking-[0.16em] text-brand-green uppercase ${className}`}
    >
      {children}
    </span>
  );
}

/**
 * Animated shimmer placeholder for preview telemetry / skeleton cards.
 */
export function ShimmerPlaceholder({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint: string;
}) {
  return (
    <div className="surface relative overflow-hidden p-4">
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-medium text-muted-foreground">{label}</span>
        <span className="inline-flex size-2 rounded-full bg-brand-green/80 animate-pulse" aria-hidden="true" />
      </div>
      <p className="mt-2 font-mono text-lg font-semibold text-foreground">{value}</p>
      <div className="rb-shimmer-bar mt-2.5 h-1.5 w-full rounded-full" aria-hidden="true" />
      <p className="mt-2 text-xs text-muted-foreground">{hint}</p>
    </div>
  );
}

/**
 * Programmatic Canvas particle/node network behind the hero section.
 * Respects prefers-reduced-motion.
 */
export function CognitiveCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      return;
    }
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    let rafId = 0;
    let width = 0;
    let height = 0;
    const pointer = { x: -9999, y: -9999 };

    type Node = { x: number; y: number; vx: number; vy: number; r: number; accent: boolean };
    const nodes: Node[] = [];

    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      width = rect.width;
      height = rect.height;
      canvas.width = Math.max(1, Math.floor(width * dpr));
      canvas.height = Math.max(1, Math.floor(height * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      if (nodes.length === 0) {
        const count = Math.min(34, Math.max(18, Math.floor(width / 42)));
        for (let i = 0; i < count; i += 1) {
          nodes.push({
            x: Math.random() * width,
            y: Math.random() * height,
            vx: (Math.random() - 0.5) * 0.35,
            vy: (Math.random() - 0.5) * 0.35,
            r: 1.4 + Math.random() * 1.4,
            accent: i % 3 === 0,
          });
        }
      }
    };

    resize();
    window.addEventListener("resize", resize);

    const onPointerMove = (event: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      pointer.x = event.clientX - rect.left;
      pointer.y = event.clientY - rect.top;
    };
    const onPointerLeave = () => {
      pointer.x = -9999;
      pointer.y = -9999;
    };
    window.addEventListener("pointermove", onPointerMove, { passive: true });
    window.addEventListener("pointerleave", onPointerLeave);

    const draw = () => {
      ctx.clearRect(0, 0, width, height);
      const isLight = document.documentElement.dataset.theme === "light";
      const lineAlpha = isLight ? 0.14 : 0.16;

      for (const node of nodes) {
        node.x += node.vx;
        node.y += node.vy;
        if (node.x < 0 || node.x > width) node.vx *= -1;
        if (node.y < 0 || node.y > height) node.vy *= -1;

        const dx = pointer.x - node.x;
        const dy = pointer.y - node.y;
        const dist = Math.hypot(dx, dy);
        if (dist > 0 && dist < 130) {
          node.x -= (dx / dist) * 0.35;
          node.y -= (dy / dist) * 0.35;
        }
      }

      for (let i = 0; i < nodes.length; i += 1) {
        const a = nodes[i];
        if (!a) continue;
        for (let j = i + 1; j < nodes.length; j += 1) {
          const b = nodes[j];
          if (!b) continue;
          const d = Math.hypot(a.x - b.x, a.y - b.y);
          if (d < 135) {
            const alpha = (1 - d / 135) * lineAlpha;
            ctx.strokeStyle = a.accent ? `rgba(20, 241, 149, ${alpha})` : `rgba(153, 69, 255, ${alpha})`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(a.x, a.y);
            ctx.lineTo(b.x, b.y);
            ctx.stroke();
          }
        }
      }

      for (const node of nodes) {
        ctx.fillStyle = node.accent
          ? isLight
            ? "rgba(5, 150, 105, 0.45)"
            : "rgba(20, 241, 149, 0.55)"
          : isLight
            ? "rgba(124, 58, 237, 0.35)"
            : "rgba(153, 69, 255, 0.55)";
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.r, 0, Math.PI * 2);
        ctx.fill();
      }

      rafId = window.requestAnimationFrame(draw);
    };

    rafId = window.requestAnimationFrame(draw);

    return () => {
      window.cancelAnimationFrame(rafId);
      window.removeEventListener("resize", resize);
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerleave", onPointerLeave);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 h-full w-full opacity-80"
    />
  );
}
