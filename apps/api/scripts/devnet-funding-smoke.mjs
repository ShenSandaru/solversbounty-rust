/**
 * Live ADR-012 funding smoke against local API + Solana Devnet.
 * Uses a throwaway keypair (not Phantom). Does not print secret keys.
 *
 * Usage (from repo root):
 *   node apps/api/scripts/devnet-funding-smoke.mjs
 */
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../../..");
const pnpmWeb3 = path.join(
  repoRoot,
  "node_modules/.pnpm/@solana+web3.js@1.99.0_buff_686fde5d6a3c95c91961e7bdf9cc725f/node_modules/@solana/web3.js/package.json",
);
const require = createRequire(pnpmWeb3);
const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} = require("@solana/web3.js");
const bs58 = require("bs58");
const { ed25519 } = require("@noble/curves/ed25519");

function encodeSignatureUrlSafe(sigBytes) {
  return Buffer.from(sigBytes)
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");
}

const API = process.env.API_URL || "http://localhost:3001";
const RPC = process.env.SOLANA_RPC_URL || "https://api.devnet.solana.com";

const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
);

function loadDotEnv() {
  const envPath = path.join(repoRoot, "apps/api/.env");
  const out = {};
  try {
    const text = fs.readFileSync(envPath, "utf8");
    for (const line of text.split(/\r?\n/)) {
      const t = line.trim();
      if (!t || t.startsWith("#")) continue;
      const i = t.indexOf("=");
      if (i < 1) continue;
      out[t.slice(0, i)] = t.slice(i + 1);
    }
  } catch {
    /* optional */
  }
  return out;
}

function keypairFromSecret(secret) {
  const trimmed = secret.trim();
  if (trimmed.startsWith("[")) {
    const arr = JSON.parse(trimmed);
    return Keypair.fromSecretKey(Uint8Array.from(arr));
  }
  if (/^[0-9a-fA-F]{128}$/.test(trimmed)) {
    return Keypair.fromSecretKey(Buffer.from(trimmed, "hex"));
  }
  try {
    return Keypair.fromSecretKey(bs58.decode(trimmed));
  } catch {
    return Keypair.fromSecretKey(Buffer.from(trimmed, "base64"));
  }
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function parseCookies(res) {
  const raw = res.headers.getSetCookie?.() || [];
  const jar = {};
  for (const line of raw) {
    const [pair] = line.split(";");
    const i = pair.indexOf("=");
    if (i > 0) jar[pair.slice(0, i)] = pair.slice(i + 1);
  }
  return jar;
}

function cookieHeader(jar) {
  return Object.entries(jar)
    .map(([k, v]) => `${k}=${v}`)
    .join("; ");
}

async function api(pathName, { method = "GET", body, jar, csrf } = {}) {
  const headers = { "content-type": "application/json" };
  if (jar && Object.keys(jar).length) headers.cookie = cookieHeader(jar);
  if (csrf) headers["x-csrf-token"] = csrf;
  const res = await fetch(`${API}${pathName}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let json = null;
  try {
    json = text ? JSON.parse(text) : null;
  } catch {
    json = { raw: text };
  }
  Object.assign(jar || {}, parseCookies(res));
  return { status: res.status, json, jar };
}

async function fundTransfer(connection, payer, treasury, lamports, memo) {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: treasury,
      lamports,
    }),
    new TransactionInstruction({
      keys: [{ pubkey: payer.publicKey, isSigner: true, isWritable: true }],
      programId: MEMO_PROGRAM_ID,
      data: Buffer.from(memo, "utf8"),
    }),
  );
  return sendAndConfirmTransaction(connection, tx, [payer], {
    commitment: "confirmed",
  });
}

async function submitFunding(jar, csrf, bountyId, purpose, signature) {
  let res = await api(`/bounties/${bountyId}/funding/${purpose}`, {
    method: "POST",
    jar,
    csrf,
    body: { signature },
  });
  for (let i = 0; i < 10 && (res.status === 409 || res.status === 503); i++) {
    await sleep(2000);
    res = await api(`/bounties/${bountyId}/funding/${purpose}`, {
      method: "POST",
      jar,
      csrf,
      body: { signature },
    });
  }
  return res;
}

async function main() {
  const fileEnv = loadDotEnv();
  const TREASURY =
    process.env.TREASURY_PUBLIC_KEY ||
    fileEnv.TREASURY_PUBLIC_KEY ||
    "DZoqPdQQStLw2VRMeQ6ZniqJ8ReNrsa5n42KfXxf2ank";
  const treasurySecret =
    process.env.TREASURY_SECRET_KEY || fileEnv.TREASURY_SECRET_KEY || "";
  if (!treasurySecret) {
    throw new Error("TREASURY_SECRET_KEY missing for bootstrap (RPC faucet closed)");
  }

  const health = await fetch(`${API}/health`).then((r) => r.json());
  if (health.status !== "ok") throw new Error("API health failed");

  const connection = new Connection(RPC, "confirmed");
  const payer = Keypair.generate();
  const wallet = payer.publicKey.toBase58();
  console.log("smoke wallet:", wallet);
  console.log("treasury:", TREASURY);

  // Devnet RPC faucet returns 410; bootstrap a few SOL from the configured treasury.
  const treasurySigner = keypairFromSecret(treasurySecret);
  if (treasurySigner.publicKey.toBase58() !== TREASURY) {
    throw new Error("TREASURY_SECRET_KEY does not match TREASURY_PUBLIC_KEY");
  }
  const bootstrapLamports = Math.floor(0.05 * LAMPORTS_PER_SOL);
  const bootstrapTx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: treasurySigner.publicKey,
      toPubkey: payer.publicKey,
      lamports: bootstrapLamports,
    }),
  );
  const bootstrapSig = await sendAndConfirmTransaction(
    connection,
    bootstrapTx,
    [treasurySigner],
    { commitment: "confirmed" },
  );
  console.log("bootstrap funded:", bootstrapSig);

  const jar = {};
  const nonceRes = await api("/auth/nonce", {
    method: "POST",
    body: { walletAddress: wallet },
    jar,
  });
  if (nonceRes.status !== 200) {
    throw new Error(
      `nonce failed: ${nonceRes.status} ${JSON.stringify(nonceRes.json)}`,
    );
  }
  const messageBytes = Buffer.from(nonceRes.json.message, "utf8");
  const seed = payer.secretKey.slice(0, 32);
  const sigBytes = ed25519.sign(messageBytes, seed);
  // API expects URL-safe base64 (no pad), not base58 — see decode_signature.
  const signature = encodeSignatureUrlSafe(sigBytes);

  const loginRes = await api("/auth/login", {
    method: "POST",
    body: { walletAddress: wallet, signature },
    jar,
  });
  if (loginRes.status !== 200) {
    throw new Error(
      `login failed: ${loginRes.status} ${JSON.stringify(loginRes.json)}`,
    );
  }
  const csrf = loginRes.json.csrfToken;
  if (!csrf) throw new Error("login missing csrfToken");
  console.log("session ok");

  const deadline = new Date(Date.now() + 7 * 24 * 3600 * 1000).toISOString();
  const createRes = await api("/bounties", {
    method: "POST",
    jar,
    csrf,
    body: {
      title: "Devnet funding smoke",
      description: "Automated ADR-012 live funding check.",
      instructions: "Ignore; smoke only.",
      prompt: "What is 2+2?",
      category: "logic",
      expectedAnswer: "4",
      verificationMethod: "exact",
      rewardSol: "0.01",
      maxRewardedSolvers: 1,
      attemptLimit: 3,
      deadline,
    },
  });
  if (createRes.status !== 200 && createRes.status !== 201) {
    throw new Error(
      `create failed: ${createRes.status} ${JSON.stringify(createRes.json)}`,
    );
  }
  const bountyId = createRes.json.id;
  console.log("draft bounty:", bountyId);

  const rewardPool = 10_000_000;
  const commission = 500_000;
  const treasury = new PublicKey(TREASURY);
  const poolMemo = `sb:deposit:${bountyId}:reward_pool`;
  const commissionMemo = `sb:deposit:${bountyId}:commission`;

  const poolSig = await fundTransfer(
    connection,
    payer,
    treasury,
    rewardPool,
    poolMemo,
  );
  console.log("reward_pool sig:", poolSig);

  const poolPost = await submitFunding(
    jar,
    csrf,
    bountyId,
    "reward-pool",
    poolSig,
  );
  if (poolPost.status !== 200) {
    throw new Error(
      `reward_pool post: ${poolPost.status} ${JSON.stringify(poolPost.json)}`,
    );
  }
  console.log("reward_pool confirmed");

  const commissionSig = await fundTransfer(
    connection,
    payer,
    treasury,
    commission,
    commissionMemo,
  );
  console.log("commission sig:", commissionSig);

  const commissionPost = await submitFunding(
    jar,
    csrf,
    bountyId,
    "commission",
    commissionSig,
  );
  if (commissionPost.status !== 200) {
    throw new Error(
      `commission post: ${commissionPost.status} ${JSON.stringify(commissionPost.json)}`,
    );
  }
  console.log("commission confirmed");

  const activate = await api(`/bounties/${bountyId}/activate`, {
    method: "POST",
    jar,
    csrf,
  });
  if (activate.status !== 200) {
    throw new Error(
      `activate: ${activate.status} ${JSON.stringify(activate.json)}`,
    );
  }
  console.log("activated status:", activate.json.status);
  if (activate.json.status !== "active") {
    throw new Error(`expected active, got ${activate.json.status}`);
  }
  console.log("ADR-012 live funding smoke OK");
}

main().catch((err) => {
  console.error("FUNDING_SMOKE_FAILED", err?.message || err);
  process.exit(1);
});
