//! HTTP client for the Koa AI server (https://github.com/xiaoyu-work/koa).
//!
//! Koa is an external "AI chief of staff" service with its own ReAct
//! orchestrator and ~30 specialized agents (email, calendar, smart home, ...).
//! Instead of calling an LLM provider directly we delegate the whole turn to
//! Koa, which decides which agents/models to use and returns a synthesized
//! response.
//!
//! API contract (Koa `koa/server/routes/chat.py`, `koa/server/models.py`):
//!
//! ```text
//! POST {base_url}/chat
//! Headers: X-API-Key: <api_key>   (optional in dev mode)
//!          Content-Type: application/json
//! Body:    {
//!            "message": "...",
//!            "tenant_id": "...",
//!            "conversation_history": [{"role": "user"|"assistant"|"system",
//!                                      "content": "..."}],
//!            "images": [{"data": "<base64>", "media_type": "image/jpeg"}]
//!          }
//! Resp:    { "response": "...", "status": "completed", ... }
//! ```

use std::time::Duration;

use reqwest::Client as HttpClient;
use rig::completion::message::{AssistantContent, Message, UserContent};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

/// Default tenant identifier sent to Koa when none is configured.
const DEFAULT_TENANT_ID: &str = "pin";

/// HTTP timeout for a single Koa /chat call.
///
/// Koa runs a ReAct loop that may call tools (Gmail, Calendar, ...) which can
/// be slow. 120s mirrors the user-facing patience budget of the Pin's UI.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Client wrapping a Koa /chat endpoint.
#[derive(Clone)]
pub struct KoaClient {
    base_url: String,
    api_key: Option<String>,
    tenant_id: String,
    http: HttpClient,
}

impl KoaClient {
    /// Build a new Koa client.
    ///
    /// `base_url` must point at the Koa server root (no trailing `/chat`),
    /// e.g. `http://192.168.1.10:8000`.
    pub fn new(base_url: String, api_key: Option<String>, tenant_id: Option<String>) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let tenant_id = tenant_id
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| DEFAULT_TENANT_ID.to_string());

        let http = HttpClient::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("reqwest client builder should not fail with default config");

        info!(base_url = %base_url, tenant_id = %tenant_id, has_api_key = api_key.is_some(),
            "Koa client ready");

        Self {
            base_url,
            api_key,
            tenant_id,
            http,
        }
    }

    /// Send a chat turn to Koa, optionally with prior conversation history
    /// and an inline base64-encoded JPEG image (vision).
    pub async fn chat(
        &self,
        utterance: &str,
        history: &[Message],
        image_base64: Option<&str>,
    ) -> Result<String, String> {
        let url = format!("{}/chat", self.base_url);

        let history_json = messages_to_json(history);
        debug!(
            url = %url,
            history_len = history_json.len(),
            has_image = image_base64.is_some(),
            "POST Koa /chat"
        );

        let mut body = json!({
            "message": utterance,
            "tenant_id": self.tenant_id,
            "conversation_history": history_json,
        });

        if let Some(b64) = image_base64 {
            body["images"] = json!([{
                "data": b64,
                "media_type": "image/jpeg",
            }]);
        }

        let mut req = self.http.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp = req.send().await.map_err(|e| {
            error!(error = %e, url = %url, "Koa request failed");
            friendly_network_error(&e)
        })?;

        let status = resp.status();
        let bytes = resp.bytes().await.map_err(|e| {
            error!(error = %e, "failed to read Koa response body");
            "I couldn't read the response from Koa. Please try again.".to_string()
        })?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&bytes);
            error!(status = %status, body = %body_str, "Koa returned non-2xx");
            return Err(friendly_http_error(status.as_u16()));
        }

        let parsed: ChatResponse = serde_json::from_slice(&bytes).map_err(|e| {
            let body_str = String::from_utf8_lossy(&bytes);
            error!(error = %e, body = %body_str, "failed to parse Koa response JSON");
            "Koa returned an unexpected response. Please try again.".to_string()
        })?;

        if parsed.response.trim().is_empty() {
            return Err("Koa returned an empty response.".to_string());
        }

        Ok(parsed.response)
    }
}

/// Koa `/chat` JSON response body. We only need `response`; everything else
/// is metadata the Pin doesn't use directly.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
    #[serde(default)]
    #[allow(dead_code)]
    status: String,
}

/// Convert rig [`Message`] history into the `[{role, content}]` shape Koa
/// expects in `conversation_history`. Mirrors `db::messages_to_pairs` but
/// emits `serde_json::Value` rows.
fn messages_to_json(history: &[Message]) -> Vec<Value> {
    history
        .iter()
        .filter_map(|m| match m {
            Message::User { content } => {
                let text: String = content
                    .iter()
                    .filter_map(|p| match p {
                        UserContent::Text(t) => Some(t.text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                (!text.is_empty()).then(|| json!({ "role": "user", "content": text }))
            }
            Message::Assistant { content, .. } => {
                let text: String = content
                    .iter()
                    .filter_map(|p| match p {
                        AssistantContent::Text(t) => Some(t.text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                (!text.is_empty()).then(|| json!({ "role": "assistant", "content": text }))
            }
            Message::System { content } => (!content.is_empty())
                .then(|| json!({ "role": "system", "content": content.clone() })),
        })
        .collect()
}

fn friendly_network_error(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "Koa took too long to respond. Please try again.".into()
    } else if e.is_connect() {
        "I couldn't reach the Koa server. Please check that it's running.".into()
    } else {
        "Something went wrong talking to Koa. Please try again.".into()
    }
}

fn friendly_http_error(status: u16) -> String {
    match status {
        401 | 403 => "Koa rejected the API key. Please check the server settings.".into(),
        404 => "Koa's chat endpoint wasn't found at the configured URL.".into(),
        429 => "Koa is rate-limiting requests right now. Please try again in a moment.".into(),
        500..=599 => "Koa hit an internal error. Please try again shortly.".into(),
        _ => "Koa returned an unexpected error. Please try again.".into(),
    }
}
