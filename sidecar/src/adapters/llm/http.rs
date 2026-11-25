//! Structured output over HTTP, for an OAuth subscription proxy container or a
//! plain API key: Anthropic Messages (`output_config.format`) or OpenAI Chat
//! Completions (`response_format` json_schema). No tools are sent: an HTTP run
//! never browses.
use super::{HttpConfig, ToolAccess, classify, prompt, schema};
use crate::*;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Api {
    /// `POST {base_url}/messages`, header `x-api-key`.
    Anthropic,
    /// `POST {base_url}/chat/completions`, header `Authorization: Bearer`.
    Openai,
}

pub struct HttpRuntime {
    config: HttpConfig,
    client: reqwest::Client,
}
impl HttpRuntime {
    pub fn new(config: HttpConfig) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client");
        Self { config, client }
    }
}

/// The request body for `api`.
pub fn body(api: Api, model: &str, max_tokens: u32, prompt: &str, schema: &Value) -> Value {
    let messages = json!([{"role": "user", "content": prompt}]);
    match api {
        Api::Anthropic => json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
            "output_config": {"format": {"type": "json_schema", "schema": schema}}
        }),
        Api::Openai => json!({
            "model": model,
            "max_completion_tokens": max_tokens,
            "messages": messages,
            "response_format": {"type": "json_schema", "json_schema": {
                "name": "kallbet_answer", "strict": true, "schema": schema
            }}
        }),
    }
}

/// Classifies a response by status and body; a 2xx body yields the answer.
pub fn parse(api: Api, status: u16, body: &str, schema: &Value) -> Result<Value, LlmError> {
    if let Some(e) = classify::status(status) {
        // A spend cap or plan quota can come back as 400/403 with a quota message.
        return Err(match classify::text(body) {
            Some(LlmError::QuotaExceeded) => LlmError::QuotaExceeded,
            _ => e,
        });
    }
    let v: Value = serde_json::from_str(body).map_err(|_| LlmError::InvalidOutput)?;
    let text = match api {
        Api::Anthropic => {
            if v["stop_reason"] != "end_turn" && v["stop_reason"] != "stop_sequence" {
                return Err(LlmError::InvalidOutput);
            }
            v["content"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect::<String>()
        }
        Api::Openai => {
            let choice = &v["choices"][0];
            if !choice["message"]["refusal"].is_null() || choice["finish_reason"] != "stop" {
                return Err(LlmError::InvalidOutput);
            }
            choice["message"]["content"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        }
    };
    match classify::json_text(&text) {
        Some(v) if schema::valid(&v, schema) => Ok(v),
        _ => Err(LlmError::InvalidOutput),
    }
}

#[async_trait]
impl LlmRuntime for HttpRuntime {
    async fn evaluate(
        &self,
        _: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        role: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        let Some(base) = &self.config.base_url else {
            return Err(LlmError::NotConfigured);
        };
        let key = match &self.config.api_key_env {
            Some(var) => Some(std::env::var(var).map_err(|_| LlmError::NotConfigured)?),
            None => None,
        };
        let text = prompt::render(dossier, tier, role, ToolAccess::None);
        let api = self.config.api;
        let path = match api {
            Api::Anthropic => "messages",
            Api::Openai => "chat/completions",
        };
        let url = format!("{}/{path}", base.trim_end_matches('/'));
        let mut request = self.client.post(url).json(&body(
            api,
            &self.config.model,
            self.config.max_tokens,
            &text,
            &role.schema,
        ));
        request = match (api, key) {
            (Api::Anthropic, key) => {
                let r = request.header("anthropic-version", "2023-06-01");
                match key {
                    Some(k) => r.header("x-api-key", k),
                    None => r,
                }
            }
            (Api::Openai, Some(k)) => request.bearer_auth(k),
            (Api::Openai, None) => request,
        };
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                LlmError::Timeout
            } else {
                LlmError::Transient
            }
        })?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|e| {
            if e.is_timeout() {
                LlmError::Timeout
            } else {
                LlmError::Transient
            }
        })?;
        parse(api, status, &body, &role.schema).map(Some)
    }
}
