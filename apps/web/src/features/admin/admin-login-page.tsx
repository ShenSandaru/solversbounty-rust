"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { PageContainer } from "@/components/page-container";
import { PageHeader } from "@/components/page-header";
import { shortAddress, useWalletAuth } from "@/features/auth/use-wallet-auth";

export function AdminLoginPage() {
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

  const isAdmin = session?.role === "admin";

  return (
    <PageContainer>
      <PageHeader
        eyebrow="Administrator"
        title="Admin sign-in"
        description="Use the same Phantom wallet login. Admin access is granted only when this wallet is listed in the API ADMIN_WALLET_ALLOWLIST. This is a minimal entry screen, not a full admin console."
      />

      <section className="surface mt-8 max-w-xl p-6" aria-labelledby="admin-login-heading">
        <h2 id="admin-login-heading" className="text-lg font-semibold">
          Wallet session
        </h2>
        <p className="mt-2 text-sm leading-6 text-muted-foreground">
          Connect Phantom, then sign the server nonce message. An empty allowlist means there are no admins.
        </p>

        <div className="mt-5 flex flex-wrap gap-2">
          {address ? (
            <Button type="button" variant="outline" disabled={busy} onClick={() => void onDisconnect()}>
              Disconnect
            </Button>
          ) : (
            <Button type="button" disabled={busy || !ready} onClick={() => void onConnect()}>
              {phase === "connecting" || connecting ? "Connecting" : "Connect Phantom"}
            </Button>
          )}
          {session ? (
            <Button type="button" variant="secondary" disabled={busy} onClick={() => void onLogout()}>
              {phase === "logging-out" ? "Signing out" : "Log out"}
            </Button>
          ) : address ? (
            <Button type="button" disabled={busy || !signMessage || !prepared} onClick={onSignIn}>
              {phase === "signing" ? "Waiting for signature" : prepared ? "Sign in as admin wallet" : "Preparing sign-in"}
            </Button>
          ) : null}
        </div>

        <dl className="mt-5 grid gap-3 text-sm">
          <div>
            <dt className="text-muted-foreground">Connected wallet</dt>
            <dd className="mt-1 font-mono">{address ? shortAddress(address) : ready ? "None" : "Checking…"}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Server session</dt>
            <dd className="mt-1">
              {session
                ? `${shortAddress(session.walletAddress)}${session.role === "admin" ? " · admin" : " · user"}${
                    wallet?.adapter.name ? ` · ${wallet.adapter.name}` : ""
                  }${connected ? "" : " · wallet disconnected"}`
                : "Not signed in"}
            </dd>
          </div>
        </dl>

        {mismatch ? (
          <p className="mt-4 text-sm text-muted-foreground">
            The connected wallet differs from the signed-in wallet. Disconnect or log out before continuing.
          </p>
        ) : null}
        {address && !signMessage && !session ? (
          <p className="mt-4 text-sm text-muted-foreground">
            This wallet cannot sign in. It does not support message signing.
          </p>
        ) : null}
        {error ? (
          <p className="mt-4 text-sm text-destructive" role="alert">
            {error}
          </p>
        ) : null}
      </section>

      {session && !isAdmin ? (
        <div
          className="mt-6 max-w-xl rounded-xl border border-warning/40 bg-warning/[0.07] px-4 py-3 text-sm leading-6 text-muted-foreground"
          role="status"
        >
          You are signed in, but this wallet is not on <code className="font-mono text-foreground">ADMIN_WALLET_ALLOWLIST</code>.
          Add the address in <code className="font-mono text-foreground">apps/api/.env</code>, restart the API, then sign in
          again.
        </div>
      ) : null}

      {isAdmin ? (
        <div className="mt-6 max-w-xl rounded-xl border border-brand-green/40 bg-brand-green/10 px-4 py-4 text-sm leading-6">
          <p className="font-medium text-foreground">Admin session active</p>
          <p className="mt-2 text-muted-foreground">
            Ordinary product screens still use Solver/Creator mode. Protected ops endpoints such as{" "}
            <code className="font-mono text-foreground">GET /admin/payouts</code> accept this session. There is no
            separate moderation dashboard in this MVP.
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            <Button asChild>
              <Link href="/dashboard">Open workspace</Link>
            </Button>
            <Button variant="outline" asChild>
              <Link href="/bounties">Marketplace</Link>
            </Button>
          </div>
        </div>
      ) : null}
    </PageContainer>
  );
}
