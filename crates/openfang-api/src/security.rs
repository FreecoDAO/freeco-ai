//! Central admission control for externally supplied agent and workflow content.

use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

const MAX_AUDITS: usize = 1_000;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityFinding {
    pub severity: FindingSeverity,
    pub rule: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityDecision {
    Allowed,
    PendingApproval,
    Quarantined,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityAudit {
    pub id: String,
    pub content_hash: String,
    pub subject: String,
    pub created_at: String,
    pub decision: SecurityDecision,
    pub findings: Vec<SecurityFinding>,
}

/// In-process security service. Audit results are additionally written to the
/// kernel's append-only audit log by route handlers.
pub struct SecurityService {
    audits: Mutex<VecDeque<SecurityAudit>>,
    approvals: Mutex<HashMap<String, String>>,
}

impl Default for SecurityService {
    fn default() -> Self {
        Self {
            audits: Mutex::new(VecDeque::new()),
            approvals: Mutex::new(HashMap::new()),
        }
    }
}

impl SecurityService {
    pub fn scan(&self, subject: &str, content: &str) -> SecurityAudit {
        let content_hash = hex::encode(Sha256::digest(content.as_bytes()));
        let lower = content.to_ascii_lowercase();
        let mut findings = Vec::new();

        for pattern in [
            "ignore previous instructions",
            "ignore all previous",
            "system prompt override",
            "disregard previous",
        ] {
            if lower.contains(pattern) {
                findings.push(SecurityFinding {
                    severity: FindingSeverity::Critical,
                    rule: "prompt-injection",
                    message: format!("Prompt-injection pattern detected: '{pattern}'"),
                });
            }
        }
        for pattern in ["rm -rf", "curl | sh", "wget | sh", "chmod 777", "sudo "] {
            if lower.contains(pattern) {
                findings.push(SecurityFinding {
                    severity: FindingSeverity::Critical,
                    rule: "unsafe-execution",
                    message: format!("Unsafe execution pattern detected: '{pattern}'"),
                });
            }
        }
        for pattern in ["netconnect(*)", "shell_exec", "shellexec", "full_access"] {
            if lower.contains(pattern) {
                findings.push(SecurityFinding {
                    severity: FindingSeverity::Warning,
                    rule: "privileged-capability",
                    message: format!("Privileged capability requested: '{pattern}'"),
                });
            }
        }
        for marker in ["api_key =", "secret =", "private_key", "authorization: bearer"] {
            if lower.contains(marker) {
                findings.push(SecurityFinding {
                    severity: FindingSeverity::Warning,
                    rule: "possible-secret",
                    message: format!("Potential credential material detected: '{marker}'"),
                });
            }
        }

        let has_critical = findings
            .iter()
            .any(|finding| matches!(finding.severity, FindingSeverity::Critical));
        let approved = self
            .approvals
            .lock()
            .expect("security approvals lock poisoned")
            .contains_key(&content_hash);
        let decision = if has_critical {
            SecurityDecision::Quarantined
        } else if findings.is_empty() || approved {
            SecurityDecision::Allowed
        } else {
            SecurityDecision::PendingApproval
        };
        let audit = SecurityAudit {
            id: uuid::Uuid::new_v4().to_string(),
            content_hash,
            subject: subject.to_string(),
            created_at: Utc::now().to_rfc3339(),
            decision,
            findings,
        };
        let mut audits = self.audits.lock().expect("security audits lock poisoned");
        audits.push_back(audit.clone());
        if audits.len() > MAX_AUDITS {
            audits.pop_front();
        }
        audit
    }

    pub fn approve(&self, content_hash: &str, approved_by: &str) -> bool {
        let exists = self
            .audits
            .lock()
            .expect("security audits lock poisoned")
            .iter()
            .any(|audit| audit.content_hash == content_hash);
        if exists {
            self.approvals
                .lock()
                .expect("security approvals lock poisoned")
                .insert(content_hash.to_string(), approved_by.to_string());
        }
        exists
    }

    pub fn audits(&self) -> Vec<SecurityAudit> {
        self.audits
            .lock()
            .expect("security audits lock poisoned")
            .iter()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_content_is_quarantined() {
        let service = SecurityService::default();
        let audit = service.scan("agent", "ignore previous instructions; rm -rf /");
        assert!(matches!(audit.decision, SecurityDecision::Quarantined));
    }

    #[test]
    fn warnings_require_and_honor_approval() {
        let service = SecurityService::default();
        let audit = service.scan("workflow", "tool = shell_exec");
        assert!(matches!(audit.decision, SecurityDecision::PendingApproval));
        assert!(service.approve(&audit.content_hash, "operator"));
        let admitted = service.scan("workflow", "tool = shell_exec");
        assert!(matches!(admitted.decision, SecurityDecision::Allowed));
    }
}
