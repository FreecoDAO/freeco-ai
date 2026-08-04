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
/// Default starter model — genuinely small enough for ordinary laptops.
/// (Gemma 4 E4B is a ~10 GB download and needs ~24 GB RAM; it is NOT a safe
/// default and was making setup download for hours on 8–16 GB machines.)
const DEFAULT_MODEL: &str = "gemma4:e2b";

/// Hardware requirements for a locally downloadable Ollama model. This is
/// deliberately catalog data rather than selection logic so the dashboard and
/// USB tooling can present the same conservative choices.
#[derive(Debug, Clone, Serialize)]
pub struct LocalModelProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub purpose: &'static str,
    pub min_ram_gb: u64,
    pub min_vram_gb: u64,
    pub min_disk_gb: u64,
    /// MEASURED download size in GB. Shown up-front so nobody starts a
    /// multi-hour download blind — on a 500 KB/s line, 5 GB is ~3 hours.
    pub download_gb: f32,
}

const LOCAL_MODEL_CATALOG: &[LocalModelProfile] = &[
    LocalModelProfile {
        id: "llama3.2:1b",
        display_name: "Llama 3.2 1B",
        purpose: "minimum-resource general assistant",
        min_ram_gb: 4,
        min_vram_gb: 0,
        min_disk_gb: 3,
        download_gb: 1.3,
    },
    LocalModelProfile {
        // Gemma has no sub-4B in the 4-series; the Gemma 3 "Nano" 2B is the
        // smallest verified-existing Gemma for weak machines (gemma4:1b/2b
        // do NOT exist on Ollama's registry — they 404).
        id: "gemma3n:e2b",
        display_name: "Gemma 3 Nano E2B",
        purpose: "balanced assistant for 8-16 GB laptops (~5.6 GB download)",
        min_ram_gb: 10,
        min_vram_gb: 0,
        min_disk_gb: 8,
        download_gb: 5.6,
    },
    LocalModelProfile {
        // MEASURED 7.2 GB. The lightest Gemma 4 that exists: the only real
        // gemma4 tags are e2b, e4b and 12b (1b/2b/4b/9b/27b all 404), so there
        // is no "small" Gemma 4 — even the smallest carries a large weight file.
        id: "gemma4:e2b",
        display_name: "Gemma 4 E2B",
        purpose: "lightest Gemma 4 — good general assistant for 16 GB machines",
        min_ram_gb: 12,
        min_vram_gb: 0,
        min_disk_gb: 10,
        download_gb: 7.2,
    },
    LocalModelProfile {
        // MEASURED: the actual Ollama download is ~9.6 GB (not the ~3 GB this
        // catalog used to claim), and a model needs roughly its file size in
        // RAM plus context. Recommending it to an 8 GB machine made setup
        // download for hours and then thrash. Sized honestly now.
        id: "gemma4:e4b",
        display_name: "Gemma 4 E4B",
        purpose: "high-quality general assistant (large download ~10 GB)",
        min_ram_gb: 24,
        min_vram_gb: 0,
        min_disk_gb: 14,
        download_gb: 9.6,
    },
    LocalModelProfile {
        id: "qwen2.5-coder:3b",
        display_name: "Qwen2.5-Coder 3B",
        purpose: "resource-conscious coding assistant",
        min_ram_gb: 8,
        min_vram_gb: 0,
        min_disk_gb: 5,
        download_gb: 1.9,
    },
    LocalModelProfile {
        id: "qwen2.5-coder:7b",
        display_name: "Qwen2.5-Coder 7B",
        purpose: "recommended local coding assistant",
        min_ram_gb: 16,
        min_vram_gb: 8,
        min_disk_gb: 10,
        download_gb: 4.7,
    },
    LocalModelProfile {
        id: "mistral:7b",
        display_name: "Mistral 7B",
        purpose: "general assistant alternative",
        min_ram_gb: 16,
        min_vram_gb: 8,
        min_disk_gb: 10,
        download_gb: 4.1,
    },
    LocalModelProfile {
        id: "gemma4:12b",
        display_name: "Gemma 4 12B",
        purpose: "higher-quality general assistant",
        min_ram_gb: 24,
        min_vram_gb: 12,
        min_disk_gb: 16,
        download_gb: 8.1,
    },
];

#[derive(Debug, Clone, Serialize)]
struct LocalHardware {
    os: String,
    architecture: String,
    ram_gb: Option<u64>,
    vram_gb: Option<u64>,
    free_disk_gb: Option<u64>,
    ollama_detected: bool,
    docker_detected: bool,
}

fn linux_mem_total_gb() -> Option<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kb = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    Some(kb / 1024 / 1024)
}

/// Total physical RAM in GiB on Windows, via PowerShell/CIM. Without this the
/// model picker sees `None` RAM and falls back to the tiniest model — the cause
/// of "Everyday: llama3.2:1b" on capable Windows machines.
async fn windows_mem_total_gb() -> Option<u64> {
    let out = command_output(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ],
    )
    .await?;
    let bytes = out.trim().parse::<u64>().ok()?;
    Some(bytes / 1024 / 1024 / 1024)
}

/// Total physical RAM in GiB on macOS, via `sysctl hw.memsize` (bytes).
async fn macos_mem_total_gb() -> Option<u64> {
    let out = command_output("sysctl", &["-n", "hw.memsize"]).await?;
    let bytes = out.trim().parse::<u64>().ok()?;
    Some(bytes / 1024 / 1024 / 1024)
}

async fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = openfang_runtime::quiet_command::quiet_async(command)
        .args(args)
        .output()
        .await
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn detect_hardware() -> LocalHardware {
    let os = std::env::consts::OS.to_string();
    let ram_gb = match os.as_str() {
        "linux" => linux_mem_total_gb(),
        "windows" => windows_mem_total_gb().await,
        "macos" => macos_mem_total_gb().await,
        _ => None,
    };
    let vram_gb = command_output(
        "nvidia-smi",
        &["--query-gpu=memory.total", "--format=csv,noheader,nounits"],
    )
    .await
    .and_then(|output| {
        output
            .lines()
            .filter_map(|line| line.trim().parse::<u64>().ok())
            .max()
            .map(|mb| mb / 1024)
    });
    let free_disk_gb = if os == "linux" || os == "macos" {
        command_output("df", &["-Pk", "."])
            .await
            .and_then(|output| {
                output
                    .lines()
                    .nth(1)
                    .and_then(|line| line.split_whitespace().nth(3))
                    .and_then(|kb| kb.parse::<u64>().ok())
                    .map(|kb| kb / 1024 / 1024)
            })
    } else {
        None
    };
    let docker_detected = command_output("docker", &["version", "--format", "{{.Server.Version}}"])
        .await
        .is_some();

    LocalHardware {
        os,
        architecture: std::env::consts::ARCH.to_string(),
        ram_gb,
        vram_gb,
        free_disk_gb,
        ollama_detected: ollama_running().await,
        docker_detected,
    }
}

/// Tokens/sec measured on an Intel UHD 620 with Vulkan offload — the weak end
/// of the range this runs on. Vulkan is already ~8x faster than pure CPU here,
/// so these are the *optimistic* numbers for a machine without a real GPU.
const IGPU_PROMPT_TOKENS_PER_SEC: f64 = 2.5;
/// A realistic agent turn: system prompt, tool schemas, and history. Chat with
/// a two-line question is far cheaper, which is why a quick "say hello" test
/// looks fine and real agent work then takes an hour.
const TYPICAL_AGENT_PROMPT_TOKENS: f64 = 11_500.0;

/// Whether local inference is actually worth enabling on this machine.
///
/// This exists because a local model that technically runs but takes an hour
/// per answer is worse than no local model at all: the assistant looks
/// configured and simply never replies. That failure is indistinguishable from
/// a broken install, and it is the single most common way local AI "does not
/// work". A GPU check up front is the honest fix.
#[derive(Debug, Clone, Serialize)]
pub struct LocalAiCapability {
    /// True only when this machine can run a local model at usable speed.
    pub suitable: bool,
    /// True when a discrete GPU with usable VRAM was detected.
    pub has_gpu: bool,
    pub vram_gb: u64,
    pub ram_gb: u64,
    /// Estimated minutes for ONE agent turn. None when a GPU makes it fast.
    pub est_minutes_per_agent_turn: Option<u64>,
    /// Plain-language explanation, shown to the user verbatim.
    pub reason: String,
    /// What this machine would need. Shown even when unsuitable, so the answer
    /// to "why not?" comes with "here is what would work" rather than a dead
    /// end. Empty when the machine is already capable.
    pub requirements: Vec<String>,
    /// Local setup is never blocked outright - the user may have a reason to
    /// accept the speed. This flags that proceeding needs explicit consent.
    pub can_proceed_anyway: bool,
}

/// The hardware that actually makes local inference usable.
///
/// Stated concretely rather than as "a decent GPU", because the whole point is
/// that the user can check these against a machine they own or are buying.
fn local_ai_requirements() -> Vec<String> {
    vec![
        "GPU: NVIDIA or AMD with 6 GB VRAM minimum, 8-12 GB comfortable \
         (e.g. RTX 3060/4060, RX 7600). The GPU is what decides speed - \
         it matters far more than the CPU."
            .to_string(),
        "Apple Silicon alternative: M1 or newer with 16 GB or more unified \
         memory works well, because the GPU shares system RAM."
            .to_string(),
        "RAM: 16 GB recommended, 8 GB the absolute floor alongside a GPU. \
         Below that the machine swaps and everything slows down."
            .to_string(),
        "CPU: any modern 4-core or better. A faster CPU barely helps once a \
         GPU is doing the work, so it is not worth spending on for this."
            .to_string(),
        "Disk: about 3-5 GB per model, on an SSD.".to_string(),
        "Integrated graphics (Intel UHD/Iris, AMD Vega) do not qualify. They \
         help against pure CPU but stay far short of usable for agent work."
            .to_string(),
    ]
}

/// Decide whether to offer local inference, and say honestly why not.
///
/// An integrated GPU does not count. It helps a great deal relative to the
/// CPU, and it is still far short of usable for agent-sized prompts, so
/// treating it as "has a GPU" would reintroduce exactly the failure this
/// guards against.
fn assess_local_ai(hardware: &LocalHardware) -> LocalAiCapability {
    let vram_gb = hardware.vram_gb.unwrap_or(0);
    let ram_gb = hardware.ram_gb.unwrap_or(0);

    // Apple Silicon shares one pool of memory between CPU and GPU, so it has
    // no separate VRAM figure and nvidia-smi does not exist there. Judging it
    // by `vram_gb` would reject every Mac, including 32 GB M-series machines
    // that run these models comfortably via Metal. Use unified memory instead.
    let apple_silicon = hardware.os == "macos" && hardware.architecture == "aarch64";
    if apple_silicon {
        let suitable = ram_gb >= 16;
        return LocalAiCapability {
            suitable,
            has_gpu: true,
            vram_gb: ram_gb, // unified: the GPU can address system memory
            ram_gb,
            est_minutes_per_agent_turn: None,
            reason: if suitable {
                format!(
                    "Apple Silicon with {ram_gb} GB unified memory. The GPU shares system \
                     memory here, so local models run well - free and private."
                )
            } else {
                format!(
                    "Apple Silicon with {ram_gb} GB unified memory. That is below the 16 GB \
                     that makes local models comfortable, so local AI stays off by default \
                     and your current model remains the default."
                )
            },
            requirements: if suitable {
                Vec::new()
            } else {
                local_ai_requirements()
            },
            can_proceed_anyway: !suitable,
        };
    }

    // 6 GB is the point where a quantised Gemma 4 fits in VRAM with room for
    // context. Below that the model spills to system RAM and the GPU stops
    // helping, which puts us back in the unusable range.
    let has_gpu = vram_gb >= 6;

    if !has_gpu {
        let minutes =
            (TYPICAL_AGENT_PROMPT_TOKENS / IGPU_PROMPT_TOKENS_PER_SEC / 60.0).round() as u64;
        return LocalAiCapability {
            suitable: false,
            has_gpu: false,
            vram_gb,
            ram_gb,
            est_minutes_per_agent_turn: Some(minutes),
            reason: format!(
                "No discrete GPU was found, so local AI is off by default here. It would \
                 still run - a single agent turn would just take roughly {minutes} minutes \
                 instead of seconds, so the assistant would look configured and rarely \
                 answer in time. This is a hardware limit, not a setting: no model size or \
                 tuning fixes it. Your current model stays the default. You can still set \
                 up local AI if you want it - short chat messages are much faster than \
                 full agent turns, and it runs entirely offline and private."
            ),
            requirements: local_ai_requirements(),
            can_proceed_anyway: true,
        };
    }

    if ram_gb > 0 && ram_gb < 8 {
        return LocalAiCapability {
            suitable: false,
            has_gpu: true,
            vram_gb,
            ram_gb,
            est_minutes_per_agent_turn: None,
            reason: format!(
                "A GPU with {vram_gb} GB VRAM was found, but only {ram_gb} GB of system RAM. \
                 Loading a model would push this machine into swapping and slow everything \
                 down, so local AI is off by default. 16 GB makes it comfortable, 8 GB is \
                 the floor. You can still set it up if you want to try it."
            ),
            requirements: local_ai_requirements(),
            can_proceed_anyway: true,
        };
    }

    LocalAiCapability {
        suitable: true,
        has_gpu: true,
        vram_gb,
        ram_gb,
        est_minutes_per_agent_turn: None,
        reason: format!(
            "A discrete GPU with {vram_gb} GB VRAM was found. This machine can run a local \
             model at usable speed, free and private."
        ),
        requirements: Vec::new(),
        can_proceed_anyway: false,
    }
}

fn recommended_model(hardware: &LocalHardware, purpose: &str) -> &'static str {
    let ram = hardware.ram_gb.unwrap_or(0);
    let vram = hardware.vram_gb.unwrap_or(0);
    if purpose == "coding" {
        return if ram >= 16 || vram >= 8 {
            "qwen2.5-coder:7b"
        } else if ram >= 8 {
            "qwen2.5-coder:3b"
        } else {
            "llama3.2:1b"
        };
    }
    // Sized against MEASURED download/RAM cost, not optimism. A model needs
    // roughly its file size in RAM plus context, so leave headroom for the OS.
    if ram >= 48 || vram >= 24 {
        "gemma4:12b"
    } else if ram >= 24 || vram >= 12 {
        "gemma4:e4b"
    } else if ram >= 12 {
        // Lightest Gemma 4 that exists (7.2 GB); comfortable on a 16 GB machine.
        "gemma4:e2b"
    } else if ram >= 6 {
        "gemma3n:e2b"
    } else {
        "llama3.2:1b"
    }
}

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

/// Model names currently installed in the local Ollama (from `/api/tags`).
/// Returns an empty list if Ollama isn't reachable.
async fn installed_ollama_models() -> Vec<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{OLLAMA_BASE}/api/tags"))
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await;
    let Ok(resp) = resp else { return Vec::new() };
    let Ok(json) = resp.json::<serde_json::Value>().await else {
        return Vec::new();
    };
    json.get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// True if `model` is present in Ollama's installed list, tolerating the
/// implicit `:latest` tag (a pulled `gemma4:e2b` may report as `gemma4:e2b` or
/// `gemma4:e2b:latest`).
fn model_is_installed(model: &str, installed: &[String]) -> bool {
    let norm = |s: &str| s.trim_end_matches(":latest").to_string();
    let target = norm(model);
    installed.iter().any(|m| norm(m) == target)
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

/// GET /api/local-ai/recommendation?purpose=general|coding
///
/// Detects resources only; it never downloads a model or changes configuration.
pub async fn local_ai_recommendation(
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let purpose = match query.get("purpose").map(String::as_str) {
        Some("coding") => "coding",
        _ => "general",
    };
    let hardware = detect_hardware().await;
    let model = recommended_model(&hardware, purpose);
    Json(serde_json::json!({
        "purpose": purpose,
        "recommended_model": model,
        "hardware": hardware,
        "catalog": LOCAL_MODEL_CATALOG,
        "notice": "Recommendations are conservative estimates. Model setup requires explicit confirmation and may use more memory with larger contexts."
    }))
}

/// Top-tier cloud models for hard work, best first. Each entry is
/// `(env var, provider, model)`; the first whose key is configured wins.
const COMPLEX_MODEL_LADDER: &[(&str, &str, &str)] = &[
    ("ANTHROPIC_API_KEY", "anthropic", "claude-sonnet-4-20250514"),
    ("OPENAI_API_KEY", "openai", "gpt-4o"),
    ("GEMINI_API_KEY", "gemini", "gemini-2.5-pro"),
    ("NOVITA_API_KEY", "novita", "zai-org/glm-5.2"),
    ("ZHIPU_API_KEY", "novita", "zai-org/glm-5.2"),
    (
        "OPENROUTER_API_KEY",
        "openrouter",
        "anthropic/claude-sonnet-4",
    ),
    // Groq is last, not a default. Its strongest model is well below the
    // others for the hard work this tier exists to handle, so reaching for it
    // ahead of a key the user already holds would downgrade their results.
    ("GROQ_API_KEY", "groq", "llama-3.3-70b-versatile"),
];

/// Pick the strongest cloud model whose API key is actually configured.
fn detect_complex_model() -> Option<(&'static str, &'static str, &'static str)> {
    COMPLEX_MODEL_LADDER
        .iter()
        .find(|(env, _, _)| {
            std::env::var(env)
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        })
        .copied()
}

/// POST /api/models/autoconfig — one-click model policy: run everyday work on a
/// free, private local Gemma sized to this machine, and register the strongest
/// configured cloud model as the `complex` tier for hard tasks.
///
/// Local-first by default: if no cloud key is configured the complex tier is
/// simply left unset and everything stays local.
pub async fn models_autoconfig(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let home = state.kernel.config.home_dir.clone();
    let hardware = detect_hardware().await;
    let recommended = recommended_model(&hardware, "general");
    let ollama_ready = ollama_running().await;

    // Point the default at a model that is ACTUALLY installed, so agents never
    // get pointed at a model that was never pulled ("no model connected").
    // Prefer the recommended model; if it isn't installed but something else is,
    // use the most capable installed catalogue model; otherwise fall back to the
    // recommendation (which the user can then install).
    let installed = if ollama_ready {
        installed_ollama_models().await
    } else {
        Vec::new()
    };
    let local_model: &str = if !ollama_ready || model_is_installed(recommended, &installed) {
        recommended
    } else {
        // LOCAL_MODEL_CATALOG is ordered smallest → largest; pick the largest
        // installed one for best capability, else keep the recommendation.
        LOCAL_MODEL_CATALOG
            .iter()
            .rev()
            .map(|p| p.id)
            .find(|id| model_is_installed(id, &installed))
            .unwrap_or(recommended)
    };

    // Only point everyday work at the local model if it is ACTUALLY installed.
    // Writing a local default that does not exist replaces a working cloud
    // configuration with a dead endpoint: the user runs "auto-configure", and
    // their assistant immediately stops answering. Local-first must never mean
    // "break what works in the hope that local shows up later".
    // llama.cpp first, Ollama only if it happens to be there already.
    //
    // Ollama was the original local path and it is why local AI kept failing:
    // its registry ignores HTTP Range, so an interrupted pull of a multi-GB
    // model restarts from zero instead of resuming, and a laptop on a normal
    // connection never finishes. llama-server reads plain GGUF files from
    // HuggingFace, which does honour Range, so a download survives being
    // interrupted. It is also the only one of the two that offloads to the
    // integrated GPU here, which is worth roughly 8x on this hardware.
    //
    // So: if a Gemma 4 GGUF is on disk and llama-server is present, point the
    // default there. We never install Ollama to satisfy autoconfig; it is
    // honoured only when the user already has it running.
    let gguf_dir = models_dir(&home);
    let llama_server = find_llama_server(&home);
    let llama_model = LLAMA_GGUF_CATALOG
        .iter()
        .find(|m| gguf_dir.join(m.file_name).is_file());

    let mut local_ready = false;
    let mut local_backend = "none";
    let mut local_label = String::new();

    // Hardware gate comes first. On a machine without a real GPU a local model
    // runs but takes about an hour per agent turn, so making it the default
    // silently breaks the assistant. Keep the working cloud model instead and
    // report why, rather than handing the user something that never answers.
    let capability = assess_local_ai(&hardware);

    if capability.suitable {
        if let (Some(model), Some(server)) = (llama_model, llama_server.as_ref()) {
            // Start it if it is not already answering, so "auto-configure" leaves
            // behind something that actually responds rather than a config file
            // pointing at a port with nothing on it.
            if !llama_server_ready().await {
                let _ = start_llama_server(server, &gguf_dir.join(model.file_name));
                wait_for_llama_server(std::time::Duration::from_secs(30)).await;
            }
            if llama_server_ready().await {
                if let Err(e) = write_default_model_llama(&home, model.id) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": e})),
                    );
                }
                local_ready = true;
                local_backend = "llama.cpp";
                local_label = model.id.to_string();
            }
        }
    }

    // Only point everyday work at the local model if it is ACTUALLY installed.
    // Writing a local default that does not exist replaces a working cloud
    // configuration with a dead endpoint: the user runs "auto-configure", and
    // their assistant immediately stops answering. Local-first must never mean
    // "break what works in the hope that local shows up later".
    if capability.suitable
        && !local_ready
        && ollama_ready
        && model_is_installed(local_model, &installed)
    {
        if let Err(e) = write_default_model(&home, local_model) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e})),
            );
        }
        local_ready = true;
        local_backend = "ollama";
        local_label = local_model.to_string();
    }

    let complex = detect_complex_model();
    if let Some((_, provider, model)) = complex {
        if let Err(e) = write_complex_model(&home, provider, model) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e})),
            );
        }
    }

    // Give every agent the other tier as a fallback. Without this the fallback
    // chain is empty on every agent, so the moment the primary model is
    // unreachable (local server not running, provider outage, context overflow)
    // the request fails outright and the user is told no language model could
    // be reached — while a perfectly good second model sits configured and
    // unused. This is what makes model routing actually resilient.
    let mut routed = 0usize;
    if let Some((env, provider, model)) = complex {
        let chain = vec![openfang_types::agent::FallbackModel {
            provider: provider.to_string(),
            model: model.to_string(),
            api_key_env: Some(env.to_string()),
            base_url: None,
        }];
        for entry in state.kernel.registry.list() {
            if state
                .kernel
                .registry
                .update_fallback_models(entry.id, chain.clone())
                .is_ok()
            {
                // update_fallback_models only mutates the in-memory registry, so
                // without this the chain silently disappears on restart - exactly
                // when a fallback matters most. Persist the updated entry the way
                // set_agent_model does.
                if let Some(updated) = state.kernel.registry.get(entry.id) {
                    let _ = state.kernel.memory.save_agent(&updated);
                }
                routed += 1;
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            // Report what was actually written, not what we hoped to write.
            "default_model": if local_ready {
                serde_json::json!({
                    "provider": if local_backend == "llama.cpp" { "openai" } else { "ollama" },
                    "model": local_label,
                    "backend": local_backend,
                    "local": true
                })
            } else {
                serde_json::json!(null)
            },
            "complex_model": complex.map(|(env, provider, model)| serde_json::json!({
                "provider": provider, "model": model, "api_key_env": env
            })),
            "agents_routed": routed,
            "local_ready": local_ready,
            "ollama_ready": ollama_ready,
            // Always reported, so the UI can explain why local AI is off
            // rather than leaving the user to guess that it silently failed.
            "local_ai_capability": capability,
            "restart_required": true,
            "message": if local_ready {
                "Everyday work now runs on your local model (free and private). Restart FreEco.ai to apply."
            } else if complex.is_some() {
                "Your existing cloud model was kept as the default because no local model is installed yet — \
                 setting a local default now would have left you with nothing that answers. \
                 Install one in Settings > Providers, then run this again."
            } else {
                "No local model is installed and no cloud key is configured, so nothing was changed. \
                 Add a cloud API key or install a local model in Settings > Providers."
            }
        })),
    )
}

/// Write `[complex_model]` in config.toml — the escalation tier for hard tasks.
fn write_complex_model(home: &std::path::Path, provider: &str, model: &str) -> Result<(), String> {
    let path = home.join("config.toml");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut table: toml::Table = existing.parse().unwrap_or_default();

    let mut cm = toml::Table::new();
    cm.insert("provider".into(), toml::Value::String(provider.into()));
    cm.insert("model".into(), toml::Value::String(model.into()));
    table.insert("complex_model".into(), toml::Value::Table(cm));

    let rendered = toml::to_string_pretty(&table).map_err(|e| format!("render config: {e}"))?;
    std::fs::write(&path, rendered).map_err(|e| format!("write config.toml: {e}"))
}

/// POST /api/local-ai/setup — body: optional {"model": "gemma4:e4b"}
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

/// Ollama is installed but not serving? Start it and wait briefly for the API.
/// Returns true once `ollama serve` answers. This avoids the worst failure mode
/// of local-AI setup: re-downloading a ~1 GB installer for software the user
/// already has.
async fn start_installed_ollama(status: &SharedLocalAiStatus) -> bool {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(local) = dirs::data_local_dir() {
        candidates.push(local.join("Programs").join("Ollama").join("ollama.exe"));
        candidates.push(local.join("Programs").join("Ollama").join("ollama app.exe"));
    }
    for p in [
        "/usr/local/bin/ollama",
        "/usr/bin/ollama",
        "/opt/homebrew/bin/ollama",
        "/Applications/Ollama.app/Contents/MacOS/Ollama",
    ] {
        candidates.push(std::path::PathBuf::from(p));
    }

    let exe = candidates.into_iter().find(|p| p.exists());
    let Some(exe) = exe else {
        return false;
    };

    set_status(
        status,
        "starting",
        "Ollama is already installed — starting it...".into(),
        -1,
    )
    .await;

    // `ollama serve` for the CLI binary; the desktop "app" launcher takes no args.
    let is_app = exe
        .file_name()
        .map(|n| n.to_string_lossy().contains("app"))
        .unwrap_or(false);
    let mut cmd = openfang_runtime::quiet_command::quiet_async(&exe);
    if !is_app {
        cmd.arg("serve");
    }
    if cmd.spawn().is_err() {
        return false;
    }

    for _ in 0..20 {
        if ollama_running().await {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    false
}

async fn run_setup(
    status: &SharedLocalAiStatus,
    model: &str,
    config_home: &std::path::Path,
) -> Result<(), String> {
    // 1. Detect
    if !ollama_running().await {
        // Ollama is very often already INSTALLED but simply not running (the
        // user closed it, or the silent installer didn't launch it). Starting
        // it takes a second; re-downloading a ~1 GB installer takes minutes and
        // is what made setup look like it "did nothing". Always try to start
        // what's on disk before downloading anything.
        if start_installed_ollama(status).await {
            // running now — fall through to the model pull
        } else if cfg!(target_os = "windows") {
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
                    "Model download kept failing after {MAX_PULL_ATTEMPTS} attempts: {}. \
                     Finished parts are kept — press Set up again to resume from where it stopped.",
                    explain_network_error(&e)
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
/// Ollama before we run it (threat-model M4). Fails closed unless the status is
/// `Valid` and the signer's subject mentions Ollama.
#[cfg(windows)]
async fn verify_windows_signature(path: &std::path::Path) -> Result<(), String> {
    // Guard first: a missing or empty file is the classic cause of a blank
    // status from Get-AuthenticodeSignature. Report it honestly rather than as
    // an empty "()" so a flaky-download failure is not mistaken for tampering.
    match tokio::fs::metadata(path).await {
        Ok(m) if m.len() == 0 => {
            return Err(
                "downloaded installer is empty (0 bytes) — the download did not \
                        complete. Check your connection and press Set up again."
                    .into(),
            );
        }
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "downloaded installer is missing ({e}) — the download did not complete. \
                 Check your connection and press Set up again."
            ));
        }
    }

    let script = format!(
        "$s = Get-AuthenticodeSignature -LiteralPath '{}'; \
         if ($null -eq $s) {{ Write-Output 'INVALID:NoSignatureObject'; exit 0 }}; \
         if ($s.Status -ne 'Valid') {{ \
             $m = if ($s.StatusMessage) {{ $s.StatusMessage }} else {{ 'no detail' }}; \
             Write-Output ('INVALID:' + $s.Status + ' - ' + $m); exit 0 }}; \
         Write-Output ('OK:' + $s.SignerCertificate.Subject)",
        path.display().to_string().replace('\'', "''")
    );
    let out = openfang_runtime::quiet_command::quiet_async("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .await
        .map_err(|e| format!("could not run signature check: {e}"))?;
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if let Some(subject) = line.strip_prefix("OK:") {
        if subject.to_lowercase().contains("ollama") {
            return Ok(());
        }
        return Err(format!(
            "installer is signed, but not by Ollama (signer: {subject}). Refusing to run it."
        ));
    }
    // Surface a real reason: prefer the parsed INVALID: detail, then any
    // PowerShell stderr, then an explicit "unreadable" note — never an empty ().
    let detail = line
        .strip_prefix("INVALID:")
        .map(str::to_string)
        .unwrap_or_else(|| {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            if !err.is_empty() {
                format!("signature check error: {err}")
            } else if line.is_empty() {
                "status unreadable (installer may be incomplete or corrupt)".into()
            } else {
                line.clone()
            }
        });
    Err(format!(
        "installer failed digital-signature verification ({detail}). It was not run — \
         press Set up again to re-download."
    ))
}

#[cfg(not(windows))]
#[allow(dead_code)]
async fn verify_windows_signature(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

/// Turn a raw network failure into an actionable message.
///
/// Real case that cost hours: on a phone hotspot, Windows preferred IPv6, the
/// model CDN resolved to an IPv6-only address, and every download died with
/// "no such host" — while a perfectly good IPv4 route sat unused. "Check your
/// internet connection" is useless there; the user needs the actual fix.
fn explain_network_error(err: &str) -> String {
    let e = err.to_lowercase();
    let looks_like_dns = e.contains("no such host")
        || e.contains("dns")
        || e.contains("failed to lookup")
        || e.contains("name or service not known");
    if looks_like_dns {
        return format!(
            "{err}\n\nThis usually means your network cannot reach the model server. \
             The most common cause is a mobile hotspot or router where IPv6 is \
             advertised but not actually routable. Fix on Windows (as Administrator):\n\
             \x20   Disable-NetAdapterBinding -Name \"Wi-Fi\" -ComponentID ms_tcpip6\n\
             then press Set up again. Alternatively switch to another network."
        );
    }
    if e.contains("timed out") || e.contains("timeout") {
        return format!(
            "{err}\n\nThe connection is very slow or stalling. Model files are several \
             GB — on a slow link consider a smaller model, or use a USB bundle that \
             already carries the model (see docs/usb-plug-and-play.md)."
        );
    }
    err.to_string()
}

/// Download `url` to `dest`, surviving flaky connections: it resumes with an
/// HTTP Range request after a dropped stream instead of starting over, and it
/// verifies the file on disk matches the server's Content-Length before
/// returning. This is what makes local-AI setup work "on any connection" —
/// a truncated file would otherwise fail signature verification with a blank
/// status downstream.
pub(crate) async fn download_with_resume(
    status: &SharedLocalAiStatus,
    url: &str,
    dest: &std::path::Path,
    label: &str,
) -> Result<(), String> {
    use futures::StreamExt;

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let max_attempts = 8u32;
    let mut last_err = String::from("unknown error");

    // Start clean. A leftover partial/complete file from an earlier failed
    // setup is the classic cause of HTTP 416 (Range Not Satisfiable): we ask to
    // resume from a byte offset that is past the end of the resource. Within a
    // single call we still resume across dropped connections; we just never
    // inherit a stale file from a previous run.
    let _ = tokio::fs::remove_file(dest).await;

    for attempt in 1..=max_attempts {
        let backoff = std::time::Duration::from_secs((attempt as u64).min(5));

        // Bytes already on disk from a previous attempt — resume from here.
        let mut have = tokio::fs::metadata(dest)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        let mut req = client.get(url);
        if have > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={have}-"));
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("connection error: {e}");
                set_status(
                    status,
                    "downloading-installer",
                    format!("{label}: connection issue, retrying ({attempt}/{max_attempts})..."),
                    -1,
                )
                .await;
                tokio::time::sleep(backoff).await;
                continue;
            }
        };

        // 416 Range Not Satisfiable: the local partial file is >= the resource
        // size (stale or already complete). Delete it and restart from byte 0
        // instead of hammering the same out-of-range request until we run out
        // of attempts.
        if resp.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            let _ = tokio::fs::remove_file(dest).await;
            last_err = "stale partial file (HTTP 416) — restarting from scratch".into();
            continue;
        }

        let resp = match resp.error_for_status() {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("HTTP error: {e}");
                set_status(
                    status,
                    "downloading-installer",
                    format!("{label}: server error, retrying ({attempt}/{max_attempts})..."),
                    -1,
                )
                .await;
                tokio::time::sleep(backoff).await;
                continue;
            }
        };

        // If we requested a range but the server ignored it (200, not 206),
        // restart from scratch so we don't append onto a full body.
        let resuming = resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        if have > 0 && !resuming {
            let _ = tokio::fs::remove_file(dest).await;
            have = 0;
        }

        let body_len = resp.content_length().unwrap_or(0);
        let total = if resuming { have + body_len } else { body_len };

        let mut file = match if have > 0 {
            tokio::fs::OpenOptions::new().append(true).open(dest).await
        } else {
            tokio::fs::File::create(dest).await
        } {
            Ok(f) => f,
            Err(e) => return Err(format!("open {label} file: {e}")),
        };

        let mut got = have;
        let mut stream = resp.bytes_stream();
        let mut interrupted = false;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk).await {
                        return Err(format!("write {label}: {e}"));
                    }
                    got += chunk.len() as u64;
                    if total > 0 {
                        let pct = ((got as f64 / total as f64) * 100.0) as i32;
                        set_status(
                            status,
                            "downloading-installer",
                            format!("Downloading {label}... {pct}%"),
                            pct,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    last_err = format!("stream interrupted: {e}");
                    interrupted = true;
                    break;
                }
            }
        }
        let _ = file.flush().await;

        if interrupted {
            set_status(
                status,
                "downloading-installer",
                format!("{label}: connection dropped, resuming ({attempt}/{max_attempts})..."),
                -1,
            )
            .await;
            tokio::time::sleep(backoff).await;
            continue;
        }

        // Completeness check: if the server told us the size, the file must match.
        let on_disk = tokio::fs::metadata(dest)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        if total > 0 && on_disk < total {
            last_err = format!("incomplete download ({on_disk} of {total} bytes)");
            set_status(
                status,
                "downloading-installer",
                format!("{label}: incomplete, resuming ({attempt}/{max_attempts})..."),
                -1,
            )
            .await;
            tokio::time::sleep(backoff).await;
            continue;
        }

        // Sanity floor: a real installer is several MB. If we got a tiny file
        // (e.g. an error page with no Content-Length), retry from scratch.
        if on_disk < 1_000_000 {
            last_err = format!("downloaded file implausibly small ({on_disk} bytes)");
            let _ = tokio::fs::remove_file(dest).await;
            tokio::time::sleep(backoff).await;
            continue;
        }

        return Ok(());
    }

    Err(format!(
        "could not download {label} after {max_attempts} attempts: {}",
        explain_network_error(&last_err)
    ))
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

    // Download robustly: resume on dropped connections and verify completeness
    // before we ever hand the file to the signature check. On a flaky link a
    // truncated installer is what produced the empty "()" verification failure.
    download_with_resume(
        status,
        OLLAMA_WINDOWS_INSTALLER,
        &installer,
        "Ollama installer",
    )
    .await?;

    // SECURITY (threat-model M4): never execute a downloaded installer without
    // verifying it. HTTPS protects transit but not integrity — a compromised
    // mirror/CDN or changed URL could yield arbitrary code execution inside our
    // "safe one-click" flow. Verify the Windows Authenticode signature
    // (publisher = Ollama), which survives Ollama's frequent version bumps
    // better than a pinned hash. Fail closed on anything else.
    set_status(
        status,
        "verifying",
        "Verifying the installer's digital signature...".into(),
        -1,
    )
    .await;
    verify_windows_signature(&installer).await?;

    set_status(
        status,
        "installing",
        "Installing Ollama (silent, per-user, no admin needed)...".into(),
        -1,
    )
    .await;
    let out = openfang_runtime::quiet_command::quiet_async(&installer)
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
                    let _ = openfang_runtime::quiet_command::quiet_async(app).spawn();
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

// ─────────────────────────────────────────────────────────────────────
// llama.cpp local runtime (Ollama-free)
//
// Runs a GGUF model via llama.cpp's `llama-server` — an OpenAI-compatible
// API on localhost. Models are fetched from HuggingFace, which supports
// HTTP Range resume (unlike Ollama's registry, which ignores Range and
// re-downloads from zero on a slow link). Gemma 4 QAT is the default family.
// ─────────────────────────────────────────────────────────────────────

const LLAMA_PORT: u16 = 8080;

/// A GGUF model FreEco can download and run with llama.cpp. Filenames and
/// sizes are verified against the HuggingFace repos.
#[derive(Debug, Clone, Serialize)]
pub struct GgufModel {
    pub id: &'static str,
    pub display_name: &'static str,
    pub purpose: &'static str,
    pub url: &'static str,
    pub file_name: &'static str,
    pub download_gb: f32,
    pub min_ram_gb: u64,
}

pub const LLAMA_GGUF_CATALOG: &[GgufModel] = &[
    GgufModel {
        id: "gemma-4-e2b-qat-q4",
        display_name: "Gemma 4 E2B — QAT Q4 (recommended)",
        purpose: "Everyday assistant; best quality-for-size on ordinary laptops.",
        url: "https://huggingface.co/unsloth/gemma-4-E2B-it-qat-GGUF/resolve/main/gemma-4-E2B-it-qat-UD-Q4_K_XL.gguf",
        file_name: "gemma-4-E2B-it-qat-UD-Q4_K_XL.gguf",
        download_gb: 2.5,
        min_ram_gb: 8,
    },
    GgufModel {
        id: "gemma-4-e2b-qat-q2",
        display_name: "Gemma 4 E2B — QAT Q2 (smallest)",
        purpose: "Lowest RAM / fastest download; slightly lower quality.",
        url: "https://huggingface.co/unsloth/gemma-4-E2B-it-qat-GGUF/resolve/main/gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf",
        file_name: "gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf",
        download_gb: 2.0,
        min_ram_gb: 6,
    },
    GgufModel {
        id: "gemma-4-e4b-qat-q2",
        display_name: "Gemma 4 E4B — QAT Q2 (bigger brain)",
        purpose: "More capable; needs more RAM.",
        url: "https://huggingface.co/unsloth/gemma-4-E4B-it-qat-GGUF/resolve/main/gemma-4-E4B-it-qat-UD-Q2_K_XL.gguf",
        file_name: "gemma-4-E4B-it-qat-UD-Q2_K_XL.gguf",
        download_gb: 3.0,
        min_ram_gb: 12,
    },
];

fn gguf_catalog_entry(id: &str) -> Option<&'static GgufModel> {
    LLAMA_GGUF_CATALOG.iter().find(|m| m.id == id)
}

/// Where downloaded GGUF models live (on the same disk as config — the USB
/// when FreEco runs from one). Overridable later for the "install to USB vs
/// folder" choice.
fn models_dir(home: &std::path::Path) -> std::path::PathBuf {
    home.join("models")
}

/// Locate the `llama-server` binary.
///
/// `llama-vk` is checked before `llamacpp` on purpose: it holds the Vulkan
/// build, which offloads to the integrated GPU and is roughly 8x faster than
/// the CPU-only build on the hardware this targets. Preferring it is the
/// difference between a local model that is usable and one that is not, so
/// when both are installed the fast one wins.
///
/// Returns None if nothing is found (the caller reports how to get it).
fn find_llama_server(home: &std::path::Path) -> Option<std::path::PathBuf> {
    let exe = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };
    for dir in ["llama-vk", "llamacpp"] {
        let candidate = home.join(dir).join(exe);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    // Fall back to PATH.
    which_on_path(exe)
}

fn which_on_path(exe: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(exe);
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}

/// Spawn `llama-server` serving `gguf` on 127.0.0.1:LLAMA_PORT. The child is
/// detached (survives this request); a std Child dropped is NOT killed.
fn start_llama_server(server: &std::path::Path, gguf: &std::path::Path) -> Result<(), String> {
    use std::process::{Command, Stdio};
    let log = gguf
        .parent()
        .map(|p| p.join("llama-server.log"))
        .and_then(|p| std::fs::File::create(p).ok());
    let (out, err) = match log {
        Some(f) => (
            Stdio::from(f.try_clone().map_err(|e| e.to_string())?),
            Stdio::from(f),
        ),
        None => (Stdio::null(), Stdio::null()),
    };
    Command::new(server)
        .arg("-m")
        .arg(gguf)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(LLAMA_PORT.to_string())
        .arg("-c")
        .arg("4096")
        .stdout(out)
        .stderr(err)
        .spawn()
        .map(|_child| ()) // detached — do not wait, do not kill on drop
        .map_err(|e| format!("could not start llama-server: {e}"))
}

/// True once llama-server answers on its OpenAI-compatible endpoint.
async fn llama_server_ready() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();
    let Ok(client) = client else { return false };
    client
        .get(format!("http://127.0.0.1:{LLAMA_PORT}/v1/models"))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Poll until llama-server answers or `budget` expires.
///
/// Loading a multi-gigabyte GGUF into memory takes many seconds on a spinning
/// laptop, so a single probe right after spawn always reports failure and the
/// caller would wrongly conclude the server never came up.
async fn wait_for_llama_server(budget: std::time::Duration) -> bool {
    let deadline = std::time::Instant::now() + budget;
    while std::time::Instant::now() < deadline {
        if llama_server_ready().await {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;
    }
    false
}

/// Point `[default_model]` at the local llama-server (OpenAI-compatible).
fn write_default_model_llama(home: &std::path::Path, model_id: &str) -> Result<(), String> {
    let path = home.join("config.toml");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut table: toml::Table = existing.parse().unwrap_or_default();
    let mut dm = toml::Table::new();
    dm.insert("provider".into(), toml::Value::String("openai".into()));
    dm.insert("model".into(), toml::Value::String(model_id.into()));
    dm.insert("api_key_env".into(), toml::Value::String(String::new()));
    dm.insert(
        "base_url".into(),
        toml::Value::String(format!("http://127.0.0.1:{LLAMA_PORT}/v1")),
    );
    table.insert("default_model".into(), toml::Value::Table(dm));
    let rendered = toml::to_string_pretty(&table).map_err(|e| format!("render config: {e}"))?;
    std::fs::write(&path, rendered).map_err(|e| format!("write config.toml: {e}"))
}

/// GET /api/local-ai/llama/catalog — Gemma 4 GGUF options + hardware + which
/// are already downloaded + whether the llama-server binary is present.
pub async fn llama_catalog(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let home = &state.kernel.config.home_dir;
    let dir = models_dir(home);
    let hardware = detect_hardware().await;
    let server_present = find_llama_server(home).is_some();
    let items: Vec<serde_json::Value> = LLAMA_GGUF_CATALOG
        .iter()
        .map(|m| {
            let downloaded = dir.join(m.file_name).exists();
            serde_json::json!({
                "id": m.id,
                "display_name": m.display_name,
                "purpose": m.purpose,
                "download_gb": m.download_gb,
                "min_ram_gb": m.min_ram_gb,
                "downloaded": downloaded,
                "runnable": hardware.ram_gb.unwrap_or(0) >= m.min_ram_gb,
            })
        })
        .collect();
    Json(serde_json::json!({
        "engine": "llama.cpp",
        "server_present": server_present,
        // The picker must warn BEFORE a multi-gigabyte download, not after.
        // Downloading 2.5 GB onto a machine that cannot run the result at
        // usable speed wastes the user's bandwidth and their time.
        "capability": assess_local_ai(&hardware),
        "hardware": hardware,
        "models": items,
    }))
}

/// POST /api/local-ai/llama/setup { "model_id": "..." } — download the GGUF
/// (resumable), start llama-server, and point default_model at it. Progress
/// is reported through the shared local-AI status (GET /api/local-ai/status).
pub async fn llama_setup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = req
        .get("model_id")
        .and_then(|v| v.as_str())
        .unwrap_or("gemma-4-e2b-qat-q4")
        .to_string();
    let Some(model) = gguf_catalog_entry(&id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("unknown model '{id}'")})),
        );
    };

    {
        let mut s = state.local_ai.write().await;
        if s.running {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "a local-AI setup is already running"})),
            );
        }
        s.running = true;
        s.phase = "checking".into();
        s.detail = format!("Preparing {}...", model.display_name);
        s.percent = -1;
        s.model = model.file_name.to_string();
    }

    let status = state.local_ai.clone();
    let home = state.kernel.config.home_dir.clone();
    tokio::spawn(async move {
        let result = provision_llama(&status, model, &home).await;
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
        Json(serde_json::json!({"status": "started", "model": model.id})),
    )
}

async fn provision_llama(
    status: &SharedLocalAiStatus,
    model: &'static GgufModel,
    home: &std::path::Path,
) -> Result<(), String> {
    // 1. Need the runtime binary.
    let Some(server) = find_llama_server(home) else {
        return Err(format!(
            "llama-server is not installed. Put it in {} (from \
             https://github.com/ggml-org/llama.cpp/releases) and try again.",
            home.join("llamacpp").display()
        ));
    };

    // 2. Download the GGUF with resume (HuggingFace supports Range).
    let dir = models_dir(home);
    std::fs::create_dir_all(&dir).map_err(|e| format!("models dir: {e}"))?;
    let gguf = dir.join(model.file_name);
    download_with_resume(status, model.url, &gguf, model.display_name).await?;

    // 3. Start the server.
    set_status(
        status,
        "starting",
        format!(
            "Starting {} on 127.0.0.1:{LLAMA_PORT}...",
            model.display_name
        ),
        -1,
    )
    .await;
    start_llama_server(&server, &gguf)?;

    // 4. Wait for it to answer, then wire it as the default model.
    let client = reqwest::Client::new();
    let health = format!("http://127.0.0.1:{LLAMA_PORT}/health");
    let mut up = false;
    for _ in 0..60 {
        if client.get(&health).send().await.is_ok() {
            up = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    if !up {
        return Err("llama-server started but did not become healthy within 2 minutes".into());
    }
    write_default_model_llama(home, model.id)?;
    set_status(
        status,
        "done",
        format!(
            "{} is running locally and set as your model. No Ollama, no cloud.",
            model.display_name
        ),
        100,
    )
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gguf_catalog_is_gemma4_only() {
        assert!(!LLAMA_GGUF_CATALOG.is_empty());
        for m in LLAMA_GGUF_CATALOG {
            assert!(m.id.starts_with("gemma-4"), "only Gemma 4 in the catalog");
            assert!(m.url.contains("huggingface.co"), "resumable HF source");
            assert!(m.url.ends_with(".gguf"));
        }
        assert!(gguf_catalog_entry("gemma-4-e2b-qat-q4").is_some());
        assert!(gguf_catalog_entry("nope").is_none());
    }

    #[test]
    fn default_model_written_to_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "api_listen = \"127.0.0.1:4200\"\n",
        )
        .unwrap();
        write_default_model(dir.path(), "gemma4:e4b").unwrap();
        let out = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(out.contains("provider = \"ollama\""));
        assert!(out.contains("gemma4:e4b"));
        // Pre-existing keys survive the rewrite.
        assert!(out.contains("api_listen"));
    }

    #[test]
    fn model_name_validation_pattern() {
        for good in ["gemma4:e4b", "llama3.2:1b", "qwen2.5-coder:7b"] {
            assert!(good
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c)));
        }
        assert!(!"bad name; rm -rf"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c)));
    }

    fn hw(ram_gb: u64, vram_gb: Option<u64>) -> LocalHardware {
        LocalHardware {
            os: "windows".into(),
            architecture: "x86_64".into(),
            ram_gb: Some(ram_gb),
            vram_gb,
            free_disk_gb: Some(100),
            ollama_detected: false,
            docker_detected: false,
        }
    }

    /// The user's actual machine: 8 GB RAM, Intel UHD 620. nvidia-smi finds
    /// nothing, so vram is None. Measured ~5 tok/s -- about an hour per agent
    /// turn. Local AI must stay OFF here; enabling it produces an assistant
    /// that looks configured and never answers.
    #[test]
    fn igpu_machine_is_not_suitable_for_local_ai() {
        let cap = assess_local_ai(&hw(8, None));
        assert!(!cap.suitable);
        assert!(!cap.has_gpu);
        // The warning must carry a concrete number, not a vague "may be slow".
        let minutes = cap.est_minutes_per_agent_turn.expect("must estimate");
        assert!(
            minutes >= 30,
            "estimate should be honest, got {minutes} min"
        );
    }

    /// Being told "no" without being told "what would work" is a dead end.
    /// Every unsuitable verdict must carry concrete requirements and must
    /// still leave local setup reachable.
    #[test]
    fn unsuitable_verdicts_explain_what_is_needed() {
        for machine in [hw(8, None), hw(16, Some(2)), hw(4, Some(12))] {
            let cap = assess_local_ai(&machine);
            assert!(!cap.suitable);
            assert!(cap.can_proceed_anyway, "local setup must stay available");
            assert!(!cap.requirements.is_empty(), "must say what is needed");
            let text = cap.requirements.join(" ");
            assert!(text.contains("VRAM"), "must state GPU/VRAM needs");
            assert!(text.contains("RAM"), "must state RAM needs");
            assert!(text.contains("CPU"), "must state CPU needs");
        }
    }

    /// Apple Silicon has no separate VRAM and no nvidia-smi, so judging it by
    /// `vram_gb` would reject every Mac -- including 32 GB M-series machines
    /// that run these models comfortably through Metal.
    #[test]
    fn apple_silicon_judged_on_unified_memory() {
        let mut m2 = hw(32, None);
        m2.os = "macos".into();
        m2.architecture = "aarch64".into();
        let cap = assess_local_ai(&m2);
        assert!(cap.suitable, "32 GB Apple Silicon must not be rejected");
        assert!(cap.has_gpu);

        let mut small = hw(8, None);
        small.os = "macos".into();
        small.architecture = "aarch64".into();
        assert!(!assess_local_ai(&small).suitable);
    }

    /// An integrated GPU reporting a little shared memory must not be mistaken
    /// for a real one; below the VRAM floor the model spills to system RAM and
    /// the GPU stops helping.
    #[test]
    fn small_vram_does_not_count_as_a_gpu() {
        assert!(!assess_local_ai(&hw(16, Some(2))).suitable);
        assert!(!assess_local_ai(&hw(16, Some(5))).suitable);
    }

    #[test]
    fn discrete_gpu_is_suitable() {
        let cap = assess_local_ai(&hw(16, Some(8)));
        assert!(cap.suitable);
        assert!(cap.has_gpu);
        assert!(cap.est_minutes_per_agent_turn.is_none());
    }

    /// A GPU alone is not enough: too little system RAM means swapping.
    #[test]
    fn gpu_with_too_little_ram_is_not_suitable() {
        assert!(!assess_local_ai(&hw(4, Some(12))).suitable);
    }

    #[test]
    fn recommends_conservative_general_models() {
        let low = LocalHardware {
            os: "linux".into(),
            architecture: "x86_64".into(),
            ram_gb: Some(4),
            vram_gb: None,
            free_disk_gb: Some(20),
            ollama_detected: false,
            docker_detected: false,
        };
        let ordinary = LocalHardware {
            ram_gb: Some(16),
            ..low.clone()
        };
        let powerful = LocalHardware {
            ram_gb: Some(32),
            ..low.clone()
        };
        let workstation = LocalHardware {
            ram_gb: Some(48),
            ..low.clone()
        };
        let small = LocalHardware {
            ram_gb: Some(8),
            ..low.clone()
        };
        // The ladder is sized against MEASURED model cost, so each rung stays
        // comfortably within RAM rather than recommending an oversized model.
        assert_eq!(recommended_model(&low, "general"), "llama3.2:1b");
        assert_eq!(recommended_model(&small, "general"), "gemma3n:e2b");
        assert_eq!(recommended_model(&ordinary, "general"), "gemma4:e2b");
        assert_eq!(recommended_model(&powerful, "general"), "gemma4:e4b");
        assert_eq!(recommended_model(&workstation, "general"), "gemma4:12b");
    }

    #[test]
    fn recommends_qwen_for_coding_when_resources_allow() {
        let hardware = LocalHardware {
            os: "linux".into(),
            architecture: "x86_64".into(),
            ram_gb: Some(16),
            vram_gb: None,
            free_disk_gb: Some(20),
            ollama_detected: false,
            docker_detected: false,
        };
        assert_eq!(recommended_model(&hardware, "coding"), "qwen2.5-coder:7b");
    }
}
