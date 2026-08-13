//! Companies, projects and teams — the structure conversations belong to.
//!
//! A chat with no home is a chat you cannot find again. Attaching every
//! session to a project is what turns a pile of conversations into an
//! organisation's memory: each team sees its own history, and an assistant
//! acting as CEO can read across all of them and delegate within one.

use crate::session::Session;
use chrono::Utc;
use freeco_types::error::{FreecoError, FreecoResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// A company: the outermost scope. Owns projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A project. Conversations, tasks and teams hang off this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub company_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A team within a project. Agents are members; membership scopes what an
/// agent can see.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub project_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Store for the organisational structure.
#[derive(Clone)]
pub struct OrgStore {
    conn: Arc<Mutex<Connection>>,
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn lock_err(e: impl std::fmt::Display) -> FreecoError {
    FreecoError::Internal(e.to_string())
}

fn mem_err(e: impl std::fmt::Display) -> FreecoError {
    FreecoError::Memory(e.to_string())
}

impl OrgStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ---- Companies -----------------------------------------------------

    pub fn create_company(&self, name: &str, description: Option<&str>) -> FreecoResult<Company> {
        let company = Company {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.map(String::from),
            archived: false,
            created_at: now(),
            updated_at: now(),
        };
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "INSERT INTO companies (id, name, description, archived, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, ?4, ?4)",
            rusqlite::params![
                company.id,
                company.name,
                company.description,
                company.created_at
            ],
        )
        .map_err(mem_err)?;
        Ok(company)
    }

    pub fn list_companies(&self, include_archived: bool) -> FreecoResult<Vec<Company>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, archived, created_at, updated_at FROM companies
                 WHERE (?1 = 1 OR archived = 0) ORDER BY name",
            )
            .map_err(mem_err)?;
        let rows = stmt
            .query_map([include_archived as i64], |r| {
                Ok(Company {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    description: r.get(2)?,
                    archived: r.get::<_, i64>(3)? != 0,
                    created_at: r.get(4)?,
                    updated_at: r.get(5)?,
                })
            })
            .map_err(mem_err)?;
        rows.collect::<Result<_, _>>().map_err(mem_err)
    }

    // ---- Projects ------------------------------------------------------

    pub fn create_project(
        &self,
        name: &str,
        company_id: Option<&str>,
        description: Option<&str>,
    ) -> FreecoResult<Project> {
        let project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            company_id: company_id.map(String::from),
            name: name.to_string(),
            description: description.map(String::from),
            status: "active".into(),
            archived: false,
            created_at: now(),
            updated_at: now(),
        };
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "INSERT INTO projects (id, company_id, name, description, status, archived, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
            rusqlite::params![
                project.id, project.company_id, project.name,
                project.description, project.status, project.created_at
            ],
        )
        .map_err(mem_err)?;
        Ok(project)
    }

    pub fn list_projects(&self, include_archived: bool) -> FreecoResult<Vec<Project>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, name, description, status, archived, created_at, updated_at
                 FROM projects WHERE (?1 = 1 OR archived = 0) ORDER BY updated_at DESC",
            )
            .map_err(mem_err)?;
        let rows = stmt
            .query_map([include_archived as i64], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    company_id: r.get(1)?,
                    name: r.get(2)?,
                    description: r.get(3)?,
                    status: r.get(4)?,
                    archived: r.get::<_, i64>(5)? != 0,
                    created_at: r.get(6)?,
                    updated_at: r.get(7)?,
                })
            })
            .map_err(mem_err)?;
        rows.collect::<Result<_, _>>().map_err(mem_err)
    }

    /// Find a project by name, case-insensitively.
    ///
    /// Used when routing a new conversation: "the Kubuntu work" should land in
    /// the existing project rather than silently creating a second one with
    /// different capitalisation.
    pub fn find_project_by_name(&self, name: &str) -> FreecoResult<Option<Project>> {
        Ok(self
            .list_projects(true)?
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(name.trim())))
    }

    /// Route a conversation to a project: reuse the one that matches, or open
    /// a new one. Never creates a duplicate of an existing name.
    pub fn route_to_project(&self, name: &str) -> FreecoResult<Project> {
        match self.find_project_by_name(name)? {
            Some(existing) => Ok(existing),
            None => self.create_project(name, None, None),
        }
    }

    // ---- Teams ---------------------------------------------------------

    pub fn create_team(&self, name: &str, project_id: Option<&str>) -> FreecoResult<Team> {
        let team = Team {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.map(String::from),
            name: name.to_string(),
            description: None,
            archived: false,
            created_at: now(),
            updated_at: now(),
        };
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "INSERT INTO teams (id, project_id, name, description, archived, created_at, updated_at)
             VALUES (?1, ?2, ?3, NULL, 0, ?4, ?4)",
            rusqlite::params![team.id, team.project_id, team.name, team.created_at],
        )
        .map_err(mem_err)?;
        Ok(team)
    }

    pub fn add_team_member(
        &self,
        team_id: &str,
        agent_id: &str,
        role: Option<&str>,
    ) -> FreecoResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "INSERT OR REPLACE INTO team_members (team_id, agent_id, role, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![team_id, agent_id, role, now()],
        )
        .map_err(mem_err)?;
        Ok(())
    }

    // ---- Scoping -------------------------------------------------------

    /// Attach a session to a project and optionally a team.
    pub fn assign_session(
        &self,
        session_id: &str,
        project_id: Option<&str>,
        team_id: Option<&str>,
    ) -> FreecoResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "UPDATE sessions SET project_id = ?1, team_id = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![project_id, team_id, now(), session_id],
        )
        .map_err(mem_err)?;
        Ok(())
    }

    /// Archive ("done, keep it") or un-archive a session.
    ///
    /// Deliberately distinct from trashing. If the only way to clear finished
    /// work off a list is to make it look deleted, people stop clearing it.
    pub fn set_session_archived(&self, session_id: &str, archived: bool) -> FreecoResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "UPDATE sessions SET archived = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![archived as i64, now(), session_id],
        )
        .map_err(mem_err)?;
        Ok(())
    }

    /// Move to trash, or restore. Nothing is deleted: after losing history to
    /// a compaction bug, "trash" here means hidden and recoverable, and only
    /// an explicit purge removes anything.
    pub fn set_session_trashed(&self, session_id: &str, trashed: bool) -> FreecoResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "UPDATE sessions SET trashed = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![trashed as i64, now(), session_id],
        )
        .map_err(mem_err)?;
        Ok(())
    }

    /// Sessions belonging to a project — the team's own history.
    pub fn sessions_for_project(&self, project_id: &str) -> FreecoResult<Vec<serde_json::Value>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label, kind, archived, updated_at FROM sessions
                 WHERE project_id = ?1 AND trashed = 0
                 ORDER BY updated_at DESC",
            )
            .map_err(mem_err)?;
        let rows = stmt
            .query_map([project_id], |r| {
                Ok(serde_json::json!({
                    "id": r.get::<_, String>(0)?,
                    "label": r.get::<_, Option<String>>(1)?,
                    "kind": r.get::<_, String>(2)?,
                    "archived": r.get::<_, i64>(3)? != 0,
                    "updated_at": r.get::<_, String>(4)?,
                }))
            })
            .map_err(mem_err)?;
        rows.collect::<Result<_, _>>().map_err(mem_err)
    }

    /// Everything the CEO-level assistant can see: one line per project with
    /// its conversation count, so it can answer "what is happening?" without
    /// loading every message in the organisation into a prompt.
    pub fn org_overview(&self) -> FreecoResult<Vec<serde_json::Value>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT p.id, p.name, p.status, c.name,
                        (SELECT count(*) FROM sessions s
                          WHERE s.project_id = p.id AND s.trashed = 0) AS sessions
                 FROM projects p
                 LEFT JOIN companies c ON c.id = p.company_id
                 WHERE p.archived = 0
                 ORDER BY p.updated_at DESC",
            )
            .map_err(mem_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "project_id": r.get::<_, String>(0)?,
                    "project": r.get::<_, String>(1)?,
                    "status": r.get::<_, String>(2)?,
                    "company": r.get::<_, Option<String>>(3)?,
                    "sessions": r.get::<_, i64>(4)?,
                }))
            })
            .map_err(mem_err)?;
        rows.collect::<Result<_, _>>().map_err(mem_err)
    }

    /// Projects an agent can reach, via the teams it belongs to. This is what
    /// keeps a team's context its own: an agent on one team does not read
    /// another team's conversations by default.
    pub fn projects_for_agent(&self, agent_id: &str) -> FreecoResult<Vec<String>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT t.project_id FROM team_members m
                 JOIN teams t ON t.id = m.team_id
                 WHERE m.agent_id = ?1 AND t.project_id IS NOT NULL",
            )
            .map_err(mem_err)?;
        let rows = stmt
            .query_map([agent_id], |r| r.get::<_, String>(0))
            .map_err(mem_err)?;
        rows.collect::<Result<_, _>>().map_err(mem_err)
    }
}

/// Suggest a project name from the first thing the user said.
///
/// Deterministic rather than model-generated for the same reason session
/// labels are: routing a conversation must not cost a token, wait on a
/// network call, or fail when the model is unreachable.
pub fn suggest_project_name(session: &Session) -> Option<String> {
    crate::session::derive_session_label(&session.messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::run_migrations;

    fn store() -> OrgStore {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        OrgStore::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn companies_own_projects_own_teams() {
        let s = store();
        let c = s.create_company("FreEco AG", Some("Swiss")).unwrap();
        let p = s.create_project("Kubuntu USB", Some(&c.id), None).unwrap();
        let t = s.create_team("Platform", Some(&p.id)).unwrap();

        assert_eq!(s.list_companies(false).unwrap().len(), 1);
        assert_eq!(
            s.list_projects(false).unwrap()[0].company_id.as_deref(),
            Some(c.id.as_str())
        );
        assert_eq!(t.project_id.as_deref(), Some(p.id.as_str()));
    }

    /// Routing must reuse an existing project rather than quietly creating a
    /// near-duplicate. Two projects called "Kubuntu USB" and "kubuntu usb"
    /// split a team's history in half, which is exactly the memory loss this
    /// structure exists to prevent.
    #[test]
    fn routing_reuses_a_project_regardless_of_case() {
        let s = store();
        let first = s.route_to_project("Kubuntu USB").unwrap();
        let again = s.route_to_project("kubuntu usb").unwrap();
        let padded = s.route_to_project("  Kubuntu USB  ").unwrap();

        assert_eq!(first.id, again.id);
        assert_eq!(first.id, padded.id);
        assert_eq!(s.list_projects(true).unwrap().len(), 1);
    }

    #[test]
    fn an_agent_only_sees_projects_of_teams_it_joined() {
        let s = store();
        let mine = s.create_project("Mine", None, None).unwrap();
        let theirs = s.create_project("Theirs", None, None).unwrap();
        let team = s.create_team("Platform", Some(&mine.id)).unwrap();
        s.create_team("Other", Some(&theirs.id)).unwrap();
        s.add_team_member(&team.id, "agent-1", Some("developer"))
            .unwrap();

        let visible = s.projects_for_agent("agent-1").unwrap();
        assert_eq!(visible, vec![mine.id]);
        assert!(s.projects_for_agent("agent-2").unwrap().is_empty());
    }

    /// Archive and trash are different states, and neither destroys anything.
    #[test]
    fn archive_and_trash_are_separate_and_non_destructive() {
        let s = store();
        let p = s.create_project("Demo", None, None).unwrap();
        {
            let conn = s.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO sessions (id, agent_id, messages, context_window_tokens, created_at, updated_at, project_id)
                 VALUES ('s1', 'a1', X'90', 0, datetime('now'), datetime('now'), ?1)",
                [&p.id],
            )
            .unwrap();
        }

        s.set_session_archived("s1", true).unwrap();
        let listed = s.sessions_for_project(&p.id).unwrap();
        assert_eq!(listed.len(), 1, "archived sessions stay listed");
        assert_eq!(listed[0]["archived"], true);

        // Trashing hides it from the project view, but the row survives.
        s.set_session_trashed("s1", true).unwrap();
        assert!(s.sessions_for_project(&p.id).unwrap().is_empty());

        s.set_session_trashed("s1", false).unwrap();
        assert_eq!(
            s.sessions_for_project(&p.id).unwrap().len(),
            1,
            "restorable"
        );
    }

    /// The overview is what lets an assistant act across the organisation
    /// without pulling every message into a prompt.
    #[test]
    fn overview_counts_conversations_per_project() {
        let s = store();
        let c = s.create_company("FreEco", None).unwrap();
        let p = s.create_project("Trading", Some(&c.id), None).unwrap();
        {
            let conn = s.conn.lock().unwrap();
            for id in ["s1", "s2"] {
                conn.execute(
                    "INSERT INTO sessions (id, agent_id, messages, context_window_tokens, created_at, updated_at, project_id)
                     VALUES (?1, 'a1', X'90', 0, datetime('now'), datetime('now'), ?2)",
                    rusqlite::params![id, p.id],
                )
                .unwrap();
            }
        }
        let view = s.org_overview().unwrap();
        assert_eq!(view.len(), 1);
        assert_eq!(view[0]["project"], "Trading");
        assert_eq!(view[0]["company"], "FreEco");
        assert_eq!(view[0]["sessions"], 2);
    }
}
