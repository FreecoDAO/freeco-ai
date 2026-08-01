//! # OpenFang AI Data Protective Gateway
//!
//! Security Layer #17 — A data-content-aware gateway that sits between
//! agent prompts and LLM provider calls. Inspects all AI traffic for
//! PII and sensitive data, masks it with reversible tokens before it
//! reaches external providers, and can de-mask tokens in responses.
//!
//! ## Architecture
//!
//! ```text
//! Agent → [DataGateway::process_outbound] → LLM Provider
//!            │
//!       ┌────┴────┐
//!       │ Detect   │  PII detection (regex + pattern matching)
//!       │ Mask     │  Replace with reversible AES-GCM tokens
//!       │ Policy   │  Allow / Mask / Block per data type
//!       │ Audit    │  Log all actions
//!       └─────────┘
//!
//! Agent ← [DataGateway::process_inbound] ← LLM Response
//!            │
//!       ┌────┴────┐
//!       │ Inspect  │  Scan response for leaked secrets
//!       │ De-mask  │  Restore original values from token store
//!       │ Filter   │  Block responses that leak sensitive data
//!       └─────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use openfang_datagateway::{DataGateway, GatewayConfig, PolicyAction};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = GatewayConfig::default();
//! let mut gateway = DataGateway::new(config);
//!
//! // Outbound: mask PII before sending to LLM
//! let result = gateway.process_outbound(
//!     "My email is john@example.com and my card is 4532-1234-5678-9123"
//! )?;
//! assert!(result.processed_text.contains("[MASKED:email:"));
//! assert!(!result.processed_text.contains("john@example.com"));
//!
//! // Inbound: de-mask tokens in LLM response
//! let restored = gateway.process_inbound(
//!     "I see your email is [MASKED:email:abc123]"
//! )?;
//! assert!(restored.processed_text.contains("john@example.com"));
//! # Ok(())
//! # }
//! ```

pub mod audit;
pub mod config;
pub mod detector;
pub mod gateway;
pub mod masker;
pub mod policy;

pub use audit::{AuditAction, AuditEntry, AuditLog};
pub use config::GatewayConfig;
pub use detector::{DataType, Detection, PiiDetector};
pub use gateway::{DataGateway, GatewayResult};
pub use masker::{MaskToken, Masker};
pub use policy::{Policy, PolicyAction, PolicyEngine};

/// Re-export common types for convenience.
pub mod prelude {
    pub use crate::audit::{AuditAction, AuditEntry, AuditLog};
    pub use crate::config::GatewayConfig;
    pub use crate::detector::{DataType, Detection, PiiDetector};
    pub use crate::gateway::{DataGateway, GatewayResult};
    pub use crate::masker::{MaskToken, Masker};
    pub use crate::policy::{Policy, PolicyAction, PolicyEngine};
}
