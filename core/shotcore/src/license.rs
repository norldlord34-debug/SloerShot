//! Offline license and entitlement validation.
//!
//! The backend signs an `Entitlement` with an ed25519 private key. The app ships
//! the matching public key and verifies entitlements fully offline, so a paying
//! user keeps working without a network round-trip. A short grace window covers
//! clock skew and brief outages just after expiry. Because the secret stays on
//! the server, the app cannot forge or escalate an entitlement.

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LicenseError {
    #[error("malformed token")]
    Malformed,
    #[error("signature verification failed")]
    BadSignature,
    #[error("entitlement expired")]
    Expired,
    #[error("encoding error")]
    Encoding,
}

/// Subscription tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Plan {
    Basic,
    Pro,
    Ultra,
}

/// Signed claims describing what a user is entitled to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entitlement {
    pub subject: String,
    pub plan: Plan,
    /// Unix seconds when issued.
    pub issued_at: i64,
    /// Unix seconds after which the entitlement is no longer valid.
    pub expires_at: i64,
    /// Feature flags unlocked, for example video, ocr, cloud.
    pub features: Vec<String>,
}

impl Entitlement {
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn is_active_at(&self, now: i64) -> bool {
        now <= self.expires_at
    }
}

/// Generate a fresh signing/verifying key pair (used by the backend and tests).
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    use rand::rngs::OsRng;
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

/// Issue a signed token: base64url(claims_json).base64url(signature).
pub fn issue(signing_key: &SigningKey, ent: &Entitlement) -> Result<String, LicenseError> {
    let claims = serde_json::to_vec(ent).map_err(|_| LicenseError::Encoding)?;
    let sig = signing_key.sign(&claims);
    Ok(format!(
        "{}.{}",
        B64.encode(&claims),
        B64.encode(sig.to_bytes())
    ))
}

fn verify_signature(
    verifying_key: &VerifyingKey,
    token: &str,
) -> Result<(Entitlement, Vec<u8>), LicenseError> {
    let (c, s) = token.split_once(".").ok_or(LicenseError::Malformed)?;
    let claims_bytes = B64.decode(c).map_err(|_| LicenseError::Encoding)?;
    let sig_bytes = B64.decode(s).map_err(|_| LicenseError::Encoding)?;
    let arr: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| LicenseError::Malformed)?;
    let sig = Signature::from_bytes(&arr);
    verifying_key
        .verify(&claims_bytes, &sig)
        .map_err(|_| LicenseError::BadSignature)?;
    let ent: Entitlement =
        serde_json::from_slice(&claims_bytes).map_err(|_| LicenseError::Malformed)?;
    Ok((ent, claims_bytes))
}

/// Verify a token and enforce strict expiry at `now` (unix seconds).
pub fn verify(
    verifying_key: &VerifyingKey,
    token: &str,
    now: i64,
) -> Result<Entitlement, LicenseError> {
    let (ent, _) = verify_signature(verifying_key, token)?;
    if ent.is_active_at(now) {
        Ok(ent)
    } else {
        Err(LicenseError::Expired)
    }
}

/// Verify a token but allow `grace_secs` of validity past expiry (offline grace).
pub fn verify_with_grace(
    verifying_key: &VerifyingKey,
    token: &str,
    now: i64,
    grace_secs: i64,
) -> Result<Entitlement, LicenseError> {
    let (ent, _) = verify_signature(verifying_key, token)?;
    if now <= ent.expires_at + grace_secs {
        Ok(ent)
    } else {
        Err(LicenseError::Expired)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let pair = std::str::from_utf8(&bytes[i..i + 2]).ok()?;
        out.push(u8::from_str_radix(pair, 16).ok()?);
        i += 2;
    }
    Some(out)
}

/// Hex-encode a public key for embedding in the app.
pub fn public_key_hex(vk: &VerifyingKey) -> String {
    to_hex(&vk.to_bytes())
}

/// Parse a hex-encoded public key (the inverse of `public_key_hex`).
pub fn verifying_key_from_hex(s: &str) -> Result<VerifyingKey, LicenseError> {
    let bytes = from_hex(s).ok_or(LicenseError::Encoding)?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| LicenseError::Encoding)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| LicenseError::BadSignature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;

    fn sample() -> Entitlement {
        Entitlement {
            subject: "user@example.com".to_string(),
            plan: Plan::Pro,
            issued_at: 1000,
            expires_at: 2000,
            features: vec!["ocr".to_string(), "video".to_string()],
        }
    }

    #[test]
    fn issue_and_verify_roundtrip() {
        let (sk, vk) = generate_keypair();
        let token = issue(&sk, &sample()).unwrap();
        let ent = verify(&vk, &token, 1500).unwrap();
        assert_eq!(ent.subject, "user@example.com");
        assert_eq!(ent.plan, Plan::Pro);
        assert!(ent.has_feature("ocr"));
        assert!(!ent.has_feature("cloud"));
    }

    #[test]
    fn wrong_key_is_rejected() {
        let (sk, _) = generate_keypair();
        let (_, other_vk) = generate_keypair();
        let token = issue(&sk, &sample()).unwrap();
        assert_eq!(
            verify(&other_vk, &token, 1500),
            Err(LicenseError::BadSignature)
        );
    }

    #[test]
    fn tampered_claims_fail_signature() {
        let (sk, vk) = generate_keypair();
        let token = issue(&sk, &sample()).unwrap();
        let (cpart, spart) = token.split_once(".").unwrap();
        let mut claims: Entitlement = serde_json::from_slice(&B64.decode(cpart).unwrap()).unwrap();
        claims.plan = Plan::Ultra;
        let tampered = format!(
            "{}.{}",
            B64.encode(serde_json::to_vec(&claims).unwrap()),
            spart
        );
        assert_eq!(
            verify(&vk, &tampered, 1500),
            Err(LicenseError::BadSignature)
        );
    }

    #[test]
    fn expiry_is_enforced() {
        let (sk, vk) = generate_keypair();
        let token = issue(&sk, &sample()).unwrap();
        assert_eq!(verify(&vk, &token, 2500), Err(LicenseError::Expired));
    }

    #[test]
    fn grace_window_extends_validity() {
        let (sk, vk) = generate_keypair();
        let token = issue(&sk, &sample()).unwrap();
        assert!(verify_with_grace(&vk, &token, 2500, 1000).is_ok());
        assert_eq!(
            verify_with_grace(&vk, &token, 3500, 1000),
            Err(LicenseError::Expired)
        );
    }

    #[test]
    fn malformed_token_is_detected() {
        let (_, vk) = generate_keypair();
        assert_eq!(
            verify(&vk, "no-dot-here", 1500),
            Err(LicenseError::Malformed)
        );
    }

    #[test]
    fn public_key_hex_roundtrip() {
        let (_, vk) = generate_keypair();
        let hex = public_key_hex(&vk);
        let back = verifying_key_from_hex(&hex).unwrap();
        assert_eq!(back.to_bytes(), vk.to_bytes());
    }
}
