use serde::{Deserialize, Serialize};

use crate::time_util::now_ms;

/// Priority of an executive directive or routed task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Who produced a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Agent,
    System,
    Tool,
}

/// Typed payload carried by a [`Message`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum MessageContent {
    /// Plain free-form text from a user or agent.
    Text(String),

    /// A structured shopping / product-research query.
    ShoppingQuery {
        query: String,
        location: Option<String>,
        language: Option<String>,
    },

    /// A high-level directive issued by the CEO agent.
    Directive {
        instruction: String,
        priority: Priority,
    },

    /// A task already classified and routed by the Secretary.
    RoutedTask {
        intent: String,
        original_query: String,
        target_agent: String,
    },

    /// Result of a tool call flowing back through the agent chain.
    ToolResult {
        tool_name: String,
        payload: serde_json::Value,
    },
}

/// A message flowing through the OpenFang runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (caller-supplied, e.g. a UUID string or counter).
    pub id: String,
    /// Sender identifier: `"user"`, an agent ID, or `"system"`.
    pub from: String,
    /// Recipient agent ID.
    pub to: String,
    /// Sender role.
    pub role: MessageRole,
    /// Typed payload.
    pub content: MessageContent,
    /// Milliseconds since Unix epoch (set automatically by constructors).
    pub timestamp_ms: i64,
}

impl Message {
    /// Create a plain text message from a user to a named agent.
    pub fn user_text(
        id: impl Into<String>,
        to: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            from: "user".into(),
            to: to.into(),
            role: MessageRole::User,
            content: MessageContent::Text(text.into()),
            timestamp_ms: now_ms(),
        }
    }

    /// Create a shopping query from a user to a named agent.
    pub fn shopping_query(
        id: impl Into<String>,
        to: impl Into<String>,
        query: impl Into<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            from: "user".into(),
            to: to.into(),
            role: MessageRole::User,
            content: MessageContent::ShoppingQuery {
                query: query.into(),
                location,
                language: None,
            },
            timestamp_ms: now_ms(),
        }
    }

    /// Create a CEO directive message.
    pub fn directive(
        id: impl Into<String>,
        to: impl Into<String>,
        instruction: impl Into<String>,
        priority: Priority,
    ) -> Self {
        Self {
            id: id.into(),
            from: "ceo".into(),
            to: to.into(),
            role: MessageRole::Agent,
            content: MessageContent::Directive {
                instruction: instruction.into(),
                priority,
            },
            timestamp_ms: now_ms(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_text_sets_correct_fields() {
        let msg = Message::user_text("msg-1", "agent-secretary", "buy oat milk");
        assert_eq!(msg.id, "msg-1");
        assert_eq!(msg.from, "user");
        assert_eq!(msg.to, "agent-secretary");
        assert_eq!(msg.role, MessageRole::User);
        assert!(matches!(msg.content, MessageContent::Text(_)));
    }

    #[test]
    fn shopping_query_sets_location() {
        let msg = Message::shopping_query(
            "q-1",
            "agent-shopping",
            "organic oat milk",
            Some("Geneva".into()),
        );
        match msg.content {
            MessageContent::ShoppingQuery {
                query, location, ..
            } => {
                assert_eq!(query, "organic oat milk");
                assert_eq!(location, Some("Geneva".into()));
            }
            _ => panic!("wrong content type"),
        }
    }

    #[test]
    fn message_roundtrips_through_json() {
        let msg = Message::user_text("m1", "agent-ceo", "prepare board report");
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "m1");
        assert_eq!(back.from, "user");
    }

    #[test]
    fn directive_message_sets_priority() {
        let msg = Message::directive("d-1", "agent-ceo", "expand to Paris", Priority::High);
        match msg.content {
            MessageContent::Directive { priority, .. } => {
                assert_eq!(priority, Priority::High);
            }
            _ => panic!("wrong content type"),
        }
    }
}
