//! JWT session issuing and verification.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::roles::Role;

/// JWT claims used by OpenWebHMI sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionClaims {
    /// Subject user id.
    pub sub: String,
    /// Username.
    pub username: String,
    /// Role names.
    pub roles: Vec<Role>,
    /// Issued-at Unix epoch seconds.
    pub iat: u64,
    /// Expiration Unix epoch seconds.
    pub exp: u64,
}

/// Verified session data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSession {
    /// User id.
    pub user_id: String,
    /// Username.
    pub username: String,
    /// Roles assigned to this session.
    pub roles: Vec<Role>,
}

/// JWT issue/verify helper.
#[derive(Clone)]
pub struct SessionManager {
    secret: Vec<u8>,
    ttl: Duration,
}

impl SessionManager {
    /// Create a session manager.
    pub fn new(secret: impl Into<Vec<u8>>, ttl: Duration) -> Self {
        Self {
            secret: secret.into(),
            ttl,
        }
    }

    /// Issue a signed JWT for a user.
    pub fn issue(
        &self,
        user_id: &str,
        username: &str,
        roles: &[Role],
    ) -> Result<String, SessionError> {
        let iat = now_epoch_seconds();
        let claims = SessionClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles: roles.to_vec(),
            iat,
            exp: iat.saturating_add(self.ttl.as_secs()),
        };
        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )?)
    }

    /// Verify a signed JWT and return its session.
    pub fn verify(&self, token: &str) -> Result<VerifiedSession, SessionError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        let data =
            decode::<SessionClaims>(token, &DecodingKey::from_secret(&self.secret), &validation)?;
        Ok(VerifiedSession {
            user_id: data.claims.sub,
            username: data.claims.username,
            roles: data.claims.roles,
        })
    }
}

/// Session error.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// JWT library error.
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
