import crypto from "node:crypto";

const PLANS = new Set(["Basic", "Pro", "Ultra"]);

const PLAN_FEATURES = {
    Basic: ["capture", "annotate", "export"],
    Pro: ["capture", "annotate", "export", "ocr", "beautify", "video", "scroll"],
    Ultra: ["capture", "annotate", "export", "ocr", "beautify", "video", "scroll", "cloud", "ai"],
};

function b64url(buf) {
    return Buffer.from(buf).toString("base64url");
}

// Build claims. Field names match shotcore::license::Entitlement exactly.
export function buildClaims({ subject, plan, issuedAt, expiresAt, features }) {
    if (!PLANS.has(plan)) throw new Error(`unknown plan: ${plan}`);
    return {
        subject,
        plan,
        issued_at: issuedAt,
        expires_at: expiresAt,
        features: features ?? PLAN_FEATURES[plan],
    };
}

// Token = base64url(claims_json) + "." + base64url(ed25519 signature over those bytes).
export function issueToken(privateKey, claims) {
    const claimsBytes = Buffer.from(JSON.stringify(claims), "utf8");
    const signature = crypto.sign(null, claimsBytes, privateKey);
    return `${b64url(claimsBytes)}.${b64url(signature)}`;
}
