//! # FreEco.ai data gateway
//!
//! Sits between an agent's prompt and the provider call. Finds sensitive values,
//! replaces them with reversible tokens before the text leaves the machine, and
//! restores them in the reply so the agent still sees real data.
//!
//! ## Why tokens map to a table rather than to ciphertext
//!
//! The obvious design encrypts each value and embeds the ciphertext in the
//! token. It needs a key, and that key has to live somewhere on the same
//! machine as the data it protects — so it buys nothing over storing the
//! mapping directly, while adding a key to lose. An earlier sketch of this
//! crate generated the key in its constructor, which meant every restart
//! silently orphaned every token it had ever issued: masked conversations
//! would come back permanently unreadable, and nothing would have reported it.
//!
//! So a token here is a random opaque string and the mapping is a row in the
//! same SQLite database the rest of the product already relies on. Tokens carry
//! no information, restoring is a primary-key lookup, and there is no key
//! management to get wrong. The security boundary is the database file, which
//! is the boundary anyway.
//!
//! ## What it does not do
//!
//! It does not stop an agent choosing to send something sensitive that it
//! composed itself, and it does not inspect binary attachments. It is a filter
//! on text, not a guarantee, and calling it a guarantee would be the more
//! dangerous error.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod detect;
pub use detect::{DataKind, Finding};

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("token store: {0}")]
    Store(String),
    #[error("blocked: {0}")]
    Blocked(String),
}

pub type Result<T> = std::result::Result<T, GatewayError>;

/// What to do with a kind of sensitive data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Let it through untouched.
    Allow,
    /// Replace with a reversible token.
    Mask,
    /// Refuse to send the request at all.
    Block,
}

/// Per-kind policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    rules: HashMap<String, Action>,
    default: Action,
}

impl Default for Policy {
    /// Masks by default, blocks what has no legitimate reason to reach a model.
    ///
    /// Private keys and card numbers are blocked rather than masked because
    /// there is no prompt that is improved by containing them, and a mask still
    /// means the value was written into a request body on its way out.
    /// Addresses and emails are masked, not blocked, because ordinary work
    /// legitimately involves them and blocking would make the product refuse
    /// normal tasks.
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert(DataKind::PrivateKey.as_str().to_string(), Action::Block);
        rules.insert(DataKind::CreditCard.as_str().to_string(), Action::Block);
        rules.insert(DataKind::ApiKey.as_str().to_string(), Action::Mask);
        rules.insert(DataKind::Iban.as_str().to_string(), Action::Mask);
        rules.insert(DataKind::Email.as_str().to_string(), Action::Mask);
        rules.insert(DataKind::IpAddress.as_str().to_string(), Action::Mask);
        Self {
            rules,
            default: Action::Mask,
        }
    }
}

impl Policy {
    pub fn action_for(&self, kind: DataKind) -> Action {
        self.rules
            .get(kind.as_str())
            .copied()
            .unwrap_or(self.default)
    }

    pub fn set(&mut self, kind: DataKind, action: Action) {
        self.rules.insert(kind.as_str().to_string(), action);
    }
}

/// What happened to one outbound request, for the audit log and the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub masked: usize,
    pub allowed: usize,
    /// Kinds seen, so the user can be told *what* was protected.
    pub kinds: Vec<String>,
}

/// The gateway.
#[derive(Clone)]
pub struct DataGateway {
    conn: Arc<Mutex<Connection>>,
    policy: Policy,
}

impl DataGateway {
    /// Open a gateway over an existing database connection.
    pub fn new(conn: Arc<Mutex<Connection>>, policy: Policy) -> Result<Self> {
        {
            let c = conn.lock().map_err(|e| GatewayError::Store(e.to_string()))?;
            c.execute_batch(
                "CREATE TABLE IF NOT EXISTS gateway_tokens (
                     token      TEXT PRIMARY KEY,
                     value      TEXT NOT NULL,
                     kind       TEXT NOT NULL,
                     created_at TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS gateway_tokens_value
                     ON gateway_tokens(value);",
            )
            .map_err(|e| GatewayError::Store(e.to_string()))?;
        }
        Ok(Self { conn, policy })
    }

    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Mask sensitive values on their way to a provider.
    ///
    /// Returns the text to send and a report of what was done. Blocking is an
    /// error rather than a silent redaction: if the user asked to send a
    /// private key, they need to be told it was refused, not left wondering why
    /// the answer is wrong.
    pub fn process_outbound(&self, text: &str) -> Result<(String, Report)> {
        let findings = detect::scan(text);
        if findings.is_empty() {
            return Ok((
                text.to_string(),
                Report {
                    masked: 0,
                    allowed: 0,
                    kinds: Vec::new(),
                },
            ));
        }

        for f in &findings {
            if self.policy.action_for(f.kind) == Action::Block {
                return Err(GatewayError::Blocked(format!(
                    "{} found in the request. Sending it to a model provider is \
                     not allowed by policy, so the request was not sent.",
                    f.kind.as_str()
                )));
            }
        }

        let mut out = String::with_capacity(text.len());
        let mut cursor = 0usize;
        let mut masked = 0usize;
        let mut allowed = 0usize;
        let mut kinds: Vec<String> = Vec::new();

        for f in &findings {
            out.push_str(&text[cursor..f.start]);
            match self.policy.action_for(f.kind) {
                Action::Allow => {
                    out.push_str(&f.value);
                    allowed += 1;
                }
                Action::Mask => {
                    out.push_str(&self.token_for(&f.value, f.kind)?);
                    masked += 1;
                    if !kinds.iter().any(|k| k == f.kind.as_str()) {
                        kinds.push(f.kind.as_str().to_string());
                    }
                }
                Action::Block => unreachable!("blocked above"),
            }
            cursor = f.end;
        }
        out.push_str(&text[cursor..]);

        Ok((
            out,
            Report {
                masked,
                allowed,
                kinds,
            },
        ))
    }

    /// Restore masked values in a provider's reply.
    ///
    /// Models echo their input, so a reply routinely contains the tokens that
    /// went out. Without this the user reads their own data back as gibberish.
    pub fn process_inbound(&self, text: &str) -> Result<String> {
        if !text.contains(TOKEN_PREFIX) {
            return Ok(text.to_string());
        }
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(pos) = rest.find(TOKEN_PREFIX) {
            out.push_str(&rest[..pos]);
            let after = &rest[pos..];
            let end = after
                .char_indices()
                .find(|(i, c)| *i > 0 && !(c.is_ascii_alphanumeric() || *c == '_'))
                .map(|(i, _)| i)
                .unwrap_or(after.len());
            let token = &after[..end];
            match self.lookup(token)? {
                Some(value) => out.push_str(&value),
                // An unknown token is left as-is rather than blanked: it may be
                // ordinary text that happens to look like one, and destroying
                // it would be a worse failure than showing it.
                None => out.push_str(token),
            }
            rest = &after[end..];
        }
        out.push_str(rest);
        Ok(out)
    }

    /// Reuse the token already issued for a value, so the same secret masks
    /// consistently within and across conversations — a model that sees one
    /// value under two names is being told they are two different things.
    fn token_for(&self, value: &str, kind: DataKind) -> Result<String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GatewayError::Store(e.to_string()))?;
        let existing: Option<String> = conn
            .query_row(
                "SELECT token FROM gateway_tokens WHERE value = ?1 AND kind = ?2",
                rusqlite::params![value, kind.as_str()],
                |r| r.get(0),
            )
            .ok();
        if let Some(t) = existing {
            return Ok(t);
        }
        let token = new_token();
        conn.execute(
            "INSERT INTO gateway_tokens (token, value, kind, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                token,
                value,
                kind.as_str(),
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| GatewayError::Store(e.to_string()))?;
        Ok(token)
    }

    fn lookup(&self, token: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GatewayError::Store(e.to_string()))?;
        Ok(conn
            .query_row(
                "SELECT value FROM gateway_tokens WHERE token = ?1",
                [token],
                |r| r.get(0),
            )
            .ok())
    }
}

/// Token marker. Chosen to be unmistakable in a prompt and stable under the
/// tokenisers models use, so a mask is never split across two tokens.
const TOKEN_PREFIX: &str = "FRECO_REDACTED_";

fn new_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let n: u128 = rng.random();
    format!("{TOKEN_PREFIX}{n:032x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gw() -> DataGateway {
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        DataGateway::new(conn, Policy::default()).unwrap()
    }

    #[test]
    fn masks_a_key_and_restores_it() {
        let g = gw();
        let original = "my key is sk-or-v1-abcdef0123456789abcdef ok";
        let (sent, report) = g.process_outbound(original).unwrap();
        assert!(!sent.contains("sk-or-v1-"), "key still present: {sent}");
        assert_eq!(report.masked, 1);
        // The model echoes the token back; the user must see their own value.
        assert_eq!(g.process_inbound(&sent).unwrap(), original);
    }

    /// The same secret must always mask to the same token. Two tokens for one
    /// value tells the model it is looking at two different things.
    #[test]
    fn one_value_gets_one_stable_token() {
        let g = gw();
        let (a, _) = g.process_outbound("k sk-ant-0123456789abcdefgh").unwrap();
        let (b, _) = g.process_outbound("again sk-ant-0123456789abcdefgh").unwrap();
        let ta = a.split_whitespace().last().unwrap();
        let tb = b.split_whitespace().last().unwrap();
        assert_eq!(ta, tb);
    }

    #[test]
    fn blocks_what_has_no_business_leaving() {
        let g = gw();
        let err = g.process_outbound("card 4242 4242 4242 4242").unwrap_err();
        assert!(matches!(err, GatewayError::Blocked(_)));
    }

    #[test]
    fn clean_text_is_untouched_and_costs_nothing() {
        let g = gw();
        let t = "Summarise the attached report and list three risks.";
        let (sent, report) = g.process_outbound(t).unwrap();
        assert_eq!(sent, t);
        assert_eq!(report.masked, 0);
    }

    /// The failure that made the original sketch unusable: a key held in the
    /// process meant every restart orphaned every token. Tokens must survive a
    /// gateway being dropped and rebuilt over the same database.
    #[test]
    fn tokens_survive_a_restart() {
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let sent = {
            let g = DataGateway::new(conn.clone(), Policy::default()).unwrap();
            g.process_outbound("key sk-proj-abcdef0123456789abcd").unwrap().0
        };
        let g2 = DataGateway::new(conn, Policy::default()).unwrap();
        assert_eq!(
            g2.process_inbound(&sent).unwrap(),
            "key sk-proj-abcdef0123456789abcd"
        );
    }

    #[test]
    fn policy_can_allow_a_kind() {
        let mut p = Policy::default();
        p.set(DataKind::Email, Action::Allow);
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let g = DataGateway::new(conn, p).unwrap();
        let (sent, r) = g.process_outbound("mail a.b@example.com").unwrap();
        assert!(sent.contains("a.b@example.com"));
        assert_eq!(r.allowed, 1);
    }

    #[test]
    fn unknown_tokens_are_left_alone_not_destroyed() {
        let g = gw();
        let t = "FRECO_REDACTED_deadbeef stays";
        assert_eq!(g.process_inbound(t).unwrap(), t);
    }
}
