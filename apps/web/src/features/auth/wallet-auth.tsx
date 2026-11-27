"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { shortAddress, useWalletAuth } from "./use-wallet-auth";
import { WalletBalance } from "./wallet-balance";

export function WalletAuth() {
  const {
    session,
    address,
    connected,
    connecting,
    wallet,
    ready,
    phase,
    busy,
    error,
    prepared,
    mismatch,
    signMessage,
    onConnect,
    onSignIn,
    onDisconnect,
    onLogout,
  } = useWalletAuth();

  return (
    <div className="flex shrink-0 flex-col items-end gap-1.5">
      <div className="flex flex-wrap items-center justify-end gap-2">
        {session ? (
          <>
            <WalletBalance walletAddress={session.walletAddress} role={session.role} variant="pill" />
            <Link
              href="/dashboard?tab=profile"
              className="inline-flex h-9 items-center gap-2 rounded-lg border border-brand-green/35 bg-brand-green/10 px-2.5 font-mono text-xs text-foreground transition-colors hover:border-brand-green/70"
              title={`Profile · Signed in as ${session.walletAddress}${wallet?.adapter.name ? ` via ${wallet.adapter.name}` : ""}`}
            >
              <ProfileAvatar active />
              <span>{shortAddress(session.walletAddress)}</span>
              {session.role === "admin" ? <span className="font-sans text-[11px] text-brand-green">admin</span> : null}
            </Link>
            <Button type="button" variant="outline" size="sm" disabled={busy} onClick={() => void onLogout()}>
              {phase === "logging-out" ? "Signing out" : "Log out"}
            </Button>
          </>
        ) : address ? (
          <>
            <span className="inline-flex h-9 items-center gap-2 rounded-lg border border-border bg-card px-2.5 font-mono text-xs text-muted-foreground" title={address}>
              <ProfileAvatar active={false} />
              <span>{shortAddress(address)}</span>
            </span>
            <Button type="button" size="sm" disabled={busy || !signMessage || !prepared} onClick={onSignIn}>
              {phase === "signing" ? "Waiting for signature" : prepared ? "Sign in" : "Preparing sign-in"}
            </Button>
            <Button type="button" variant="outline" size="sm" disabled={busy} onClick={() => void onDisconnect()}>
              Disconnect
            </Button>
          </>
        ) : (
          <Button type="button" size="sm" disabled={busy} onClick={() => void onConnect()}>
            {phase === "connecting" || connecting ? "Connecting" : ready ? "Connect wallet" : "Checking session"}
          </Button>
        )}
      </div>
      {session && !connected ? (
        <p className="text-xs text-muted-foreground">Wallet disconnected. Log out ends the server session.</p>
      ) : null}
      {mismatch ? (
        <p className="text-xs text-warning">Connected wallet differs from the signed-in wallet.</p>
      ) : null}
      {address && !signMessage && !session ? (
        <p className="text-xs text-muted-foreground">This wallet does not support message signing.</p>
      ) : null}
      {error ? (
        <p className="text-xs text-destructive" role="alert">
          {error}
        </p>
      ) : null}
    </div>
  );
}

function ProfileAvatar({ active }: { active: boolean }) {
  return (
    <span
      className={`relative grid size-6 shrink-0 place-items-center rounded-full ${
        active ? "bg-brand-gradient text-[#111116]" : "bg-secondary text-muted-foreground"
      }`}
      aria-hidden="true"
    >
      <svg viewBox="0 0 20 20" fill="none" className="size-3.5">
        <circle cx="10" cy="7" r="3" stroke="currentColor" strokeWidth="1.7" />
        <path d="M4.5 16.2c1.2-2.4 3.2-3.7 5.5-3.7s4.3 1.3 5.5 3.7" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
      </svg>
    </span>
  );
}
