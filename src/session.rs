use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{Error, Result};

/// Configuration for a Copilot chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// A human-readable label for the session (used in logging).
    pub label: String,
    /// Maximum number of tokens to generate per response.
    pub max_tokens: Option<u32>,
    /// System prompt / context injected at the start of every conversation.
    pub system_prompt: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            label: String::from("default"),
            max_tokens: None,
            system_prompt: None,
        }
    }
}

/// Options that can be passed to a single [`Session::send_and_collect`] call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendOptions {
    /// Optional context/extra instructions for this specific message only.
    pub context: Option<String>,
}

/// A single conversation session with the Copilot CLI.
///
/// Obtain one via [`crate::Client::create_session`].
pub struct Session {
    pub(crate) config: SessionConfig,
    pub(crate) history: Arc<Mutex<Vec<Message>>>,
    pub(crate) process: Arc<crate::process::CopilotProcess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Message {
    pub role: String,
    pub content: String,
}

impl Session {
    /// Send a message and collect the complete response as a `String`.
    ///
    /// `options` can provide per-request overrides; pass `None` to use defaults.
    pub async fn send_and_collect(
        &self,
        message: &str,
        options: Option<SendOptions>,
    ) -> Result<String> {
        let mut history = self.history.lock().await;

        if let Some(ref prompt) = self.config.system_prompt {
            if history.is_empty() {
                history.push(Message {
                    role: "system".to_string(),
                    content: prompt.clone(),
                });
            }
        }

        history.push(Message {
            role: "user".to_string(),
            content: message.to_string(),
        });

        let request = ChatRequest {
            session_label: self.config.label.clone(),
            messages: history.clone(),
            max_tokens: self.config.max_tokens,
            context: options.and_then(|o| o.context),
        };

        let response = self.process.send_request(request).await?;

        history.push(Message {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        Ok(response)
    }
}

// ---------------------------------------------------------------------------
// Internal wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub(crate) struct ChatRequest {
    pub session_label: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponse {
    pub content: Option<String>,
    pub error: Option<String>,
}

impl ChatResponse {
    pub fn into_result(self) -> Result<String> {
        if let Some(err) = self.error {
            return Err(Error::CliError(err));
        }
        Ok(self.content.unwrap_or_default())
    }
}
