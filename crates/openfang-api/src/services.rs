//! One-click service provisioning — Freeco installs and runs the services it
//! needs, instead of assuming the user already has them.
//!
//! `POST /api/services/{id}/install` starts a background task that:
//! 1. checks Docker is available (and says how to get it if not);
//! 2. materialises the service (git clone, or an embedded compose file) into
//!    `~/.openfang/services/{id}/`;
//! 3. runs `docker compose up -d`, streaming progress;
//! 4. waits for the service's health endpoint;
//! 5. registers it as an MCP server so agents can use it immediately.
//!
//! `GET /api/services` lists the catalog with live status; `GET
//! /api/services/status` returns progress for the running install.

use crate::routes::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

/// A service Freeco can install and run on the user's behalf.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub purpose: &'static str,
    /// Git repo to clone when the compose file needs the repo's other files.
    pub repo: Option<&'static str>,
    /// Self-contained compose file, when no repo checkout is needed.
    pub compose: Option<&'static str>,
    /// URL polled to decide the service is up.
    pub health_url: &'static str,
    /// MCP endpoint to auto-register once healthy (None = needs a wrapper/key).
    pub mcp_url: Option<&'static str>,
    /// Rough disk/download cost so the user is never surprised.
    pub approx_download_gb: f32,
    /// Extra step the user must do themselves (e.g. create an API key).
    pub manual_step: Option<&'static str>,
}

/// Minimal, self-contained compose for Twenty CRM.
const TWENTY_COMPOSE: &str = r#"services:
  twenty-db:
    image: postgres:16
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: freeco
    volumes: [twenty-db-data:/var/lib/postgresql/data]
  twenty:
    image: twentycrm/twenty:latest
    depends_on: [twenty-db]
    environment:
      PG_DATABASE_URL: postgres://postgres:freeco@twenty-db:5432/default
      SERVER_URL: http://localhost:3000
      APP_SECRET: freeco-local-dev-secret-change-me
    ports: ["3000:3000"]
    volumes: [twenty-data:/app/packages/twenty-server/.local-storage]
volumes:
  twenty-db-data:
  twenty-data:
"#;

/// Minimal, self-contained compose for Akaunting.
const AKAUNTING_COMPOSE: &str = r#"services:
  akaunting-db:
    image: mariadb:11
    environment:
      MYSQL_ROOT_PASSWORD: freeco
      MYSQL_DATABASE: akaunting
      MYSQL_USER: akaunting
      MYSQL_PASSWORD: freeco
    volumes: [akaunting-db-data:/var/lib/mysql]
  akaunting:
    image: akaunting/akaunting:latest
    depends_on: [akaunting-db]
    environment:
      DB_HOST: akaunting-db
      DB_DATABASE: akaunting
      DB_USERNAME: akaunting
      DB_PASSWORD: freeco
      APP_URL: http://localhost:8080
    ports: ["8080:80"]
    volumes: [akaunting-data:/var/www/html]
volumes:
  akaunting-db-data:
  akaunting-data:
"#;

pub const SERVICE_CATALOG: &[ServiceProfile] = &[
    ServiceProfile {
        id: "dograh",
        name: "dograh — real-time voice AI",
        purpose: "Live voice calls, barge-in, telephony and voice-campaign workflows for outreach.",
        repo: Some("https://github.com/FreecoDAO/dograh"),
        compose: None,
        health_url: "http://localhost:8000",
        // dograh is MCP-native, so it connects with no extra wrapper or key.
        mcp_url: Some("http://localhost:8000/mcp"),
        approx_download_gb: 3.5,
        manual_step: None,
    },
    ServiceProfile {
        id: "crm",
        name: "Twenty — CRM",
        purpose: "Where donors, grant givers, VCs and partners live: people, companies, notes, pipeline.",
        repo: None,
        compose: Some(TWENTY_COMPOSE),
        health_url: "http://localhost:3000",
        mcp_url: None,
        approx_download_gb: 2.0,
        manual_step: Some(
            "Open http://localhost:3000, create your workspace, then Settings → API & Webhooks → \
             create an API key. Paste it when connecting the CRM tools.",
        ),
    },
    ServiceProfile {
        id: "accounting",
        name: "Akaunting — accounting",
        purpose: "Auditable books: income/expense, invoices, and reports for grant funders.",
        repo: None,
        compose: Some(AKAUNTING_COMPOSE),
        health_url: "http://localhost:8080",
        mcp_url: None,
        approx_download_gb: 1.5,
        manual_step: Some(
            "Open http://localhost:8080 and finish the first-run wizard (company + admin user). \
             Those credentials are what the accounting tools use.",
        ),
    },
];

pub fn service_profile(id: &str) -> Option<&'static ServiceProfile> {
    SERVICE_CATALOG.iter().find(|s| s.id == id)
}

/// Live progress of a service install (mirrors the local-AI status shape).
#[derive(Debug, Clone, Serialize, Default)]
pub struct ServiceStatus {
    /// idle | checking | fetching | starting | waiting | connecting | done | needs-docker | error
    pub phase: String,
    pub detail: String,
    pub percent: i32,
    pub service: String,
    pub running: bool,
}

pub type SharedServiceStatus = Arc<tokio::sync::RwLock<ServiceStatus>>;

async fn set_status(s: &SharedServiceStatus, phase: &str, detail: String, percent: i32) {
    let mut w = s.write().await;
    w.phase = phase.to_string();
    w.detail = detail;
    w.percent = percent;
}

async fn run(cmd: &str, args: &[&str], cwd: Option<&std::path::Path>) -> Result<String, String> {
    let mut c = tokio::process::Command::new(cmd);
    c.args(args);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let out = c
        .output()
        .await
        .map_err(|e| format!("could not run {cmd}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Is Docker installed AND the daemon running?
pub async fn docker_ready() -> bool {
    run("docker", &["version", "--format", "{{.Server.Version}}"], None)
        .await
        .is_ok()
}

async fn health_ok(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build();
    let Ok(client) = client else { return false };
    // Any HTTP response means something is listening and serving.
    client.get(url).send().await.is_ok()
}

/// GET /api/services — catalog plus live status of each service.
pub async fn list_services(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let docker = docker_ready().await;
    let configured: std::collections::HashSet<&str> = state
        .kernel
        .config
        .mcp_servers
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    let mut items = Vec::new();
    for s in SERVICE_CATALOG {
        let running = health_ok(s.health_url).await;
        items.push(serde_json::json!({
            "id": s.id,
            "name": s.name,
            "purpose": s.purpose,
            "approx_download_gb": s.approx_download_gb,
            "manual_step": s.manual_step,
            "health_url": s.health_url,
            "running": running,
            "connected": configured.contains(s.id),
            "auto_connects": s.mcp_url.is_some(),
        }));
    }
    Json(serde_json::json!({ "docker_ready": docker, "services": items }))
}

/// GET /api/services/status — progress of the running install.
pub async fn service_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let s = state.services.read().await.clone();
    Json(serde_json::json!(s))
}

/// POST /api/services/{id}/install — install, run and connect a service.
pub async fn install_service(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let Some(profile) = service_profile(&id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("unknown service '{id}'")})),
        );
    };

    {
        let mut s = state.services.write().await;
        if s.running {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "another service install is already running"})),
            );
        }
        s.running = true;
        s.service = profile.id.to_string();
        s.phase = "checking".into();
        s.detail = format!("Preparing {}...", profile.name);
        s.percent = -1;
    }

    let status = state.services.clone();
    let home = state.kernel.config.home_dir.clone();
    tokio::spawn(async move {
        let result = provision(&status, profile, &home).await;
        let mut s = status.write().await;
        s.running = false;
        match result {
            Ok(()) => {}
            Err(e) => {
                s.phase = "error".into();
                s.detail = e;
                s.percent = -1;
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"status": "started", "service": profile.id})),
    )
}

async fn provision(
    status: &SharedServiceStatus,
    profile: &'static ServiceProfile,
    home: &std::path::Path,
) -> Result<(), String> {
    // 1. Docker
    if !docker_ready().await {
        set_status(
            status,
            "needs-docker",
            "Docker isn't available. Install Docker Desktop from https://docker.com/products/docker-desktop \
             (one installer), start it, then press Install again — everything else is automatic."
                .into(),
            -1,
        )
        .await;
        return Ok(());
    }

    let dir = home.join("services").join(profile.id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("create service dir: {e}"))?;

    // 2. Materialise the service definition.
    if let Some(compose) = profile.compose {
        set_status(
            status,
            "fetching",
            format!("Writing {} configuration...", profile.name),
            -1,
        )
        .await;
        std::fs::write(dir.join("docker-compose.yaml"), compose)
            .map_err(|e| format!("write compose: {e}"))?;
    } else if let Some(repo) = profile.repo {
        if dir.join(".git").exists() {
            set_status(status, "fetching", format!("Updating {}...", profile.name), -1).await;
            let _ = run("git", &["pull", "--ff-only"], Some(&dir)).await;
        } else {
            set_status(
                status,
                "fetching",
                format!(
                    "Downloading {} (about {:.1} GB including images)...",
                    profile.name, profile.approx_download_gb
                ),
                -1,
            )
            .await;
            run(
                "git",
                &["clone", "--depth", "1", repo, &dir.to_string_lossy()],
                None,
            )
            .await?;
        }
    }

    // 3. Start it.
    set_status(
        status,
        "starting",
        format!("Starting {} with Docker (first run pulls images)...", profile.name),
        -1,
    )
    .await;
    run("docker", &["compose", "up", "-d"], Some(&dir))
        .await
        .map_err(|e| format!("could not start {}: {e}", profile.name))?;

    // 4. Wait for health.
    set_status(
        status,
        "waiting",
        format!("Waiting for {} to come up...", profile.name),
        -1,
    )
    .await;
    let mut up = false;
    for i in 0..90 {
        if health_ok(profile.health_url).await {
            up = true;
            break;
        }
        if i % 10 == 0 {
            set_status(
                status,
                "waiting",
                format!(
                    "Still starting {} ({}s)... first run downloads images, this can take a few minutes.",
                    profile.name,
                    i * 2
                ),
                -1,
            )
            .await;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    if !up {
        return Err(format!(
            "{} was started but did not answer at {} within 3 minutes. \
             Check `docker compose logs` in {}.",
            profile.name,
            profile.health_url,
            dir.display()
        ));
    }

    // 5. Connect it (MCP-native services need nothing else).
    if let Some(mcp_url) = profile.mcp_url {
        set_status(
            status,
            "connecting",
            format!("Connecting {} to your agents...", profile.name),
            -1,
        )
        .await;
        let config_path = home.join("config.toml");
        crate::routes::register_mcp_http_server(&config_path, profile.id, mcp_url)
            .map_err(|e| format!("connected but could not save MCP entry: {e}"))?;
    }

    let done_msg = match profile.manual_step {
        Some(step) => format!(
            "{} is running at {}. One step left: {}",
            profile.name, profile.health_url, step
        ),
        None => format!(
            "{} is running and connected to your agents. Restart FreEco.ai to load its tools.",
            profile.name
        ),
    };
    set_status(status, "done", done_msg, 100).await;
    Ok(())
}
