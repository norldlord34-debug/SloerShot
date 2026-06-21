import { loadOrCreateKeyPair, publicKeyHex } from "../src/keys.js";
import { buildClaims, issueToken } from "../src/entitlement.js";

const { privateKey, publicKey } = loadOrCreateKeyPair();
const nowSec = Math.floor(Date.now() / 1000);
const claims = buildClaims({ subject: "demo@sloershot.app", plan: "Pro", issuedAt: nowSec, expiresAt: nowSec + 3600 });
const token = issueToken(privateKey, claims);
console.log(JSON.stringify({ public_key_hex: publicKeyHex(publicKey), token, now: nowSec, claims }));
