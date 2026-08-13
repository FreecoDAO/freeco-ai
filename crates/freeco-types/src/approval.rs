//! Execution approval types for the Freeco agent OS.
//!
//! When an agent attempts a dangerous operation (e.g. `shell_exec`), the kernel
//! creates an [`ApprovalRequest`] and pauses the agent until a human operator
//! responds with an [`ApprovalResponse`]. The [`ApprovalPolicy`] configures
//! which tools require approval and how long to wait before auto-denying.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum length of tool names (chars).
const MAX_TOOL_NAME_LEN: usize = 64;

/// Maximum length of a request description (chars).
const MAX_DESCRIPTION_LEN: usize = 1024;

/// Maximum length of an action summary (chars).
const MAX_ACTION_SUMMARY_LEN: usize = 512;

/// Minimum approval timeout in seconds.
const MIN_TIMEOUT_SECS: u64 = 10;

/// Four hours. Long enough to survive a night's sleep or a working day away
/// from the desk, short enough that a forgotten request does not sit pending
/// for a week. Callers that genuinely want a fast auto-deny still set their
/// own shorter timeout.
const DEFAULT_TIMEOUT_SECS: u64 = 4 * 60 * 60;

/// Maximum approval timeout in seconds.
/// 24 hours. An approval request is a question put to a human, and humans
/// are asleep, in meetings, or away from the machine for hours at a time.
/// The old five-minute ceiling meant any request raised while the user was
/// not watching auto-denied itself, so a long agent run would fail on a
/// permission the user would happily have granted an hour later - and the
/// failure looked like a bug rather than a timeout.
const MAX_TIMEOUT_SECS: u64 = 24 * 60 * 60;

// ---------------------------------------------------------------------------
// RiskLevel
// ---------------------------------------------------------------------------

/// Risk level of an operation requiring approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// Returns a warning emoji suitable for display in dashboards and chat.
    pub fn emoji(&self) -> &'static str {
        match self {
            RiskLevel::Low => "\u{2139}\u{fe0f}",      // information source
            RiskLevel::Medium => "\u{26a0}\u{fe0f}",   // warning sign
            RiskLevel::High => "\u{1f6a8}",            // rotating light
            RiskLevel::Critical => "\u{2620}\u{fe0f}", // skull and crossbones
        }
    }
}

// ---------------------------------------------------------------------------
// ApprovalDecision
// ---------------------------------------------------------------------------

/// Decision on an approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approved,
    Denied,
    TimedOut,
}

// ---------------------------------------------------------------------------
// ApprovalRequest
// ---------------------------------------------------------------------------

/// What an agent must say when it asks permission.
///
/// Written because the old requests were unanswerable. A user was shown a
/// wall of shell with no statement of intent and no statement of cost, and
/// asked yes or no. Faced with that, people either approve everything or deny
/// everything; neither is a decision.
pub const APPROVAL_REQUEST_RULE: &str = "\
ASKING PERMISSION

When you need approval, write the request for the person reading it, not for
yourself. Three things, always:

1. The exact command or action, verbatim. Not a paraphrase.
2. Why you want it - the outcome you are after, in a sentence someone who does
   not read code can act on. \"Run git push\" is not a reason; \"publish the
   reviewed fix so CI can build it\" is.
3. What changes if it is allowed, whether it can be undone, and what you will
   do instead if it is denied.

If you cannot state the aim and the consequences, you do not understand the
action well enough to be asking for it yet.

Ask once and wait. Do not re-send the same request in different words, and do
not look for another route to the same effect - that turns a decision the user
was entitled to make into one you made for them.

If a request is denied, say what you will do instead and carry on with the
parts of the work that do not depend on it.";

/// An approval request for a dangerous agent operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub agent_id: String,
    pub tool_name: String,
    pub description: String,
    /// The specific action being requested (sanitized for display).
    pub action_summary: String,
    /// Why the agent wants this, in plain language.
    ///
    /// Required, and required to be a sentence rather than a restatement of
    /// the command. "Run git push" is not an aim; "publish the reviewed fix so
    /// CI can build it" is. Without this the user is asked to authorise a
    /// string of shell they have to reverse-engineer, which is how people end
    /// up approving reflexively.
    #[serde(default)]
    pub aim: String,
    /// What will change if this is allowed, and what happens if it is refused.
    ///
    /// Separate from `aim` on purpose: the reason to want something and the
    /// cost of granting it are different questions, and only the second tells
    /// the user what they are risking.
    #[serde(default)]
    pub consequences: String,
    /// Whether the effect can be undone, and how. `None` means the agent did
    /// not say, which the UI must present as "unknown", never as "safe".
    #[serde(default)]
    pub reversible: Option<bool>,
    pub risk_level: RiskLevel,
    pub requested_at: DateTime<Utc>,
    /// Auto-deny timeout in seconds.
    pub timeout_secs: u64,
}

/// Placeholder text that means an agent filled the field to satisfy the
/// validator rather than to inform anyone.
const NON_ANSWERS: &[&str] = &[
    "n/a",
    "na",
    "none",
    "-",
    "--",
    "tbd",
    "unknown",
    "see above",
    "as described",
    "to complete the task",
    "as requested",
    "required",
    "necessary",
];

fn is_real_answer(text: &str) -> bool {
    let t = text.trim().to_ascii_lowercase();
    // Short enough to be a shrug, or a known non-answer.
    t.len() >= 15 && !NON_ANSWERS.contains(&t.as_str())
}

impl ApprovalRequest {
    /// Render the request the way it must be shown to a person.
    ///
    /// One place, so every surface -- chat, dashboard, CLI -- asks the same
    /// question the same way. A user who sees a different shape in each place
    /// cannot build the habit of reading it.
    pub fn plain_language(&self) -> String {
        let reversible = match self.reversible {
            Some(true) => "Yes, this can be undone.",
            Some(false) => "No. This cannot be undone.",
            None => "Not stated by the agent - treat as if it cannot be undone.",
        };
        format!(
            "{agent} is asking to run:\n\n    {action}\n\n\
             Why: {aim}\n\
             If you allow it: {consequences}\n\
             Reversible: {reversible}\n\
             Risk: {risk:?}\n\n\
             You can allow this once, allow it always, or deny it. An \"always\" \
             can be withdrawn later from Settings > Approvals.",
            agent = self.agent_id,
            action = self.action_summary,
            aim = if self.aim.trim().is_empty() {
                "(the agent did not say - ask it before allowing)"
            } else {
                self.aim.trim()
            },
            consequences = if self.consequences.trim().is_empty() {
                "(the agent did not say - ask it before allowing)"
            } else {
                self.consequences.trim()
            },
            reversible = reversible,
            risk = self.risk_level,
        )
    }

    /// Whether this request is fit to show a human.
    ///
    /// Enforced rather than advised: an agent that can skip the explanation
    /// will skip it under time pressure, which is precisely when the user most
    /// needs it.
    pub fn explains_itself(&self) -> Result<(), String> {
        if !is_real_answer(&self.aim) {
            return Err(
                "aim must say why you want this, in a sentence a non-programmer                  can act on - not a restatement of the command"
                    .into(),
            );
        }
        if !is_real_answer(&self.consequences) {
            return Err(
                "consequences must say what changes if this is allowed, and                  whether it can be undone"
                    .into(),
            );
        }
        Ok(())
    }
}

impl ApprovalRequest {
    /// Validate this request's fields.
    ///
    /// Returns `Ok(())` or an error message describing the first validation failure.
    pub fn validate(&self) -> Result<(), String> {
        // -- tool_name --
        if self.tool_name.is_empty() {
            return Err("tool_name must not be empty".into());
        }
        if self.tool_name.len() > MAX_TOOL_NAME_LEN {
            return Err(format!(
                "tool_name too long ({} chars, max {MAX_TOOL_NAME_LEN})",
                self.tool_name.len()
            ));
        }
        if !self
            .tool_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(
                "tool_name may only contain alphanumeric characters and underscores".into(),
            );
        }

        // -- description --
        if self.description.len() > MAX_DESCRIPTION_LEN {
            return Err(format!(
                "description too long ({} chars, max {MAX_DESCRIPTION_LEN})",
                self.description.len()
            ));
        }

        // -- action_summary --
        if self.action_summary.len() > MAX_ACTION_SUMMARY_LEN {
            return Err(format!(
                "action_summary too long ({} chars, max {MAX_ACTION_SUMMARY_LEN})",
                self.action_summary.len()
            ));
        }

        // -- timeout_secs --
        if self.timeout_secs < MIN_TIMEOUT_SECS {
            return Err(format!(
                "timeout_secs too small ({}, min {MIN_TIMEOUT_SECS})",
                self.timeout_secs
            ));
        }
        if self.timeout_secs > MAX_TIMEOUT_SECS {
            return Err(format!(
                "timeout_secs too large ({}, max {MAX_TIMEOUT_SECS})",
                self.timeout_secs
            ));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ApprovalResponse
// ---------------------------------------------------------------------------

/// Response to an approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: Uuid,
    pub decision: ApprovalDecision,
    pub decided_at: DateTime<Utc>,
    pub decided_by: Option<String>,
}

// ---------------------------------------------------------------------------
// ApprovalPolicy
// ---------------------------------------------------------------------------

/// Configurable approval policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApprovalPolicy {
    /// Tools that always require approval. Default: `["shell_exec"]`.
    ///
    /// Accepts either a list of tool names or a boolean shorthand:
    /// - `require_approval = false` → empty list (no tools require approval)
    /// - `require_approval = true`  → `["shell_exec"]` (the default set)
    #[serde(deserialize_with = "deserialize_require_approval")]
    pub require_approval: Vec<String>,
    /// Timeout in seconds before the request auto-denies.
    /// Default: 4 hours, range: 10 seconds..=24 hours.
    pub timeout_secs: u64,
    /// Auto-approve in autonomous mode. Default: `false`.
    pub auto_approve_autonomous: bool,
    /// Alias: if `auto_approve = true`, clears the require list at boot.
    #[serde(default, alias = "auto_approve")]
    pub auto_approve: bool,
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self {
            require_approval: vec!["shell_exec".to_string()],
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            auto_approve_autonomous: false,
            auto_approve: false,
        }
    }
}

/// Custom deserializer that accepts:
/// - A list of strings: `["shell_exec", "file_write"]`
/// - A boolean: `false` → `[]`, `true` → `["shell_exec"]`
fn deserialize_require_approval<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct RequireApprovalVisitor;

    impl<'de> de::Visitor<'de> for RequireApprovalVisitor {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a list of tool names or a boolean")
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(if v {
                vec!["shell_exec".to_string()]
            } else {
                vec![]
            })
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                v.push(s);
            }
            Ok(v)
        }
    }

    deserializer.deserialize_any(RequireApprovalVisitor)
}

impl ApprovalPolicy {
    /// Apply the `auto_approve` shorthand: if true, clears the require list.
    pub fn apply_shorthands(&mut self) {
        if self.auto_approve {
            self.require_approval.clear();
        }
    }

    /// Validate this policy's fields.
    ///
    /// Returns `Ok(())` or an error message describing the first validation failure.
    pub fn validate(&self) -> Result<(), String> {
        // -- timeout_secs --
        if self.timeout_secs < MIN_TIMEOUT_SECS {
            return Err(format!(
                "timeout_secs too small ({}, min {MIN_TIMEOUT_SECS})",
                self.timeout_secs
            ));
        }
        if self.timeout_secs > MAX_TIMEOUT_SECS {
            return Err(format!(
                "timeout_secs too large ({}, max {MAX_TIMEOUT_SECS})",
                self.timeout_secs
            ));
        }

        // -- require_approval tool names --
        for (i, name) in self.require_approval.iter().enumerate() {
            if name.is_empty() {
                return Err(format!("require_approval[{i}] must not be empty"));
            }
            if name.len() > MAX_TOOL_NAME_LEN {
                return Err(format!(
                    "require_approval[{i}] too long ({} chars, max {MAX_TOOL_NAME_LEN})",
                    name.len()
                ));
            }
            if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(format!(
                    "require_approval[{i}] may only contain alphanumeric characters and underscores: \"{name}\""
                ));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers --

    fn valid_request() -> ApprovalRequest {
        ApprovalRequest {
            id: Uuid::new_v4(),
            agent_id: "agent-001".into(),
            tool_name: "shell_exec".into(),
            description: "Execute rm -rf /tmp/stale_cache".into(),
            action_summary: "rm -rf /tmp/stale_cache".into(),
            aim: "free disk space by clearing the stale build cache".into(),
            consequences: "the cache directory is deleted and will be rebuilt on next use".into(),
            reversible: Some(true),
            risk_level: RiskLevel::High,
            requested_at: Utc::now(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    fn valid_policy() -> ApprovalPolicy {
        ApprovalPolicy::default()
    }

    // -----------------------------------------------------------------------
    // RiskLevel
    // -----------------------------------------------------------------------

    #[test]
    fn risk_level_emoji() {
        assert_eq!(RiskLevel::Low.emoji(), "\u{2139}\u{fe0f}");
        assert_eq!(RiskLevel::Medium.emoji(), "\u{26a0}\u{fe0f}");
        assert_eq!(RiskLevel::High.emoji(), "\u{1f6a8}");
        assert_eq!(RiskLevel::Critical.emoji(), "\u{2620}\u{fe0f}");
    }

    #[test]
    fn risk_level_serde_roundtrip() {
        for level in [
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let back: RiskLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, back);
        }
    }

    #[test]
    fn risk_level_rename_all() {
        let json = serde_json::to_string(&RiskLevel::Critical).unwrap();
        assert_eq!(json, "\"critical\"");
        let json = serde_json::to_string(&RiskLevel::Low).unwrap();
        assert_eq!(json, "\"low\"");
    }

    // -----------------------------------------------------------------------
    // ApprovalDecision
    // -----------------------------------------------------------------------

    #[test]
    fn decision_serde_roundtrip() {
        for decision in [
            ApprovalDecision::Approved,
            ApprovalDecision::Denied,
            ApprovalDecision::TimedOut,
        ] {
            let json = serde_json::to_string(&decision).unwrap();
            let back: ApprovalDecision = serde_json::from_str(&json).unwrap();
            assert_eq!(decision, back);
        }
    }

    #[test]
    fn decision_rename_all() {
        let json = serde_json::to_string(&ApprovalDecision::TimedOut).unwrap();
        assert_eq!(json, "\"timed_out\"");
    }

    // -----------------------------------------------------------------------
    // ApprovalRequest — valid
    // -----------------------------------------------------------------------

    #[test]
    fn valid_request_passes() {
        assert!(valid_request().validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // ApprovalRequest — tool_name
    // -----------------------------------------------------------------------

    #[test]
    fn request_empty_tool_name() {
        let mut req = valid_request();
        req.tool_name = String::new();
        let err = req.validate().unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    #[test]
    fn request_tool_name_too_long() {
        let mut req = valid_request();
        req.tool_name = "a".repeat(65);
        let err = req.validate().unwrap_err();
        assert!(err.contains("too long"), "{err}");
    }

    #[test]
    fn request_tool_name_64_chars_ok() {
        let mut req = valid_request();
        req.tool_name = "a".repeat(64);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn request_tool_name_invalid_chars() {
        let mut req = valid_request();
        req.tool_name = "shell-exec".into();
        let err = req.validate().unwrap_err();
        assert!(err.contains("alphanumeric"), "{err}");
    }

    #[test]
    fn request_tool_name_with_underscore_ok() {
        let mut req = valid_request();
        req.tool_name = "file_write".into();
        assert!(req.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // ApprovalRequest — description
    // -----------------------------------------------------------------------

    #[test]
    fn request_description_too_long() {
        let mut req = valid_request();
        req.description = "x".repeat(1025);
        let err = req.validate().unwrap_err();
        assert!(err.contains("description"), "{err}");
        assert!(err.contains("too long"), "{err}");
    }

    #[test]
    fn request_description_1024_ok() {
        let mut req = valid_request();
        req.description = "x".repeat(1024);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn request_description_empty_ok() {
        let mut req = valid_request();
        req.description = String::new();
        assert!(req.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // ApprovalRequest — action_summary
    // -----------------------------------------------------------------------

    #[test]
    fn request_action_summary_too_long() {
        let mut req = valid_request();
        req.action_summary = "x".repeat(513);
        let err = req.validate().unwrap_err();
        assert!(err.contains("action_summary"), "{err}");
        assert!(err.contains("too long"), "{err}");
    }

    #[test]
    fn request_action_summary_512_ok() {
        let mut req = valid_request();
        req.action_summary = "x".repeat(512);
        assert!(req.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // ApprovalRequest — timeout_secs
    // -----------------------------------------------------------------------

    #[test]
    fn request_timeout_too_small() {
        let mut req = valid_request();
        req.timeout_secs = 9;
        let err = req.validate().unwrap_err();
        assert!(err.contains("too small"), "{err}");
    }

    #[test]
    fn request_timeout_too_large() {
        let mut req = valid_request();
        // Just past the 24-hour ceiling.
        req.timeout_secs = MAX_TIMEOUT_SECS + 1;
        let err = req.validate().unwrap_err();
        assert!(err.contains("too large"), "{err}");
    }

    #[test]
    fn request_timeout_min_boundary_ok() {
        let mut req = valid_request();
        req.timeout_secs = 10;
        assert!(req.validate().is_ok());
    }

    #[test]
    fn request_timeout_max_boundary_ok() {
        let mut req = valid_request();
        req.timeout_secs = 300;
        assert!(req.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // ApprovalResponse — serde
    // -----------------------------------------------------------------------

    #[test]
    fn response_serde_roundtrip() {
        let resp = ApprovalResponse {
            request_id: Uuid::new_v4(),
            decision: ApprovalDecision::Approved,
            decided_at: Utc::now(),
            decided_by: Some("admin@example.com".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: ApprovalResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.request_id, resp.request_id);
        assert_eq!(back.decision, ApprovalDecision::Approved);
        assert_eq!(back.decided_by, Some("admin@example.com".into()));
    }

    #[test]
    fn response_decided_by_none() {
        let resp = ApprovalResponse {
            request_id: Uuid::new_v4(),
            decision: ApprovalDecision::TimedOut,
            decided_at: Utc::now(),
            decided_by: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: ApprovalResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.decided_by, None);
        assert_eq!(back.decision, ApprovalDecision::TimedOut);
    }

    // -----------------------------------------------------------------------
    // ApprovalPolicy — defaults
    // -----------------------------------------------------------------------

    #[test]
    fn policy_default_valid() {
        let policy = ApprovalPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.require_approval, vec!["shell_exec".to_string()]);
        assert_eq!(policy.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert!(!policy.auto_approve_autonomous);
        assert!(!policy.auto_approve);
    }

    #[test]
    fn policy_serde_default() {
        // An empty JSON object should deserialize to defaults via #[serde(default)].
        let policy: ApprovalPolicy = serde_json::from_str("{}").unwrap();
        assert_eq!(policy.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(policy.require_approval, vec!["shell_exec".to_string()]);
        assert!(!policy.auto_approve_autonomous);
    }

    #[test]
    fn policy_require_approval_bool_false() {
        // require_approval = false → empty list
        let policy: ApprovalPolicy =
            serde_json::from_str(r#"{"require_approval": false}"#).unwrap();
        assert!(policy.require_approval.is_empty());
    }

    #[test]
    fn policy_require_approval_bool_true() {
        // require_approval = true → ["shell_exec"]
        let policy: ApprovalPolicy = serde_json::from_str(r#"{"require_approval": true}"#).unwrap();
        assert_eq!(policy.require_approval, vec!["shell_exec"]);
    }

    #[test]
    fn policy_auto_approve_clears_list() {
        let mut policy = ApprovalPolicy::default();
        assert!(!policy.require_approval.is_empty());
        policy.auto_approve = true;
        policy.apply_shorthands();
        assert!(policy.require_approval.is_empty());
    }

    // -----------------------------------------------------------------------
    // ApprovalPolicy — timeout_secs
    // -----------------------------------------------------------------------

    #[test]
    fn policy_timeout_too_small() {
        let mut policy = valid_policy();
        policy.timeout_secs = 9;
        let err = policy.validate().unwrap_err();
        assert!(err.contains("too small"), "{err}");
    }

    #[test]
    fn policy_timeout_too_large() {
        let mut policy = valid_policy();
        // Just past the 24-hour ceiling.
        policy.timeout_secs = MAX_TIMEOUT_SECS + 1;
        let err = policy.validate().unwrap_err();
        assert!(err.contains("too large"), "{err}");
    }

    #[test]
    fn policy_timeout_boundaries_ok() {
        let mut policy = valid_policy();
        policy.timeout_secs = 10;
        assert!(policy.validate().is_ok());
        policy.timeout_secs = 300;
        assert!(policy.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // ApprovalPolicy — require_approval tool names
    // -----------------------------------------------------------------------

    #[test]
    fn policy_empty_tool_name() {
        let mut policy = valid_policy();
        policy.require_approval = vec!["shell_exec".into(), "".into()];
        let err = policy.validate().unwrap_err();
        assert!(err.contains("require_approval[1]"), "{err}");
        assert!(err.contains("empty"), "{err}");
    }

    #[test]
    fn policy_tool_name_too_long() {
        let mut policy = valid_policy();
        policy.require_approval = vec!["a".repeat(65)];
        let err = policy.validate().unwrap_err();
        assert!(err.contains("too long"), "{err}");
    }

    #[test]
    fn policy_tool_name_invalid_chars() {
        let mut policy = valid_policy();
        policy.require_approval = vec!["shell-exec".into()];
        let err = policy.validate().unwrap_err();
        assert!(err.contains("alphanumeric"), "{err}");
    }

    #[test]
    fn policy_tool_name_with_spaces_rejected() {
        let mut policy = valid_policy();
        policy.require_approval = vec!["shell exec".into()];
        let err = policy.validate().unwrap_err();
        assert!(err.contains("alphanumeric"), "{err}");
    }

    #[test]
    fn policy_multiple_valid_tools() {
        let mut policy = valid_policy();
        policy.require_approval = vec![
            "shell_exec".into(),
            "file_write".into(),
            "file_delete".into(),
        ];
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn policy_empty_require_approval_ok() {
        let mut policy = valid_policy();
        policy.require_approval = vec![];
        assert!(policy.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // Full serde roundtrip — ApprovalRequest
    // -----------------------------------------------------------------------

    #[test]
    fn request_serde_roundtrip() {
        let req = valid_request();
        let json = serde_json::to_string_pretty(&req).unwrap();
        let back: ApprovalRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, req.id);
        assert_eq!(back.agent_id, req.agent_id);
        assert_eq!(back.tool_name, req.tool_name);
        assert_eq!(back.description, req.description);
        assert_eq!(back.action_summary, req.action_summary);
        assert_eq!(back.risk_level, req.risk_level);
        assert_eq!(back.timeout_secs, req.timeout_secs);
    }

    // -----------------------------------------------------------------------
    // Full serde roundtrip — ApprovalPolicy
    // -----------------------------------------------------------------------

    #[test]
    fn policy_serde_roundtrip() {
        let policy = ApprovalPolicy {
            require_approval: vec!["shell_exec".into(), "file_delete".into()],
            timeout_secs: 120,
            auto_approve_autonomous: true,
            auto_approve: false,
        };
        let json = serde_json::to_string(&policy).unwrap();
        let back: ApprovalPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back.require_approval, policy.require_approval);
        assert_eq!(back.timeout_secs, 120);
        assert!(back.auto_approve_autonomous);
    }
}

#[cfg(test)]
mod plain_language_tests {
    use super::*;

    fn req(aim: &str, consequences: &str, reversible: Option<bool>) -> ApprovalRequest {
        ApprovalRequest {
            id: Uuid::new_v4(),
            agent_id: "Developer".into(),
            tool_name: "shell_exec".into(),
            description: String::new(),
            action_summary: "git push origin main".into(),
            aim: aim.into(),
            consequences: consequences.into(),
            reversible,
            risk_level: RiskLevel::High,
            requested_at: Utc::now(),
            timeout_secs: 3600,
        }
    }

    /// The rendered request must answer the three questions a person needs:
    /// what exactly, why, and what it costs.
    #[test]
    fn rendered_request_answers_what_why_and_cost() {
        let text = req(
            "publish the reviewed fix so CI can build it",
            "the commit becomes public history on the main branch",
            Some(false),
        )
        .plain_language();

        assert!(
            text.contains("git push origin main"),
            "must show the exact action"
        );
        assert!(
            text.contains("publish the reviewed fix"),
            "must show the aim"
        );
        assert!(
            text.contains("becomes public history"),
            "must show the cost"
        );
        assert!(text.contains("cannot be undone"));
        assert!(
            text.contains("withdrawn later"),
            "must say an always can be revoked"
        );
    }

    /// Unstated reversibility must read as dangerous, never as safe. Silence
    /// is the most common case and the easiest to misread.
    #[test]
    fn unknown_reversibility_is_not_presented_as_safe() {
        let text = req("do the thing properly", "some files are written", None).plain_language();
        assert!(text.contains("treat as if it cannot be undone"));
        assert!(!text.contains("Yes, this can be undone"));
    }

    /// A missing explanation must be visible in the prompt itself, so the user
    /// sees the gap instead of a blank they might read past.
    #[test]
    fn missing_explanation_is_shown_to_the_user() {
        let text = req("", "", None).plain_language();
        assert_eq!(text.matches("the agent did not say").count(), 2);
    }

    /// Boilerplate must not pass. An agent that can type "n/a" to clear the
    /// check has removed the protection while appearing to satisfy it.
    #[test]
    fn boilerplate_is_rejected() {
        for filler in ["n/a", "none", "TBD", "as requested", "to complete the task"] {
            assert!(
                req(
                    filler,
                    "the files are written and cannot be recovered",
                    None
                )
                .explains_itself()
                .is_err(),
                "{filler} should not count as an aim"
            );
        }
        assert!(
            req("x", "y", None).explains_itself().is_err(),
            "too short to inform"
        );
    }

    #[test]
    fn a_real_explanation_passes() {
        assert!(req(
            "publish the reviewed fix so CI can build it",
            "the commit becomes public history and cannot be unpublished",
            Some(false),
        )
        .explains_itself()
        .is_ok());
    }

    /// The rule has to tell agents what to do instead of retrying or routing
    /// around, or it only stops the most literal repetition.
    #[test]
    fn the_rule_forbids_retrying_and_rerouting() {
        let r = APPROVAL_REQUEST_RULE;
        assert!(r.contains("verbatim"));
        assert!(r.contains("can be undone"));
        assert!(r.contains("Ask once and wait"));
        assert!(r.contains("another route"));
        // Matched on a fragment that cannot straddle the line wrap: the rule
        // text is hard-wrapped, so a longer phrase spans a newline and would
        // fail for a formatting reason rather than a real one.
        assert!(r.contains("carry on with the"));
        assert!(r.contains("do not depend on it"));
    }
}
