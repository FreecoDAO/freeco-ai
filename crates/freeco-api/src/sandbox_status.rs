//! Sandbox visibility.
//!
//! The Docker sandbox is the boundary that keeps agent-written code off the
//! user's machine, and until now it was completely invisible: no endpoint, no
//! panel, no way to tell whether it was on, whether Docker was running, or
//! whether the image was even downloaded. A security control nobody can see is
//! one nobody can trust, and when it silently was not working there was no way
//! to find out except by reading the source.
//!
//! This exposes the whole truth in one call: is it enabled, is Docker up, is
//! the image present, and what limits are actually applied.

use crate::routes::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

/// GET /api/sandbox/status
///
/// Reports what is true right now rather than what is configured. `ready` is
/// the single field the UI needs: it is only true when the sandbox would
/// actually run something, which requires all three of enabled, Docker
/// running, and the image already downloaded.
pub async fn sandbox_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = &state.kernel.config.docker;

    let docker_running = freeco_kernel_runtime::docker_sandbox::is_docker_available().await;
    // Only probe the image when Docker answers; otherwise the inspect call
    // just times out and we would report "image missing" for the wrong reason.
    let image_present = if docker_running {
        freeco_kernel_runtime::docker_sandbox::is_image_present(&config.image).await
    } else {
        false
    };

    let ready = config.enabled && docker_running && image_present;

    // Say what to do next, in the order it has to be done. A status panel that
    // only reports "not ready" leaves the user guessing which of three things
    // is wrong.
    let next_step = if !config.enabled {
        Some("Sandbox is switched off. Set docker.enabled = true in ~/.freeco-ai/config.toml.")
    } else if !docker_running {
        Some("Docker is not running. Start Docker Desktop, or install it from https://docker.com/products/docker-desktop.")
    } else if !image_present {
        Some("The sandbox image has not been downloaded yet. It is a one-time download, and it is not started automatically so it cannot surprise a metered connection.")
    } else {
        None
    };

    Json(serde_json::json!({
        "enabled": config.enabled,
        "docker_running": docker_running,
        "image": config.image,
        "image_present": image_present,
        "ready": ready,
        "next_step": next_step,
        // The protections actually in force, so the user can see the boundary
        // rather than take it on faith.
        "protections": {
            "network": config.network,
            "network_blocked": config.network == "none",
            "memory_limit": config.memory_limit,
            "cpu_limit": config.cpu_limit,
            "pids_limit": config.pids_limit,
            "read_only_root": config.read_only_root,
            "timeout_secs": config.timeout_secs,
            "capabilities_dropped": "ALL",
            "no_new_privileges": true,
        },
    }))
}

/// POST /api/sandbox/pull — download the sandbox image on request.
///
/// Deliberately explicit. `docker run` would pull the image silently on first
/// use, which on a metered connection is an unannounced several-hundred-
/// megabyte download. Making it a button the user presses keeps that decision
/// theirs.
pub async fn sandbox_pull(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let image = state.kernel.config.docker.image.clone();

    if !freeco_kernel_runtime::docker_sandbox::is_docker_available().await {
        return Json(serde_json::json!({
            "ok": false,
            "error": "Docker is not running, so the image cannot be downloaded. Start Docker Desktop and try again.",
        }));
    }

    match freeco_kernel_runtime::quiet_command::quiet_async("docker")
        .arg("pull")
        .arg(&image)
        .output()
        .await
    {
        Ok(out) if out.status.success() => Json(serde_json::json!({
            "ok": true,
            "image": image,
            "message": format!("{image} is downloaded. The sandbox is ready."),
        })),
        Ok(out) => Json(serde_json::json!({
            "ok": false,
            "error": String::from_utf8_lossy(&out.stderr).trim().to_string(),
        })),
        Err(e) => Json(serde_json::json!({
            "ok": false,
            "error": format!("could not run docker pull: {e}"),
        })),
    }
}
