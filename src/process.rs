use std::sync::Arc;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    sync::Mutex,
};
use tracing::{debug, warn};

use crate::{
    session::{ChatRequest, ChatResponse},
    Error, Result,
};

/// Wraps the long-running Copilot CLI subprocess and handles newline-delimited
/// JSON communication over its stdin/stdout.
pub(crate) struct CopilotProcess {
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<BufReader<ChildStdout>>,
    // Keep the Child handle alive so the process is not reaped.
    _child: Mutex<Child>,
}

impl CopilotProcess {
    pub fn new(mut child: Child) -> Result<Arc<Self>> {
        let stdin = child.stdin.take().ok_or(Error::ProcessExited)?;
        let stdout = child.stdout.take().ok_or(Error::ProcessExited)?;

        Ok(Arc::new(Self {
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(BufReader::new(stdout)),
            _child: Mutex::new(child),
        }))
    }

    /// Send a JSON request line and read back a JSON response line.
    pub async fn send_request(&self, request: ChatRequest) -> Result<String> {
        let line = serde_json::to_string(&request)? + "\n";
        debug!(request = %line.trim(), "sending request to Copilot CLI");

        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(line.as_bytes()).await?;
            stdin.flush().await?;
        }

        let mut response_line = String::new();
        {
            let mut stdout = self.stdout.lock().await;
            let n = stdout.read_line(&mut response_line).await?;
            if n == 0 {
                return Err(Error::ProcessExited);
            }
        }

        debug!(response = %response_line.trim(), "received response from Copilot CLI");

        let chat_response: ChatResponse = serde_json::from_str(response_line.trim())?;
        chat_response.into_result()
    }

    /// Attempt a graceful shutdown by closing stdin.
    pub async fn shutdown(&self) {
        // Dropping stdin causes the process to receive EOF and exit on its own.
        let mut stdin = self.stdin.lock().await;
        if let Err(e) = stdin.shutdown().await {
            warn!("error shutting down Copilot CLI stdin: {e}");
        }
    }
}
