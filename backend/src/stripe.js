import crypto from "node:crypto";

// Verify a Stripe webhook signature (t=...,v1=...) using HMAC-SHA256.
// Same scheme as the official SDK, implemented with node:crypto so no extra dependency.
export function verifyStripeSignature(rawBody, signatureHeader, secret, toleranceSec = 300) {
    if (!signatureHeader || !secret) return false;
    const parts = Object.fromEntries(signatureHeader.split(",").map((p) => p.split("=")));
    const timestamp = parts.t;
    const expected = parts.v1;
    if (!timestamp || !expected) return false;
    const signedPayload = `${timestamp}.${rawBody}`;
    const digest = crypto.createHmac("sha256", secret).update(signedPayload, "utf8").digest("hex");
    const a = Buffer.from(digest);
    const b = Buffer.from(expected);
    if (a.length !== b.length) return false;
    const fresh = Math.abs(Date.now() / 1000 - Number(timestamp)) < toleranceSec;
    return fresh && crypto.timingSafeEqual(a, b);
}

// Map a verified Stripe subscription event to an entitlement action. Pure function so it
// is unit-testable without a live Stripe account. Returns one of:
// { action: "issue", subject, plan, expiresAt }
// { action: "revoke", subject }
// { action: "ignore" }
const VALID_PLANS = new Set(["Basic", "Pro", "Ultra"]);

function planFromSubscription(obj) {
 const item = obj?.items?.data?.[0];
 const raw = item?.price?.nickname || item?.price?.lookup_key || obj?.plan?.nickname || "";
 const norm = String(raw).trim();
 const cap = norm.charAt(0).toUpperCase() + norm.slice(1).toLowerCase();
 return VALID_PLANS.has(cap) ? cap : "Pro";
}

export function mapSubscriptionEvent(event, nowSec) {
 const type = event?.type ?? "";
 const obj = event?.data?.object ?? {};
 const subject = obj.customer || obj.customer_email || null;
 if (type === "customer.subscription.deleted") {
 return { action: "revoke", subject };
 }
 if (type === "customer.subscription.created" || type === "customer.subscription.updated") {
 const status = obj.status;
 if (status === "active" || status === "trialing") {
 const expiresAt = obj.current_period_end || nowSec + 30 * 86400;
 return { action: "issue", subject, plan: planFromSubscription(obj), expiresAt };
 }
 if (status === "canceled" || status === "unpaid" || status === "incomplete_expired") {
 return { action: "revoke", subject };
 }
 }
 return { action: "ignore" };
}
