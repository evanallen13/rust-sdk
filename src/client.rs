use std::{env, path::PathBuf, sync::Arc};

use tokio::{
    process::Command,
    sync::{Mutex, RwLock},
};
use tracing::info;

use crate::{
    process::CopilotProcess,
    session::{Session, SessionConfig},
    Error, Result,
};

// ---------------------------------------------------------------------------
// ClientBuilder
// ---------------------------------------------------------------------------

/// Builder for a [`Client`].
///
/// Obtain one via [`Client::builder`].
pub struct ClientBuilder {
    cli_path: Option<PathBuf>,
}

impl ClientBuilder {
    fn new() -> Self {
        Self { cli_path: None }
    }

    /// Override the path to the Copilot CLI executable.
    ///
    /// If not set, the builder checks the `COPILOT_CLI_PATH` environment
    /// variable and then falls back to looking for `copilot` in `PATH`.
    pub fn cli_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.cli_path = Some(path.into());
        self
    }

    /// Build the [`Client`].
    ///
    /// This resolves the CLI path but does **not** start the process yet —
    /// call [`Client::start`] to do that.
    pub fn build(self) -> Result<Client> {
        let cli_path = self
            .cli_path
            .or_else(|| env::var("COPILOT_CLI_PATH").ok().map(PathBuf::from))
            .or_else(which_copilot)
            .ok_or(Error::CliNotFound)?;

        Ok(Client {
            cli_path,
            process: Arc::new(RwLock::new(None)),
        })
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// The main entry point for the Copilot SDK.
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> copilot_sdk::Result<()> {
/// use copilot_sdk::{Client, SessionConfig};
///
/// let client = Client::builder().build()?;
/// client.start().await?;
///
/// let session = client.create_session(SessionConfig::default()).await?;
/// let response = session.send_and_collect("Hello!", None).await?;
/// println!("{}", response);
///
/// client.stop().await;
/// # Ok(())
/// # }
/// ```
pub struct Client {
    cli_path: PathBuf,
    process: Arc<RwLock<Option<Arc<CopilotProcess>>>>,
}

impl Client {
    /// Return a new [`ClientBuilder`].
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Start the Copilot CLI subprocess.
    ///
    /// Must be called before [`Client::create_session`].
    pub async fn start(&self) -> Result<()> {
        info!(cli = %self.cli_path.display(), "starting Copilot CLI");

        let child = Command::new(&self.cli_path)
            .arg("--sdk-mode")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(Error::SpawnFailed)?;

        let proc = CopilotProcess::new(child)?;

        let mut guard = self.process.write().await;
        *guard = Some(proc);

        info!("Copilot CLI started");
        Ok(())
    }

    /// Stop the Copilot CLI subprocess gracefully.
    pub async fn stop(&self) {
        let mut guard = self.process.write().await;
        if let Some(proc) = guard.take() {
            proc.shutdown().await;
            info!("Copilot CLI stopped");
        }
    }

    /// Create a new chat [`Session`] using the given configuration.
    pub async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        let proc = {
            let guard = self.process.read().await;
            guard.as_ref().cloned().ok_or(Error::NotStarted)?
        };

        Ok(Session {
            config,
            history: Arc::new(Mutex::new(Vec::new())),
            process: proc,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Look for `copilot` in `PATH`, checking that the file is executable.
fn which_copilot() -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    env::var_os("PATH").and_then(|path_var| {
        env::split_paths(&path_var).find_map(|dir| {
            let candidate = dir.join("copilot");
            let is_executable = candidate
                .metadata()
                .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);
            if is_executable { Some(candidate) } else { None }
        })
    })
}
