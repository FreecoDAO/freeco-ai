//! Stateless session token authentication for the dashboard.
//! Tokens are HMAC-SHA256 signed and contain username + expiry.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Return the secret used to sign all dashboard sessions.
///
/// The persisted value takes precedence. The legacy fallback remains so existing
/// password configurations keep working until their next password change.
pub(crate) fn derive_dashboard_session_secret(
    config: &openfang_types::config::KernelConfig,
) -> String {
    if !config.auth.session_secret.trim().is_empty() {
        return config.auth.session_secret.clone();
    }

    use sha2::Digest;
    let mut hashes: Vec<&str> = config
        .users
        .iter()
        .filter(|user| user.enabled)
        .filter_map(|user| user.password_hash.as_deref())
        .filter(|hash| !hash.trim().is_empty())
        .collect();
    hashes.sort_unstable();
    if hashes.is_empty() && !config.auth.password_hash.trim().is_empty() {
        hashes.push(config.auth.password_hash.as_str());
    }
    let mut hasher = Sha256::new();
    for hash in hashes {
        hasher.update(hash.as_bytes());
        hasher.update([0]);
    }
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionClaims {
    pub username: String,
    pub role: String,
    pub expiry_unix: i64,
    pub step_up_until_unix: i64,
}

/// Create a session token: base64(username:expiry_unix:hmac_hex)
pub fn create_session_token(username: &str, secret: &str, ttl_hours: u64) -> String {
    create_session_token_with_role(username, "admin", secret, ttl_hours, 0)
}

/// Create a session token with role + optional step-up expiry.
pub fn create_session_token_with_role(
    username: &str,
    role: &str,
    secret: &str,
    ttl_hours: u64,
    step_up_until_unix: i64,
) -> String {
    use base64::Engine;
    let expiry = chrono::Utc::now().timestamp() + (ttl_hours as i64 * 3600);
    let payload = format!("{username}:{role}:{expiry}:{step_up_until_unix}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    base64::engine::general_purpose::STANDARD.encode(format!("{payload}:{signature}"))
}

/// Extract the `openfang_session` cookie value from a `Cookie` header string.
///
/// Returns `None` if the header is absent or the cookie is not present.
/// Used by both the HTTP auth middleware and the WebSocket upgrade handler so
/// that browser sessions established via `sessionLogin()` are honored on both
/// surfaces (issue #1085).
pub fn extract_session_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                c.trim()
                    .strip_prefix("openfang_session=")
                    .map(|v| v.to_string())
            })
        })
}

/// Verify a session token. Returns the username if valid and not expired.
pub fn verify_session_token(token: &str, secret: &str) -> Option<String> {
    verify_session_claims(token, secret).map(|claims| claims.username)
}

/// Verify a session token. Returns full claims if valid and not expired.
pub fn verify_session_claims(token: &str, secret: &str) -> Option<SessionClaims> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(token)
        .ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = decoded_str.split(':').collect();

    let (username, role, expiry_str, step_up_until_str, provided_sig, payload) = match parts.as_slice()
    {
        // v2 format: username:role:expiry:step_up_until:signature
        [username, role, expiry_str, step_up_until_str, provided_sig] => (
            *username,
            *role,
            *expiry_str,
            *step_up_until_str,
            *provided_sig,
            format!("{username}:{role}:{expiry_str}:{step_up_until_str}"),
        ),
        // legacy format: username:expiry:signature
        [username, expiry_str, provided_sig] => (
            *username,
            "admin",
            *expiry_str,
            "0",
            *provided_sig,
            format!("{username}:{expiry_str}"),
        ),
        _ => return None,
    };

    let expiry_unix: i64 = expiry_str.parse().ok()?;
    if chrono::Utc::now().timestamp() > expiry_unix {
        return None;
    }

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(payload.as_bytes());
    let expected_sig = hex::encode(mac.finalize().into_bytes());

    use subtle::ConstantTimeEq;
    if provided_sig.len() != expected_sig.len() {
        return None;
    }
    if provided_sig
        .as_bytes()
        .ct_eq(expected_sig.as_bytes())
        .into()
    {
        Some(SessionClaims {
            username: username.to_string(),
            role: role.to_string(),
            expiry_unix,
            step_up_until_unix: step_up_until_str.parse().ok()?,
        })
    } else {
        None
    }
}

pub fn has_recent_step_up(claims: &SessionClaims) -> bool {
    claims.step_up_until_unix > chrono::Utc::now().timestamp()
}

/// Hash a password with Argon2id for config storage.
///
/// Returns a PHC-format string (e.g. `$argon2id$v=19$m=19456,t=2,p=1$...`).
pub fn hash_password(password: &str) -> String {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Argon2 hashing should not fail with valid inputs")
        .to_string()
}

/// Verify a password against a stored Argon2id hash (PHC string format).
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let hash = hash_password("secret123");
        assert!(
            hash.starts_with("$argon2id$"),
            "should produce Argon2id PHC string"
        );
        assert!(verify_password("secret123", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_hash_produces_unique_salts() {
        let h1 = hash_password("same");
        let h2 = hash_password("same");
        assert_ne!(h1, h2, "each hash should use a unique salt");
        assert!(verify_password("same", &h1));
        assert!(verify_password("same", &h2));
    }

    #[test]
    fn test_rejects_non_argon2_hash() {
        // A plain SHA256 hex string should no longer be accepted.
        use sha2::Digest;
        let sha256_hash = hex::encode(sha2::Sha256::digest(b"password"));
        assert!(!verify_password("password", &sha256_hash));
    }

    #[test]
    fn test_create_and_verify_token() {
        let token = create_session_token("admin", "my-secret", 1);
        let user = verify_session_token(&token, "my-secret");
        assert_eq!(user, Some("admin".to_string()));
    }

    #[test]
    fn test_create_and_verify_token_with_role() {
        let token = create_session_token_with_role("kiddo", "kid", "my-secret", 1, 12345);
        let claims = verify_session_claims(&token, "my-secret").expect("valid claims");
        assert_eq!(claims.username, "kiddo");
        assert_eq!(claims.role, "kid");
        assert_eq!(claims.step_up_until_unix, 12345);
    }

    #[test]
    fn test_token_wrong_secret() {
        let token = create_session_token("admin", "my-secret", 1);
        let user = verify_session_token(&token, "wrong-secret");
        assert_eq!(user, None);
    }

    #[test]
    fn test_token_invalid_base64() {
        let user = verify_session_token("not-valid-base64!!!", "secret");
        assert_eq!(user, None);
    }

    #[test]
    fn test_rejects_garbage_input() {
        assert!(!verify_password("x", "short"));
        assert!(!verify_password("x", ""));
    }

    #[test]
    fn test_verify_malformed_argon2_hash() {
        // Starts with $argon2 but is not a valid PHC string.
        assert!(!verify_password("x", "$argon2id$garbage"));
    }

    #[test]
    fn test_extract_session_cookie_present() {
        let mut h = axum::http::HeaderMap::new();
        h.insert(
            "cookie",
            "foo=bar; openfang_session=abc.def.ghi; baz=qux"
                .parse()
                .unwrap(),
        );
        assert_eq!(extract_session_cookie(&h).as_deref(), Some("abc.def.ghi"));
    }

    #[test]
    fn test_extract_session_cookie_absent() {
        let mut h = axum::http::HeaderMap::new();
        h.insert("cookie", "foo=bar; baz=qux".parse().unwrap());
        assert_eq!(extract_session_cookie(&h), None);
    }

    #[test]
    fn test_extract_session_cookie_no_header() {
        let h = axum::http::HeaderMap::new();
        assert_eq!(extract_session_cookie(&h), None);
    }

    #[test]
    fn test_extract_session_cookie_only_value() {
        let mut h = axum::http::HeaderMap::new();
        h.insert("cookie", "openfang_session=lonely".parse().unwrap());
        assert_eq!(extract_session_cookie(&h).as_deref(), Some("lonely"));
    }
}
