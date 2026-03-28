//! # copilot-sdk
//!
//! A Rust SDK for interacting with the [GitHub Copilot CLI](https://cli.github.com/).
//!
//! ## Requirements
//!
//! * Rust 1.85+ (Edition 2024)
//! * The `copilot` binary available in `PATH`, **or** set the `COPILOT_CLI_PATH`
//!   environment variable to the full path of the Copilot CLI executable.
//!
//! ## Quick Start
//!
//! ```no_run
//! use copilot_sdk::{Client, SessionConfig};
//!
//! #[tokio::main]
//! async fn main() -> copilot_sdk::Result<()> {
//!     let client = Client::builder().build()?;
//!     client.start().await?;
//!
//!     let session = client.create_session(SessionConfig::default()).await?;
//!     let response = session.send_and_collect("Hello!", None).await?;
//!     println!("{}", response);
//!
//!     client.stop().await;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod session;

mod process;

pub use client::{Client, ClientBuilder};
pub use error::Error;
pub use session::{SendOptions, Session, SessionConfig};

/// Convenience `Result` type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
