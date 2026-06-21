import "dotenv/config";
import express from "express";
import crypto from "node:crypto";
import { loadOrCreateKeyPair, publicKeyHex } from "./keys.js";
import { buildClaims, issueToken } from "./entitlement.js";
import { verifyStripeSignature, mapSubscriptionEvent } from "./stripe.js";
import { createLink, tryView } from "./share.js";
import { loadUsers, verifyCredentials } from "./auth.js";
const PORT = process.env.PORT || 8787;
const { privateKey, publicKey } = loadOrCreateKeyPair();
const PUBKEY_HEX = publicKeyHex(publicKey);
const app = express();
// Stripe webhook needs the raw body for signature verification; register it before json().
app.use("/v1/stripe/webhook", express.raw({ type: "*/*" }));
app.use(express.json());
const now = () => Math.floor(Date.now() / 1000);
app.get("/health", (req, res) => res.json({ status: "ok", service: "sloershot-backend" }));
// Public key for the apps to embed and verify entitlements offline.
app.get("/v1/public-key", (req, res) => res.json({ alg: "ed25519", public_key_hex: PUBKEY_HEX }));
// Login: verifies a password against the SHA-256 hashed user store (see auth.js).
app.post("/v1/auth/login", (req, res) => {
 const { email, password } = req.body ?? {};
 if (!email || !password) return res.status(400).json({ error: "email and password required" });
 const users = loadUsers();
 if (Object.keys(users).length === 0) return res.status(503).json({ error: "auth not configured" });
 if (!verifyCredentials(email, password, users)) return res.status(401).json({ error: "invalid credentials" });
 res.json({ session: crypto.randomUUID(), email });
});
// Issue a signed entitlement for a subject + plan.
app.post("/v1/entitlement", (req, res) => {
 const body = req.body ?? {};
 const subject = body.subject;
 const plan = body.plan ?? "Pro";
 const days = body.days ?? 30;
 if (!subject) return res.status(400).json({ error: "subject required" });
 try {
 const claims = buildClaims({ subject, plan, issuedAt: now(), expiresAt: now() + days * 86400 });
 res.json({ token: issueToken(privateKey, claims), claims });
 } catch (e) {
 res.status(400).json({ error: String(e.message ?? e) });
 }
});
// Stripe subscription webhook -> issues or revokes entitlements based on the event.
app.post("/v1/stripe/webhook", (req, res) => {
 const secret = process.env.STRIPE_WEBHOOK_SECRET;
 const sig = req.header("Stripe-Signature");
 const raw = Buffer.isBuffer(req.body) ? req.body.toString("utf8") : "";
 if (secret && !verifyStripeSignature(raw, sig, secret)) {
 return res.status(400).json({ error: "invalid signature" });
 }
 let event;
 try {
 event = JSON.parse(raw || "{}");
 } catch {
 return res.status(400).json({ error: "invalid json" });
 }
 const mapped = mapSubscriptionEvent(event, now());
 if (mapped.action === "issue" && mapped.subject) {
 const claims = buildClaims({ subject: mapped.subject, plan: mapped.plan, issuedAt: now(), expiresAt: mapped.expiresAt });
 const token = issueToken(privateKey, claims);
 return res.json({ received: true, action: "issue", subject: mapped.subject, plan: mapped.plan, token });
 }
 if (mapped.action === "revoke" && mapped.subject) {
 return res.json({ received: true, action: "revoke", subject: mapped.subject });
 }
 res.json({ received: true, action: "ignore" });
});
// Create a share link (optional expiry, password, self-destruct view cap).
app.post("/v1/share", (req, res) => {
 const body = req.body ?? {};
 const link = createLink({
 expiresAt: body.expires_at ?? null,
 password: body.password ?? null,
 maxViews: body.max_views ?? null,
 });
 res.json({ id: link.id, expires_at: link.expires_at, max_views: link.max_views, url: `/s/${link.id}` });
});
// Resolve a share link, enforcing expiry/password/self-destruct.
app.post("/v1/share/:id", (req, res) => {
 const result = tryView(req.params.id, now(), req.body?.password);
 if (!result.ok) {
 const code = result.reason === "not_found" ? 404 : result.reason === "bad_password" ? 401 : 410;
 return res.status(code).json({ error: result.reason });
 }
 res.json({ id: result.link.id, views: result.link.views });
});
export { app };
if (!process.env.SLOERSHOT_NO_LISTEN)
 app.listen(PORT, () => {
 console.log(`SloerShot backend listening on http://localhost:${PORT}`);
 console.log(`ed25519 public key (hex): ${PUBKEY_HEX}`);
 });
