import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const KEY_DIR = path.resolve(HERE, "..", "keys");
const PRIV_PATH = path.join(KEY_DIR, "ed25519_private.pem");
const PUB_PATH = path.join(KEY_DIR, "ed25519_public.pem");

// Load the signing key pair, generating and persisting one on first run.
export function loadOrCreateKeyPair() {
    if (fs.existsSync(PRIV_PATH) && fs.existsSync(PUB_PATH)) {
        const privateKey = crypto.createPrivateKey(fs.readFileSync(PRIV_PATH));
        const publicKey = crypto.createPublicKey(fs.readFileSync(PUB_PATH));
        return { privateKey, publicKey };
    }
    const { privateKey, publicKey } = crypto.generateKeyPairSync("ed25519");
    fs.mkdirSync(KEY_DIR, { recursive: true });
    fs.writeFileSync(PRIV_PATH, privateKey.export({ type: "pkcs8", format: "pem" }));
    fs.writeFileSync(PUB_PATH, publicKey.export({ type: "spki", format: "pem" }));
    return { privateKey, publicKey };
}

// Raw 32-byte ed25519 public key as hex (the format shotcore embeds and verifies).
export function publicKeyHex(publicKey) {
    const jwk = publicKey.export({ format: "jwk" });
    return Buffer.from(jwk.x, "base64url").toString("hex");
}
