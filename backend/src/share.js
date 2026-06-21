// Share-link logic for SloerShot Cloud: expiry, SHA-256 password protection, and
// self-destruct after N views. Mirrors core/shotcore/src/share.rs byte for byte
// (same SHA-256 hex, same precedence). In-memory store; swap for a DB in production.
import crypto from "node:crypto";

const links = new Map();

export function hashPassword(plain) {
 return crypto.createHash("sha256").update(String(plain), "utf8").digest("hex");
}

export function createLink({ id, createdAt, expiresAt = null, password = null, maxViews = null } = {}) {
 const linkId = id || crypto.randomUUID();
 const link = {
 id: linkId,
 created_at: createdAt ?? Math.floor(Date.now() / 1000),
 expires_at: expiresAt,
 password_hash: password ? hashPassword(password) : null,
 max_views: maxViews,
 views: 0,
 };
 links.set(linkId, link);
 return link;
}

export function getLink(id) {
 return links.get(id);
}

// Returns { ok: true, link } or { ok: false, reason }.
// reason: not_found | expired | exhausted | bad_password
export function tryView(id, now, password) {
 const link = links.get(id);
 if (!link) return { ok: false, reason: "not_found" };
 if (link.expires_at != null && now >= link.expires_at) return { ok: false, reason: "expired" };
 if (link.max_views != null && link.views >= link.max_views) return { ok: false, reason: "exhausted" };
 if (link.password_hash) {
 if (!password || hashPassword(password) !== link.password_hash) {
 return { ok: false, reason: "bad_password" };
 }
 }
 link.views += 1;
 return { ok: true, link };
}

export function _resetForTests() {
 links.clear();
}
