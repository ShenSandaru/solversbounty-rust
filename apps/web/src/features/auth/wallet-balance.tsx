"use client";

import { formatLamportsAsSol } from "@solversbounty/shared";
import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { useCallback, useEffect, useState } from "react";
import { shortAddress } from "./short-address";

export function WalletBalance({
  walletAddress,
  role,
  variant = "pill",
}: {
  walletAddress: string;
  role?: string;
  variant?: "pill" | "card";
}) {
  const { connection } = useConnection();
  const [lamports, setLamports] = useState<bigint | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  const fetchBalance = useCallback(async () => {
    setLoading(true);
    setError(false);
    try {
      const key = new PublicKey(walletAddress);
      const value = await connection.getBalance(key, "confirmed");
      setLamports(BigInt(value));
    } catch {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, [connection, walletAddress]);

  useEffect(() => {
    let active = true;
    const key = (() => {
      try {
        return new PublicKey(walletAddress);
      } catch {
        return null;
      }
    })();
    if (!key) {
      return;
    }
    connection
      .getBalance(key, "confirmed")
      .then((value) => {
        if (active) {
          setLamports(BigInt(value));
          setError(false);
          setLoading(false);
        }
      })
      .catch(() => {
        if (active) {
          setError(true);
          setLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, [connection, walletAddress]);

  const formatted = lamports !== null ? `${formatLamportsAsSol(lamports)} SOL` : error ? "Balance unavailable" : "Loading...";

  if (variant === "card") {
    return (
      <section className="surface flex flex-wrap items-center justify-between gap-4 p-4 sm:p-5" aria-label="Connected wallet balance">
        <div>
          <p className="text-xs font-semibold tracking-[0.12em] text-brand-green uppercase">
            Wallet profile{role ? ` · ${role}` : ""}
          </p>
          <p className="mt-1 font-mono text-xs text-muted-foreground" title={walletAddress}>
            {shortAddress(walletAddress)}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="text-right">
            <p className="text-xs text-muted-foreground">Available balance</p>
            <p className="font-mono text-xl font-semibold text-reward">{formatted}</p>
          </div>
          <button
            type="button"
            onClick={() => void fetchBalance()}
            disabled={loading}
            aria-label="Refresh wallet balance"
            title="Refresh balance"
            className="inline-flex size-8 items-center justify-center rounded-lg border border-border bg-card text-xs text-muted-foreground transition-colors hover:border-brand-green/60 hover:text-foreground disabled:opacity-50"
          >
            ↻
          </button>
        </div>
      </section>
    );
  }

  return (
    <span
      className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-border bg-card px-2.5 font-mono text-xs text-reward"
      title={`Available SOL balance for ${walletAddress}`}
    >
      <span>{formatted}</span>
      <button
        type="button"
        onClick={() => void fetchBalance()}
        disabled={loading}
        aria-label="Refresh wallet balance"
        title="Refresh balance"
        className="text-muted-foreground transition-colors hover:text-foreground disabled:opacity-50"
      >
        ↻
      </button>
    </span>
  );
}
