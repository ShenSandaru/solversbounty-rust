"use client";

import { Button } from "@/components/ui/button";
import { shortAddress, useWalletAuth } from "./use-wallet-auth";

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
            <span
              className="inline-flex h-9 items-center gap-2 rounded-lg border border-brand-green/35 bg-brand-green/10 px-3 font-mono text-xs text-foreground"
              title={`Signed in as ${session.walletAddress}${wallet?.adapter.name ? ` via ${wallet.adapter.name}` : ""}`}
            >
              <span className="size-1.5 rounded-full bg-brand-green" aria-hidden="true" />
              {shortAddress(session.walletAddress)}
              {session.role === "admin" ? <span className="font-sans text-[11px] text-brand-green">admin</span> : null}
            </span>
            <Button type="button" variant="outline" size="sm" disabled={busy} onClick={() => void onLogout()}>
              {phase === "logging-out" ? "Signing out" : "Log out"}
            </Button>
          </>
        ) : address ? (
          <>
            <p className="font-mono text-xs text-muted-foreground" title={address}>
              {shortAddress(address)}
            </p>
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
