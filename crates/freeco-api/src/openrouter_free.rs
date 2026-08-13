//! Free models on OpenRouter, fetched live.
//!
//! A new install should be able to hold a conversation before the user has
//! decided anything about providers. OpenRouter publishes models priced at
//! zero, which makes that possible with the user's own key and no spend.
//!
//! The list is fetched rather than hardcoded, deliberately. Which models are
//! free changes week to week: entries appear, are withdrawn, get renamed, and
//! change their rate limits. A baked-in list is correct on the day it ships
//! and quietly wrong afterwards, and a first-run experience that recommends a
//! model that no longer exists is worse than one that recommends nothing.
//!
//! No key is required to read the catalogue, so this works before the user has
//! signed up and can populate the picker they are choosing from.

use crate::routes::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

/// The default model for a fresh install.
///
/// `openrouter/free` is OpenRouter's own router across every zero-cost model:
/// one id, 200k context, priced at zero, and they keep it pointed at whatever
/// is currently available. That is the whole reason to use it -- the set of
/// free models changes weekly, and this moves that maintenance burden to the
/// people who already track it.
///
/// The alternative was shipping a list of individual `:free` ids, which is
/// correct on the day it ships and quietly wrong afterwards.
pub const DEFAULT_FREE_MODEL: &str = "openrouter/free";

/// How the user gets a key.
///
/// Shipping one inside the product was considered and rejected. FreEco.ai is
/// open source, so a bundled key is extractable from the binary or the
/// repository in seconds. Every install would then share one rate limit -
/// exhausted quickly, at which point nobody's first run works - and the key's
/// owner would carry the cost and the liability for whatever anyone did with
/// it. It would be revoked, correctly, as soon as the provider noticed one key
/// serving thousands of installs.
///
/// The user's own free key takes about a minute to obtain and has none of
/// those properties.
pub const KEY_SIGNUP_URL: &str = "https://openrouter.ai/keys";

/// GET /api/models/free — models currently priced at zero on OpenRouter.
///
/// Returns an empty list with an explanation rather than an error when the
/// network is unavailable: a first-run screen that cannot reach the internet
/// should say so plainly, not present a failure the user cannot act on.
pub async fn list_free_models(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return offline(format!("could not build HTTP client: {e}")),
    };

    let resp = match client.get(OPENROUTER_MODELS_URL).send().await {
        Ok(r) => r,
        Err(e) => {
            return offline(format!(
                "could not reach OpenRouter ({e}). This needs internet access once, \
                 to list which models are currently free."
            ))
        }
    };

    if !resp.status().is_success() {
        return offline(format!("OpenRouter returned HTTP {}", resp.status()));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return offline(format!("could not read the model list: {e}")),
    };

    // Every model, each flagged free or paid, rather than only the free ones.
    //
    // This is what a model picker needs, and it removes any argument for a
    // hand-maintained list of "top-tier" models or a nightly job to refresh
    // one. OpenRouter already tracks every vendor's current line-up; a list
    // baked into this repository would be a second copy that goes stale
    // between releases and has to be noticed by a person.
    let mut free: Vec<serde_json::Value> = body["data"]
        .as_array()
        .map(|models| {
            models
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m["id"].as_str().unwrap_or(""),
                        "name": m["name"].as_str().unwrap_or(""),
                        "context_length": m["context_length"].as_u64().unwrap_or(0),
                        "free": is_free(m),
                        "description": truncate(m["description"].as_str().unwrap_or(""), 220),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Largest context first. For an assistant that has to hold a working
    // conversation, context is the difference between usable and not, and it
    // is the one attribute a newcomer can compare without knowing the models.
    free.sort_by_key(|m| {
        (
            // Free first: someone opening this for the first time should meet
            // the models that cost nothing before the ones that do.
            !m["free"].as_bool().unwrap_or(false),
            // Then widest context, the one attribute a newcomer can compare
            // without already knowing the models.
            std::cmp::Reverse(m["context_length"].as_u64().unwrap_or(0)),
        )
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "provider": "openrouter",
            "recommended": DEFAULT_FREE_MODEL,
            "models": free,
            "free_count": free.iter().filter(|m| m["free"].as_bool().unwrap_or(false)).count(),
            "signup_url": KEY_SIGNUP_URL,
            "note": "These models are free to use with your own OpenRouter key. \
                     The key takes about a minute to create and is stored on this \
                     machine only. FreEco.ai does not ship a shared key: it is open \
                     source, so a bundled key would be extractable by anyone and \
                     every install would share one rate limit.",
        })),
    )
}

/// A model is free when every price component is zero.
///
/// Checked across all components rather than just the prompt price: a model
/// with free prompts and paid completions is not free, and presenting it as
/// such on a first-run screen would produce a surprise bill on the first real
/// answer.
fn is_free(model: &serde_json::Value) -> bool {
    let pricing = &model["pricing"];
    if !pricing.is_object() {
        return false;
    }
    let zero = |key: &str| -> bool {
        match &pricing[key] {
            serde_json::Value::String(s) => s.parse::<f64>().map(|v| v == 0.0).unwrap_or(false),
            serde_json::Value::Number(n) => n.as_f64().map(|v| v == 0.0).unwrap_or(false),
            // Absent means the provider does not charge for it.
            serde_json::Value::Null => true,
            _ => false,
        }
    };
    // Only the two components OpenRouter actually returns. An earlier version
    // also required `request` to be zero; that key is never present in their
    // response, so the check relied on absent-means-free and added a failure
    // mode without adding safety. Both of these must be zero, because a model
    // that is free to send to and paid to answer is not free, and presenting
    // it as such produces a surprise bill on the first real reply.
    ["prompt", "completion"].iter().all(|k| zero(k))
}

/// The best free model that will actually accept tool definitions.
///
/// `openrouter/free` lists `tools` among its supported parameters, but the
/// endpoints it routes to do not serve them: a request carrying tools comes
/// back as "No endpoints found that support tool use", both with two tools and
/// with sixty-five. Named free models accept the identical payload. Agents
/// always send tools, so the router is the wrong default for them however
/// convenient its single stable id is — an agent pointed at it cannot answer at
/// all, which is indistinguishable from the product being broken.
///
/// The choice is made from OpenRouter's live model list rather than written
/// down here. Free models are added and delisted constantly; a hardcoded id is
/// correct on the day it ships and quietly dead a few weeks later, with nobody
/// watching. Widest context wins, because that is the one property a newcomer
/// benefits from without having to know anything about the models.
///
/// Returns `None` if the list cannot be fetched, so callers can fall back
/// rather than pretend a guess is an answer.
pub async fn best_free_tool_model() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .ok()?;
    let body: serde_json::Value = client
        .get(OPENROUTER_MODELS_URL)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    let mut candidates: Vec<(u64, String)> = body["data"]
        .as_array()?
        .iter()
        .filter(|m| is_free(m))
        .filter(|m| {
            m["supported_parameters"]
                .as_array()
                .is_some_and(|ps| ps.iter().any(|p| p.as_str() == Some("tools")))
        })
        .filter_map(|m| {
            Some((
                m["context_length"].as_u64().unwrap_or(0),
                m["id"].as_str()?.to_string(),
            ))
        })
        .collect();

    // Widest context first; id as a tiebreak so the choice is stable between
    // calls rather than depending on the order the API happened to return.
    candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    candidates.into_iter().next().map(|(_, id)| id)
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{cut}...")
}

fn offline(reason: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "provider": "openrouter",
            "models": [],
            "signup_url": KEY_SIGNUP_URL,
            "offline": true,
            "note": reason,
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(prompt: &str, completion: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "vendor/model:free",
            "pricing": { "prompt": prompt, "completion": completion }
        })
    }

    /// The router that a fresh install defaults to. Verified against the live
    /// API: id `openrouter/free`, 200k context, prompt and completion both "0".
    #[test]
    fn the_default_router_is_recognised_as_free() {
        let router = serde_json::json!({
            "id": "openrouter/free",
            "name": "Free Models Router",
            "context_length": 200000,
            "pricing": { "prompt": "0", "completion": "0" }
        });
        assert!(is_free(&router));
        assert_eq!(DEFAULT_FREE_MODEL, "openrouter/free");
    }

    /// The paid auto-router must not be mistaken for the free one. It prices
    /// at "-1", meaning "varies", and defaulting a new user onto it would
    /// charge them on their first message.
    #[test]
    fn the_paid_auto_router_is_not_free() {
        let auto = serde_json::json!({
            "id": "openrouter/auto",
            "pricing": { "prompt": "-1", "completion": "-1" }
        });
        assert!(!is_free(&auto));
    }

    #[test]
    fn a_model_priced_at_zero_is_free() {
        assert!(is_free(&model("0", "0")));
    }

    /// The case that would produce a surprise bill: free to send, paid to
    /// answer. Checking only the prompt price would let this through onto a
    /// screen labelled "free".
    #[test]
    fn free_prompts_with_paid_completions_is_not_free() {
        assert!(!is_free(&model("0", "0.0000015")));
        assert!(!is_free(&model("0.0000005", "0")));
    }

    #[test]
    fn a_priced_model_is_not_free() {
        assert!(!is_free(&model("0.000003", "0.000015")));
    }

    /// Absent components mean the provider does not charge for them, which is
    /// different from a missing pricing object entirely.
    #[test]
    fn absent_components_count_as_free_but_absent_pricing_does_not() {
        assert!(is_free(
            &serde_json::json!({ "pricing": { "prompt": "0" } })
        ));
        assert!(!is_free(&serde_json::json!({ "id": "x" })));
    }

    /// Numeric zero is as valid as string zero; the API has used both.
    #[test]
    fn numeric_and_string_zero_are_both_accepted() {
        assert!(is_free(
            &serde_json::json!({ "pricing": { "prompt": 0, "completion": 0 } })
        ));
    }

    #[test]
    fn descriptions_are_truncated_without_panicking_on_unicode() {
        let text = "π".repeat(400);
        assert!(truncate(&text, 220).chars().count() <= 223);
        assert_eq!(truncate("short", 220), "short");
    }
}
