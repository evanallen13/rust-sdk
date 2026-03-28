use thiserror::Error;

/// Errors that can occur when using the Copilot SDK.
#[derive(Debug, Error)]
pub enum Error {
    /// The Copilot CLI executable could not be found.
    #[error("Copilot CLI not found: set COPILOT_CLI_PATH or ensure 'copilot' is in PATH")]
    CliNotFound,

    /// Failed to spawn the Copilot CLI process.
    #[error("Failed to spawn Copilot CLI process: {0}")]
    SpawnFailed(#[source] std::io::Error),

    /// I/O error communicating with the Copilot CLI process.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The CLI process exited unexpectedly.
    #[error("Copilot CLI process exited unexpectedly")]
    ProcessExited,

    /// Failed to serialize or deserialize a message.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// The CLI returned an error response.
    #[error("Copilot CLI error: {0}")]
    CliError(String),

    /// The client has not been started yet.
    #[error("Client is not running — call client.start() first")]
    NotStarted,

    /// An operation timed out.
    #[error("Operation timed out")]
    Timeout,
}
