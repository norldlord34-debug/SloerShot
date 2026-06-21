// Real credential verification for the login endpoint. Passwords are checked against
// a SHA-256 hashed user store (no plaintext compare, timing-safe). The store is loaded
// from the SLOERSHOT_USERS_JSON env var: a JSON object of { "email": "sha256hex(password)" }.
// Swap loadUsers for a database lookup in production; the verify logic stays the same.
import crypto from "node:crypto";
import { hashPassword } from "./share.js";

export function loadUsers() {
 const raw = process.env.SLOERSHOT_USERS_JSON;
 if (!raw) return {};
 try {
 const obj = JSON.parse(raw);
 return obj && typeof obj === "object" ? obj : {};
 } catch {
 return {};
 }
}

// Constant-time verification of a plaintext password against the stored SHA-256 hash.
export function verifyCredentials(email, password, users) {
 if (!email || !password) return false;
 const stored = users[email];
 if (!stored) return false;
 const got = hashPassword(password);
 const a = Buffer.from(got, "utf8");
 const b = Buffer.from(String(stored), "utf8");
 if (a.length !== b.length) return false;
 return crypto.timingSafeEqual(a, b);
}
