//! What the Freeco Assistant is allowed to do on its own, and what it must ask about.
//!
//! The Assistant acts as a CEO across the whole system: it can create and tune
//! agents, wire up tools and MCP hands, organise teams and projects, write code
//! in the sandbox, and open and merge pull requests. That breadth is the point —
//! an orchestrator that has to be driven by hand for every step is just a chat
//! window.
//!
//! Breadth without gates is the failure mode, though. The rule here is narrow
//! and worth stating plainly: **an action is free if its damage is bounded and
//! reversible, and gated if it is neither.** Reading anything is free. Creating
//! things is free, because an unwanted new agent can be deleted. Destroying
//! things, changing what the system trusts, spending money, and anything that
//! leaves this machine are gated, because no amount of care afterwards undoes
//! them.
//!
//! Gating is deliberately not a function of how "important" something feels. A
//! prompt tweak on a production agent feels weightier than deleting a scratch
//! project, but the tweak is reversible and the deletion is not.

use serde::{Deserialize, Serialize};

use crate::approval::RiskLevel;

/// Every capability the Assistant can exercise, and what it costs to be wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // ---- Reading. Always free: an orchestrator that must ask permission to
    // look at things cannot orchestrate anything.
    ReadAgents,
    ReadTools,
    ReadOrg,
    ReadSessions,
    ReadConfig,
    ReadRepo,

    // ---- Creating. Free, because the worst case is clutter, and clutter is
    // deletable. A new agent that turns out to be wrong costs a deletion; a
    // permission prompt on every creation costs the user their attention all
    // day, which is the scarcer resource.
    CreateAgent,
    CreateTeam,
    CreateProject,
    CreateCompany,
    CreateTask,
    CreateSession,

    // ---- Tuning. Free for reversible edits: a prompt or a model can be
    // changed back, and the previous value is in the record.
    TuneAgentPrompt,
    TuneAgentModel,
    AssignSessionToProject,
    ArchiveSession,

    // ---- Sandboxed work. Free because the sandbox is the boundary: no
    // network, dropped capabilities, thrown away afterwards. Requiring
    // approval here would push the work back onto the host, which is strictly
    // worse.
    RunInSandbox,
    WriteSandboxCode,

    // ---- Destruction. Gated. Deleting an agent takes its history with it,
    // and no care afterwards brings it back.
    DeleteAgent,
    KillAgent,
    DeleteProject,
    DeleteTeam,
    PurgeSession,

    // ---- Trust and permissions. Gated. These decide what the system will
    // allow later, so a mistake here is not one bad action but a standing
    // grant that keeps producing them.
    ModifyToolFilters,
    InstallTool,
    InstallMcpHand,
    ModifySystemConfig,
    ModifyExecPolicy,
    ModifyUserRoles,

    // ---- Leaving the machine. Gated. Anything published, sent or spent
    // cannot be recalled, whatever the dashboard says afterwards.
    MergePullRequest,
    PushToRemote,
    CreatePullRequest,
    SendMessageExternally,
    SpendBudget,
}

/// What the Assistant may do with a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grant {
    /// Proceed without asking.
    Allowed,
    /// Ask first, every time, and wait for an answer.
    NeedsApproval(RiskLevel),
}

impl Capability {
    /// Whether this capability is free or gated, and how loudly to ask.
    ///
    /// Written as one exhaustive match on purpose. A new capability will not
    /// compile until someone decides which side of the line it falls on, which
    /// is the only reliable way to stop the gated list quietly rotting as the
    /// system grows.
    pub fn grant(self) -> Grant {
        use Capability::*;
        use RiskLevel::*;
        match self {
            // Reading and creating: bounded, reversible.
            ReadAgents | ReadTools | ReadOrg | ReadSessions | ReadConfig | ReadRepo => {
                Grant::Allowed
            }
            CreateAgent | CreateTeam | CreateProject | CreateCompany | CreateTask
            | CreateSession => Grant::Allowed,

            // Reversible edits, and work inside the sandbox boundary.
            TuneAgentPrompt | TuneAgentModel | AssignSessionToProject | ArchiveSession => {
                Grant::Allowed
            }
            RunInSandbox | WriteSandboxCode => Grant::Allowed,

            // Destruction. High rather than Critical: painful, but scoped to
            // one thing the user can name.
            DeleteAgent | KillAgent | DeleteProject | DeleteTeam => Grant::NeedsApproval(High),
            // Purge is the one delete that truly destroys history, so it is
            // rated above the others even though it sounds smaller.
            PurgeSession => Grant::NeedsApproval(Critical),

            // Standing grants. Critical because the blast radius is every
            // future action, not this one.
            ModifyExecPolicy | ModifyUserRoles | ModifySystemConfig => {
                Grant::NeedsApproval(Critical)
            }
            InstallTool | InstallMcpHand => Grant::NeedsApproval(Critical),
            ModifyToolFilters => Grant::NeedsApproval(High),

            // Irreversible and outward-facing.
            MergePullRequest | PushToRemote => Grant::NeedsApproval(High),
            CreatePullRequest => Grant::NeedsApproval(Medium),
            SendMessageExternally => Grant::NeedsApproval(High),
            SpendBudget => Grant::NeedsApproval(High),
        }
    }

    /// True when the Assistant must stop and ask.
    pub fn needs_approval(self) -> bool {
        matches!(self.grant(), Grant::NeedsApproval(_))
    }

    /// Plain-language description of the action, for the approval card.
    ///
    /// Written for someone glancing at a prompt mid-task, so it says what will
    /// happen and what it costs — not the capability's own name, which tells
    /// the reader nothing they did not already see.
    pub fn describe(self) -> &'static str {
        use Capability::*;
        match self {
            ReadAgents => "look at your agents",
            ReadTools => "look at available tools",
            ReadOrg => "look at companies, projects and teams",
            ReadSessions => "read conversation history",
            ReadConfig => "read the configuration",
            ReadRepo => "read the repository",
            CreateAgent => "create a new agent",
            CreateTeam => "create a team",
            CreateProject => "create a project",
            CreateCompany => "create a company",
            CreateTask => "create a task",
            CreateSession => "start a new conversation",
            TuneAgentPrompt => "change an agent's instructions",
            TuneAgentModel => "change which model an agent uses",
            AssignSessionToProject => "file a conversation under a project",
            ArchiveSession => "archive a conversation",
            RunInSandbox => "run a command in the isolated sandbox",
            WriteSandboxCode => "write code inside the sandbox",
            DeleteAgent => "DELETE an agent, and its history with it",
            KillAgent => "stop a running agent mid-task",
            DeleteProject => "DELETE a project",
            DeleteTeam => "DELETE a team",
            PurgeSession => "PERMANENTLY destroy a conversation - this cannot be undone",
            ModifyToolFilters => "change which tools an agent may use",
            InstallTool => "install a new tool, which will run on this machine",
            InstallMcpHand => "connect a new MCP server, which can act on your behalf",
            ModifySystemConfig => "change system configuration",
            ModifyExecPolicy => "change what commands may run on this machine",
            ModifyUserRoles => "change who can do what",
            MergePullRequest => "MERGE a pull request into your repository",
            PushToRemote => "push commits to GitHub",
            CreatePullRequest => "open a pull request",
            SendMessageExternally => "send a message outside this machine",
            SpendBudget => "spend money against your budget",
        }
    }
}

/// How long an approval request stays open before auto-denying.
///
/// Four hours, matching the approval default. The Assistant works while the
/// user is away — that is the point of it — so a request raised at 2am must
/// still be answerable at 9am. A short window turns "I was asleep" into a
/// failed run that looks like a bug.
pub const ASSISTANT_APPROVAL_TIMEOUT_SECS: u64 = 4 * 60 * 60;

#[cfg(test)]
mod tests {
    use super::*;

    /// Reading and creating must never prompt. If they do, the Assistant
    /// cannot orchestrate anything without a human clicking through its every
    /// step, which defeats the purpose of having it.
    #[test]
    fn reading_and_creating_are_free() {
        for c in [
            Capability::ReadAgents,
            Capability::ReadOrg,
            Capability::ReadSessions,
            Capability::CreateAgent,
            Capability::CreateProject,
            Capability::CreateTeam,
        ] {
            assert!(!c.needs_approval(), "{c:?} must not prompt");
        }
    }

    /// The sandbox is the boundary. Gating work inside it would push that work
    /// back onto the host, which is the opposite of safe.
    #[test]
    fn sandboxed_work_is_free() {
        assert!(!Capability::RunInSandbox.needs_approval());
        assert!(!Capability::WriteSandboxCode.needs_approval());
    }

    /// Nothing irreversible, outward-facing, or trust-changing may happen
    /// without the user saying yes.
    #[test]
    fn destruction_and_trust_changes_are_gated() {
        for c in [
            Capability::DeleteAgent,
            Capability::PurgeSession,
            Capability::InstallTool,
            Capability::InstallMcpHand,
            Capability::ModifyExecPolicy,
            Capability::ModifyUserRoles,
            Capability::MergePullRequest,
            Capability::PushToRemote,
            Capability::SendMessageExternally,
            Capability::SpendBudget,
        ] {
            assert!(c.needs_approval(), "{c:?} must ask first");
        }
    }

    /// Permanent destruction and standing grants outrank one-off damage: the
    /// first cannot be undone, the second keeps causing damage after the fact.
    #[test]
    fn permanent_and_standing_risks_rate_highest() {
        for c in [
            Capability::PurgeSession,
            Capability::ModifyExecPolicy,
            Capability::InstallMcpHand,
        ] {
            assert_eq!(
                c.grant(),
                Grant::NeedsApproval(RiskLevel::Critical),
                "{c:?}"
            );
        }
    }

    /// An approval card that only names the capability tells the reader
    /// nothing. Every description must be plain language, and the destructive
    /// ones must say so where a glance will catch it.
    #[test]
    fn descriptions_are_written_for_a_human_glancing_at_a_prompt() {
        for c in [
            Capability::DeleteAgent,
            Capability::PurgeSession,
            Capability::MergePullRequest,
        ] {
            let d = c.describe();
            assert!(d.len() > 12, "{c:?} description is too thin: {d}");
            assert!(
                d.chars().any(|ch| ch.is_uppercase()),
                "{c:?} must flag its severity visibly: {d}"
            );
        }
    }

    /// A request raised while the user sleeps must still be answerable when
    /// they wake up.
    #[test]
    fn approval_window_survives_a_night() {
        assert!(ASSISTANT_APPROVAL_TIMEOUT_SECS >= 4 * 60 * 60);
    }
}
