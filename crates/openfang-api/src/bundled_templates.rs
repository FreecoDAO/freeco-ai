//! Compile-time embedded agent templates, mirrored from the repo's `agents/`
//! directory (same files the CLI embeds via `install_bundled_agents`).
//!
//! The templates API merges these with `$OPENFANG_HOME/agents/` so that
//! templates added in a new release are visible immediately after updating —
//! without them, users only ever saw the set copied to disk by their first
//! `openfang init`.

/// All bundled templates as `(name, toml_content)` pairs.
pub fn bundled_templates() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "analyst",
            include_str!("../../../agents/analyst/agent.toml"),
        ),
        (
            "architect",
            include_str!("../../../agents/architect/agent.toml"),
        ),
        (
            "assistant",
            include_str!("../../../agents/assistant/agent.toml"),
        ),
        ("coder", include_str!("../../../agents/coder/agent.toml")),
        (
            "code-reviewer",
            include_str!("../../../agents/code-reviewer/agent.toml"),
        ),
        (
            "customer-support",
            include_str!("../../../agents/customer-support/agent.toml"),
        ),
        (
            "data-scientist",
            include_str!("../../../agents/data-scientist/agent.toml"),
        ),
        (
            "debugger",
            include_str!("../../../agents/debugger/agent.toml"),
        ),
        (
            "devops-lead",
            include_str!("../../../agents/devops-lead/agent.toml"),
        ),
        (
            "doc-writer",
            include_str!("../../../agents/doc-writer/agent.toml"),
        ),
        (
            "email-assistant",
            include_str!("../../../agents/email-assistant/agent.toml"),
        ),
        (
            "freeco-ceo",
            include_str!("../../../agents/freeco-ceo/agent.toml"),
        ),
        (
            "freeco-concierge",
            include_str!("../../../agents/freeco-concierge/agent.toml"),
        ),
        (
            "freeco-developer",
            include_str!("../../../agents/freeco-developer/agent.toml"),
        ),
        (
            "freeco-kids",
            include_str!("../../../agents/freeco-kids/agent.toml"),
        ),
        (
            "freeco-secretary",
            include_str!("../../../agents/freeco-secretary/agent.toml"),
        ),
        (
            "freeco-shopping",
            include_str!("../../../agents/freeco-shopping/agent.toml"),
        ),
        (
            "freeco-tester",
            include_str!("../../../agents/freeco-tester/agent.toml"),
        ),
        (
            "health-tracker",
            include_str!("../../../agents/health-tracker/agent.toml"),
        ),
        (
            "hello-world",
            include_str!("../../../agents/hello-world/agent.toml"),
        ),
    ]
}

/// Look up a single bundled template body by name.
pub fn bundled_template(name: &str) -> Option<&'static str> {
    bundled_templates()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, body)| body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_bundled_templates_parse_as_toml() {
        for (name, body) in bundled_templates() {
            let parsed: Result<toml::Table, _> = body.parse();
            assert!(parsed.is_ok(), "template {name} is not valid TOML");
            let t = parsed.unwrap();
            assert_eq!(
                t.get("name").and_then(|v| v.as_str()),
                Some(name),
                "template {name}: manifest name mismatch"
            );
        }
    }

    #[test]
    fn freeco_editions_are_bundled() {
        for n in [
            "freeco-ceo",
            "freeco-concierge",
            "freeco-kids",
            "freeco-secretary",
            "freeco-shopping",
            "freeco-developer",
            "freeco-tester",
        ] {
            assert!(bundled_template(n).is_some(), "{n} missing from bundle");
        }
    }
}
