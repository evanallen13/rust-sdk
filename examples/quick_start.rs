//! Quick-start example.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example quick_start
//! ```
//!
//! Make sure the `copilot` CLI is in your PATH (or set `COPILOT_CLI_PATH`),
//! and that you have authenticated with `gh auth login`.

use copilot_sdk::{Client, SessionConfig};

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let client = Client::builder().build()?;
    client.start().await?;

    let session = client.create_session(SessionConfig::default()).await?;
    let response = session.send_and_collect("Hello!", None).await?;
    println!("{}", response);

    client.stop().await;
    Ok(())
}
