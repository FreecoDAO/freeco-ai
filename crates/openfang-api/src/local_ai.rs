//! One-click free local AI — guided Ollama setup with progress reporting.
//!
//! `POST /api/local-ai/setup` starts a background task that:
//! 1. detects a running Ollama at `127.0.0.1:11434`;
//! 2. on Windows, downloads the official installer (from ollama.com only)
//!    and runs it silently per-user; on macOS/Linux it reports guided
//!    manual instructions instead (installs there need the user);
//! 3. pulls the requested model (default: a lightweight Gemma) streaming
//!    download progress;
//! 4. writes `[default_model]` in config.toml to point at Ollama.
//!
//! `GET /api/local-ai/status` returns the live progress so the dashboard
//! can show a progress bar. State is process-local and single-flight: only
//! one setup can run at a time.

use crate::routes::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

const OLLAMA_BASE: &str = "http://127.0.0.1:11434";
const OLLAMA_WINDOWS_INSTALLER: &str = "https://ollama.com/download/OllamaSetup.exe";
/// Default starter model — small enough for ordinary laptops.
const DEFAULT_MODEL: &str = "gemma3:4b";

#[derive(Debug, Clone, Serialize, Default)]
pub struct LocalAiStatus {
    /// idle | checking | downloading-installer | installing | starting |
    /// pulling-model | configuring | done | needs-manual-install | error
    pub phase: String,
    /// Human-readable progress detail.
    pub detail: String,
    /// 0–100 for phases with measurable progress, otherwise -1.
    pub percent: i32,
    /// The model being set up.
    pub model: String,
    /// True while the background task runs.
    pub running: bool,
}

pub type SharedLocalAiStatus = Arc<tokio::sync::RwLock<LocalAiStatus>>;

async fn set_status(s: &SharedLocalAiStatus, phase: &str, detail: String, percent: i32) {
    let mut w = s.write().await;
    w.phase = phase.to_string();
    w.detail = detail;
    w.percent = percent;
}

async fn ollama_running() -> bool {
    let client = reqwest::Client::new();
    matches!(
        client
            .get(format!("{OLLAMA_BASE}/api/version"))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await,
        Ok(r) if r.status().is_success()
    )
}

/// GET /api/local-ai/status
pub async fn local_ai_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let s = state.local_ai.read().await.clone();
    Json(serde_json::json!({
        "phase": if s.phase.is_empty() { "idle" } else { s.phase.as_str() },
        "detail": s.detail,
        "percent": s.percent,
        "model": s.model,
        "running": s.running,
        "ollama_detected": ollama_running().await,
    }))
}

/// POST /api/local-ai/setup — body: optional {"model": "gemma3:4b"}
pub async fn local_ai_setup(
    State(state): State<Arc<AppState>>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    let model = body
        .as_ref()
        .and_then(|b| b.get("model"))
        .and_then(|m| m.as_str())
        .unwrap_or(DEFAULT_MODEL)
        .to_string();

    // SECURITY: model names feed a JSON body sent to localhost Ollama only,
    // but validate anyway — alphanumeric plus [._:-], max 64 chars.
    if model.is_empty()
        || model.len() > 64
        || !model
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid model name"})),
        );
    }

    {
        let mut s = state.local_ai.write().await;
        if s.running {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "setup already running"})),
            );
        }
        s.running = true;
        s.model = model.clone();
        s.phase = "checking".into();
        s.detail = "Checking for a local Ollama runtime...".into();
        s.percent = -1;
    }

    let status = state.local_ai.clone();
    let config_home = state.kernel.config.home_dir.clone();
    let task_model = model.clone();
    tokio::spawn(async move {
        let result = run_setup(&status, &task_model, &config_home).await;
        let mut s = status.write().await;
        s.running = false;
        if let Err(e) = result {
            s.phase = "error".into();
            s.detail = e;
            s.percent = -1;
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"status": "started", "model": model})),
    )
}

async fn run_setup(
    status: &SharedLocalAiStatus,
    model: &str,
    config_home: &std::path::Path,
) -> Result<(), String> {
    // 1. Detect
    if !ollama_running().await {
        if cfg!(target_os = "windows") {
            install_ollama_windows(status).await?;
        } else {
            set_status(
                status,
                "needs-manual-install",
                "Ollama isn't installed. Install it from https://ollama.com/download (one click), \
                 then press Set up again — the model download will continue automatically."
                    .into(),
                -1,
            )
            .await;
            return Ok(());
        }
    }

    // 2. Pull model with streaming progress.
    //
    // Network hiccups are the norm on multi-GB downloads. Ollama caches
    // completed layers, so retrying continues from what's already on disk
    // instead of starting over — but only if WE retry instead of failing
    // out and making the user press the button again. Retry with backoff.
    set_status(
        status,
        "pulling-model",
        format!("Downloading {model} (this is the big download — a few GB)..."),
        0,
    )
    .await;
    const MAX_PULL_ATTEMPTS: u32 = 6;
    let mut attempt = 1;
    loop {
        match pull_model(status, model).await {
            Ok(()) => break,
            Err(e) if attempt < MAX_PULL_ATTEMPTS => {
                let wait = 5 * attempt as u64;
                set_status(
                    status,
                    "pulling-model",
                    format!(
                        "Connection dropped ({e}) — resuming download automatically \
                         in {wait}s (attempt {attempt}/{MAX_PULL_ATTEMPTS}, finished \
                         parts are kept)..."
                    ),
                    -1,
                )
                .await;
                tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                attempt += 1;
            }
            Err(e) => {
                return Err(format!(
                    "Model download kept failing after {MAX_PULL_ATTEMPTS} attempts: {e}. \
                     Finished parts are kept — press Set up again to resume from where it stopped."
                ))
            }
        }
    }

    // 3. Point default_model at Ollama
    set_status(
        status,
        "configuring",
        "Setting the local model as default...".into(),
        -1,
    )
    .await;
    write_default_model(config_home, model)?;

    set_status(
        status,
        "done",
        format!(
            "Local AI ready — {model} runs on this device. Reload config or restart to activate."
        ),
        100,
    )
    .await;
    Ok(())
}

/// Verify a downloaded Windows executable is validly Authenticode-signed by
/// Ollama before we run it (threat-model M4). Uses PowerShell's
/// `Get-AuthenticodeSignature`; fails closed unless the status is `Valid` and
/// the signer's subject mentions Ollama.
#[cfg(windows)]
async fn verify_windows_signature(path: &std::path::Path) -> Result<(), String> {
    let script = format!(
        "$s = Get-AuthenticodeSignature -LiteralPath '{}'; \
         if ($s.Status -ne 'Valid') {{ Write-Output ('INVALID:' + $s.Status); exit 0 }}; \
         Write-Output ('OK:' + $s.SignerCertificate.Subject)",
        path.display().to_string().replace('\'', "''")
    );
    let out = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .await
        .map_err(|e| format!("could not run signature check: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.trim();
    if let Some(subject) = line.strip_prefix("OK:") {
        if subject.to_lowercase().contains("ollama") {
            return Ok(());
        }
        return Err(format!(
            "installer is signed, but not by Ollama (signer: {subject}). \
             Refusing to run it for your safety."
        ));
    }
    Err(format!(
        "installer failed digital-signature verification ({}). It was not run. \
         Install Ollama manually from https://ollama.com/download instead.",
        line.strip_prefix("INVALID:").unwrap_or("unknown")
    ))
}

/// Non-Windows builds never reach the Windows installer path; provide a stub
/// so the call site compiles on all targets.
#[cfg(not(windows))]
#[allow(dead_code)]
async fn verify_windows_signature(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

async fn install_ollama_windows(status: &SharedLocalAiStatus) -> Result<(), String> {
    set_status(
        status,
        "downloading-installer",
        "Downloading Ollama (official installer from ollama.com)...".into(),
        0,
    )
    .await;

    let dir = std::env::temp_dir().join("freeco-local-ai");
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp dir: {e}"))?;
    let installer = dir.join("OllamaSetup.exe");

    let client = reqwest::Client::new();
    let resp = client
        .get(OLLAMA_WINDOWS_INSTALLER)
        .send()
        .await
        .map_err(|e| format!("download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("download failed: {e}"))?;
    let total = resp.content_length().unwrap_or(0);

    let mut file = tokio::fs::File::create(&installer)
        .await
        .map_err(|e| format!("create installer file: {e}"))?;
    let mut stream = resp.bytes_stream();
    let mut got: u64 = 0;
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download interrupted: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write installer: {e}"))?;
        got += chunk.len() as u64;
        if total > 0 {
            let pct = ((got as f64 / total as f64) * 100.0) as i32;
            set_status(
                status,
                "downloading-installer",
                format!("Downloading Ollama installer... {pct}%"),
                pct,
            )
            .await;
        }
    }
    file.flush().await.ok();

    set_status(
        status,
        "verifying",
        "Verifying the installer's digital signature...".into(),
        -1,
    )
    .await;
    // SECURITY (threat-model M4): never execute a downloaded installer without
    // verifying it. HTTPS protects transit but not integrity — a compromised
    // mirror/CDN or a changed URL could yield arbitrary code execution inside
    // our "safe one-click" flow. We verify the Windows Authenticode signature
    // (publisher identity), which survives Ollama's frequent version bumps
    // better than a pinned hash would. Fail closed on anything but a valid,
    // Ollama-signed binary.
    verify_windows_signature(&installer).await?;

    set_status(
        status,
        "installing",
        "Installing Ollama (silent, per-user, no admin needed)...".into(),
        -1,
    )
    .await;
    let out = tokio::process::Command::new(&installer)
        .args(["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"])
        .output()
        .await
        .map_err(|e| format!("run installer: {e}"))?;
    if !out.status.success() {
        return Err(format!("installer exited with {:?}", out.status.code()));
    }

    set_status(status, "starting", "Starting Ollama...".into(), -1).await;
    // The installer usually starts Ollama; if not, poke the app binary.
    for i in 0..60 {
        if ollama_running().await {
            return Ok(());
        }
        if i == 5 {
            if let Some(local) = dirs::data_local_dir() {
                let app = local.join("Programs").join("Ollama").join("ollama app.exe");
                if app.exists() {
                    let _ = tokio::process::Command::new(app).spawn();
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    Err("Ollama installed but did not start within 3 minutes — start it from the Start Menu and press Set up again".into())
}

async fn pull_model(status: &SharedLocalAiStatus, model: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{OLLAMA_BASE}/api/pull"))
        .json(&serde_json::json!({"model": model, "stream": true}))
        .send()
        .await
        .map_err(|e| format!("pull request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("pull request failed: {e}"))?;

    use futures::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buf = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("pull interrupted: {e}"))?;
        buf.extend_from_slice(&chunk);
        // Ollama streams newline-delimited JSON.
        while let Some(pos) = buf.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = buf.drain(..=pos).collect();
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&line) {
                if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
                    return Err(format!("ollama: {err}"));
                }
                let phase_txt = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
                let (completed, total) = (
                    v.get("completed").and_then(|c| c.as_u64()).unwrap_or(0),
                    v.get("total").and_then(|t| t.as_u64()).unwrap_or(0),
                );
                let pct = if total > 0 {
                    ((completed as f64 / total as f64) * 100.0) as i32
                } else {
                    -1
                };
                set_status(
                    status,
                    "pulling-model",
                    format!("{model}: {phase_txt}"),
                    pct,
                )
                .await;
            }
        }
    }
    Ok(())
}

/// Write `[default_model]` in config.toml pointing at the local Ollama model.
fn write_default_model(home: &std::path::Path, model: &str) -> Result<(), String> {
    let path = home.join("config.toml");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut table: toml::Table = existing.parse().unwrap_or_default();

    let mut dm = toml::Table::new();
    dm.insert("provider".into(), toml::Value::String("ollama".into()));
    dm.insert("model".into(), toml::Value::String(model.into()));
    dm.insert("api_key_env".into(), toml::Value::String(String::new()));
    dm.insert(
        "base_url".into(),
        toml::Value::String("http://127.0.0.1:11434/v1".into()),
    );
    table.insert("default_model".into(), toml::Value::Table(dm));

    let rendered = toml::to_string_pretty(&table).map_err(|e| format!("render config: {e}"))?;
    std::fs::write(&path, rendered).map_err(|e| format!("write config.toml: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_written_to_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "api_listen = \"127.0.0.1:4200\"\n",
        )
        .unwrap();
        write_default_model(dir.path(), "gemma3:4b").unwrap();
        let out = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(out.contains("provider = \"ollama\""));
        assert!(out.contains("gemma3:4b"));
        // Pre-existing keys survive the rewrite.
        assert!(out.contains("api_listen"));
    }

    #[test]
    fn model_name_validation_pattern() {
        for good in ["gemma3:4b", "llama3.2:1b", "qwen2.5-coder:7b"] {
            assert!(good
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c)));
        }
        assert!(!"bad name; rm -rf"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c)));
    }
}
