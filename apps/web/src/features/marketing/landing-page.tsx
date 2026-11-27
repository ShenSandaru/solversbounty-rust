import { APP_NAME } from "@solversbounty/shared";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { HealthStatus } from "@/features/home/health-status";
import { HeroArt } from "./hero-art";
import { BlurText, CognitiveCanvas, ShinyBadge, SpotlightCard } from "./react-bits";
import { Reveal } from "./reveal";

const solverSteps = [
  {
    step: "1",
    title: "Browse open challenges",
    body: "Explore active cognitive challenges in the marketplace. Read prompts publicly before connecting a wallet.",
  },
  {
    step: "2",
    title: "Consent & submit",
    body: "Review and accept the research consent, then submit your answer. All verification runs securely on the server.",
  },
  {
    step: "3",
    title: "Earn instant SOL rewards",
    body: "Every eligible correct solver receives the configured SOL payout automatically while the reward pool remains open.",
  },
] as const;

const creatorSteps = [
  {
    step: "1",
    title: "Draft a challenge",
    body: "Define your public prompt, encrypted expected answer, attempt limit, and reward terms. Saving a draft is free.",
  },
  {
    step: "2",
    title: "Fund the reward pool",
    body: "Approve the reward pool and separate 5% platform fee transfers in Phantom to the custodial Devnet treasury.",
  },
  {
    step: "3",
    title: "Activate & review insights",
    body: "Publish to the marketplace and track consented aggregate metrics—participants, accuracy, attempts, and completion time.",
  },
] as const;

const trustPoints = [
  "Settlements currently run on Solana Devnet via a backend-managed custodial Devnet treasury (not trustless escrow).",
  "Research recording requires explicit opt-in consent. Connecting or signing in with a wallet never implies research consent.",
  "Analytics are strictly aggregated. Creators and administrators never see individual solver records or wallet addresses in analytics.",
  "Solvers pay zero participation fees. Wallet addresses are pseudonymous and on-chain transfers are public.",
] as const;

export function LandingPage() {
  return (
    <main>
      <section className="relative overflow-hidden">
        <CognitiveCanvas />
        <div className="relative z-10 mx-auto grid w-full max-w-6xl items-center gap-12 px-4 py-16 sm:px-6 lg:grid-cols-[minmax(0,1.15fr)_minmax(16rem,0.85fr)] lg:px-8 lg:py-24">
          <div>
            <ShinyBadge className="gap-2">
              <span>Powered by</span>
              <SolanaMark />
              <span>Solana</span>
            </ShinyBadge>
            <h1 className="mt-5 max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-6xl">
              <BlurText text="Turn human reasoning into measurable value." accentWord="measurable value." />
            </h1>
            <p className="mt-5 max-w-2xl text-lg leading-8 text-muted-foreground">
              {APP_NAME} is the cognitive bounty platform where organizations fund high-signal challenges
              to unlock consented human insights—and solvers earn instant SOL rewards for every right answer.
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Button size="lg" asChild>
                <Link href="/get-started">Launch a Challenge</Link>
              </Button>
              <Button size="lg" variant="outline" asChild>
                <Link href="/bounties">Explore Challenges</Link>
              </Button>
            </div>
            <p className="mt-4 text-sm text-muted-foreground">
              Launch in minutes. Zero solver fees. Privacy-first by design.
            </p>
          </div>
          <div className="flex justify-center lg:justify-end">
            <HeroArt />
          </div>
        </div>
      </section>

      <Reveal>
        <section className="mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 lg:px-8" aria-labelledby="what-heading">
          <p className="gradient-text text-xs font-semibold tracking-[0.16em] uppercase">Built for teams &amp; talent</p>
          <h2 id="what-heading" className="mt-3 max-w-3xl text-3xl font-semibold tracking-tight sm:text-4xl">
            High-signal human insights for organizations. Fair rewards for solvers.
          </h2>
          <p className="mt-4 max-w-3xl text-base leading-7 text-muted-foreground">
            Replace slow surveys and low-effort clicks with verifiable cognitive challenges. Set a fixed
            budget, attract motivated human thinkers, and receive clean aggregate performance data—all
            while protecting participant privacy.
          </p>
          <div className="mt-10 grid gap-4 md:grid-cols-3">
            <FeatureCard
              title="AI &amp; research teams"
              body="Benchmark human reasoning on structured problems and measure accuracy, attempts, and speed at scale."
            />
            <FeatureCard
              title="Growth &amp; community teams"
              body="Engage global audiences with funded, verifiable challenges that reward real skill instead of bots."
            />
            <FeatureCard
              title="Privacy-first by default"
              body="No email, passwords, or KYC. Explicit opt-in consent and strictly aggregated analytics protect every participant."
            />
          </div>
        </section>
      </Reveal>

      <Reveal>
        <section
          id="how-it-works"
          className="scroll-mt-24 border-y border-border bg-card/60"
          aria-labelledby="how-heading"
        >
          <div className="mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 lg:px-8">
            <p className="gradient-text text-xs font-semibold tracking-[0.16em] uppercase">How it works</p>
            <h2 id="how-heading" className="mt-3 text-3xl font-semibold tracking-tight sm:text-4xl">
              Two journeys. One marketplace.
            </h2>
            <p className="mt-4 max-w-3xl text-base leading-7 text-muted-foreground">
              Individual researchers and organizations use the same creator tools. Students, specialists,
              and other solvers use the same marketplace. Choose a path when you are ready.
            </p>
            <div className="mt-10 grid gap-6 lg:grid-cols-2">
              <JourneyCard
                eyebrow="For solvers"
                title="Solve, then earn"
                steps={solverSteps}
                href="/bounties"
                cta="Explore Challenges"
              />
              <JourneyCard
                eyebrow="For creators"
                title="Publish, then fund"
                steps={creatorSteps}
                href="/get-started"
                cta="Open creator tools"
              />
            </div>
          </div>
        </section>
      </Reveal>

      <Reveal>
        <section id="for-solvers" className="scroll-mt-24 mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 lg:px-8">
          <p className="gradient-text text-xs font-semibold tracking-[0.16em] uppercase">Benefits for solvers</p>
          <h2 className="mt-3 max-w-3xl text-3xl font-semibold tracking-tight sm:text-4xl">
            Sharpen your mind, stay private, and earn SOL.
          </h2>
          <ul className="mt-6 grid gap-3 text-base leading-7 text-muted-foreground sm:grid-cols-2">
            <li className="glass p-5">100% free to participate—you never pay a fee to attempt a challenge.</li>
            <li className="glass p-5">Fair payouts: every eligible correct solver receives the same configured SOL reward.</li>
            <li className="glass p-5">Your privacy comes first: no account forms, no email, and no individual profiles shared.</li>
            <li className="glass p-5">Fast, transparent SOL payouts sent directly to your wallet once verified.</li>
          </ul>
        </section>
      </Reveal>

      <Reveal>
        <section
          id="for-organizations"
          className="scroll-mt-24 mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 lg:px-8"
        >
          <p className="gradient-text text-xs font-semibold tracking-[0.16em] uppercase">
            Benefits for creators
          </p>
          <h2 className="mt-3 max-w-3xl text-3xl font-semibold tracking-tight sm:text-4xl">
            Launch challenges in minutes and gather ethical human insights.
          </h2>
          <p className="mt-4 max-w-3xl text-base leading-7 text-muted-foreground">
            Whether you are an independent researcher, educator, or organization, anyone can create a
            funded challenge with a single wallet—no lengthy onboarding required.
          </p>
          <div className="mt-8 grid gap-4 md:grid-cols-3">
            <FeatureCard
              title="Secret answers stay safe"
              body="Your predefined answers are encrypted and never exposed to browsers, keeping every challenge fair."
            />
            <FeatureCard
              title="Predictable budgets"
              body="Set your reward per solver and maximum winners upfront. The full reward pool goes straight to solvers."
            />
            <FeatureCard
              title="Actionable group insights"
              body="Track overall participation, accuracy, attempts, and completion time for your challenges in one view."
            />
          </div>
        </section>
      </Reveal>

      <Reveal>
        <section className="border-y border-border bg-card/60" aria-labelledby="research-heading">
          <div className="mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 lg:px-8">
            <p className="gradient-text text-xs font-semibold tracking-[0.16em] uppercase">Privacy &amp; research</p>
            <h2 id="research-heading" className="mt-3 max-w-3xl text-3xl font-semibold tracking-tight sm:text-4xl">
              How we protect your privacy.
            </h2>
            <p className="mt-4 max-w-3xl text-base leading-7 text-muted-foreground">
              We believe human intelligence research should be ethical and transparent. Research metrics are
              only recorded after you explicitly opt in, and analytics only ever show four group-level
              numbers: participant count, correctness, attempt count, and completion time. Individual solver
              records and wallet addresses are never shown in analytics.
            </p>
            <Button className="mt-6" variant="outline" asChild>
              <Link href="/get-started">Get started safely</Link>
            </Button>
          </div>
        </section>
      </Reveal>

      <Reveal>
        <section className="mx-auto w-full max-w-6xl px-4 pb-16 sm:px-6 lg:px-8" aria-labelledby="trust-heading">
          <div className="glass gradient-border relative overflow-hidden p-6 sm:p-8">
            <h2 id="trust-heading" className="text-2xl font-semibold tracking-tight">
              Trust and transparency
            </h2>
            <ul className="mt-5 grid gap-3 text-sm leading-6 text-muted-foreground sm:grid-cols-2">
              {trustPoints.map((point) => (
                <li key={point} className="rounded-xl border border-border bg-background/60 px-4 py-3">
                  {point}
                </li>
              ))}
            </ul>
            <div className="mt-6">
              <HealthStatus />
            </div>
          </div>
        </section>
      </Reveal>

      <section className="border-t border-border">
        <div className="mx-auto flex w-full max-w-6xl flex-col items-start justify-between gap-6 px-4 py-16 sm:px-6 lg:flex-row lg:items-center lg:px-8">
          <div>
            <h2 className="text-3xl font-semibold tracking-tight">Ready to launch your first challenge?</h2>
            <p className="mt-2 max-w-xl text-muted-foreground">
              Publish a funded cognitive challenge in minutes, or jump into the marketplace and start
              earning SOL for your reasoning.
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <Button size="lg" asChild>
              <Link href="/get-started">Get Started</Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/bounties">Explore Challenges</Link>
            </Button>
          </div>
        </div>
      </section>
    </main>
  );
}

function FeatureCard({ title, body }: { title: string; body: string }) {
  return (
    <SpotlightCard className="glass h-full p-5 transition-colors hover:border-brand-purple/50">
      <h3 className="text-lg font-semibold">{title}</h3>
      <p className="mt-2 text-sm leading-6 text-muted-foreground">{body}</p>
    </SpotlightCard>
  );
}

function JourneyCard({
  eyebrow,
  title,
  steps,
  href,
  cta,
}: {
  eyebrow: string;
  title: string;
  steps: readonly { step: string; title: string; body: string }[];
  href: string;
  cta: string;
}) {
  return (
    <SpotlightCard className="glass gradient-border flex h-full flex-col p-6">
      <p className="text-xs font-semibold tracking-[0.14em] text-brand-green uppercase">{eyebrow}</p>
      <h3 className="mt-2 text-2xl font-semibold">{title}</h3>
      <ol className="mt-6 grid flex-1 gap-4">
        {steps.map((item) => (
          <li key={item.step} className="flex gap-3">
            <span className="grid size-8 shrink-0 place-items-center rounded-full bg-brand-gradient font-mono text-xs font-bold text-[#111116]">
              {item.step}
            </span>
            <span>
              <span className="block font-medium">{item.title}</span>
              <span className="mt-1 block text-sm leading-6 text-muted-foreground">{item.body}</span>
            </span>
          </li>
        ))}
      </ol>
      <Button className="mt-8 w-fit" asChild>
        <Link href={href}>{cta}</Link>
      </Button>
    </SpotlightCard>
  );
}

function SolanaMark() {
  return (
    <svg viewBox="0 0 24 20" fill="none" className="size-3.5 shrink-0" aria-hidden="true">
      <defs>
        <linearGradient id="solana-mark-grad" x1="0" y1="0" x2="24" y2="20" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stopColor="#9945FF" />
          <stop offset="100%" stopColor="#14F195" />
        </linearGradient>
      </defs>
      <path d="M4.2 14.5h16.6a.6.6 0 0 1 .42 1.02l-2.6 2.6a1.2 1.2 0 0 1-.85.35H1.2a.6.6 0 0 1-.42-1.02l2.6-2.6a1.2 1.2 0 0 1 .82-.35Z" fill="url(#solana-mark-grad)" />
      <path d="M4.2 1.5h16.6a.6.6 0 0 1 .42 1.02l-2.6 2.6a1.2 1.2 0 0 1-.85.35H1.2a.6.6 0 0 1-.42-1.02l2.6-2.6a1.2 1.2 0 0 1 .82-.35Z" fill="url(#solana-mark-grad)" />
      <path d="M19.8 8H3.2a.6.6 0 0 0-.42 1.02l2.6 2.6c.22.23.53.35.85.35h16.57a.6.6 0 0 0 .42-1.02l-2.6-2.6A1.2 1.2 0 0 0 19.8 8Z" fill="url(#solana-mark-grad)" />
    </svg>
  );
}
