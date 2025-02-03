use anyhow::Result;
use mcp_sdk::{client::ClientBuilder, transport::ClientStdioTransport};
use mcp_sdk::transport::Transport;
use mcp_sdk::protocol::RequestOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let transport = ClientStdioTransport::new("cat", &[])?;

    // Open transport
    transport.open()?;

    let client = ClientBuilder::new(transport).build();
    let client_clone = client.clone();
    tokio::spawn(async move { client_clone.start().await });
    let response = client
        .request(
            "echo",
            None,
            RequestOptions::default().timeout(Duration::from_secs(1)),
        )
        .await?;
    println!("{:?}", response);

    Ok(())
}
