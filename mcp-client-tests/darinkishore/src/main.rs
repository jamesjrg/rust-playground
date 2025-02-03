use anyhow::Result;
use mcp_client_rs::client::ClientBuilder;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    #[arg(help = "Runtime to use (node or python)")]
    runtime: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = match cli.runtime.as_str() {
        "node" => ClientBuilder::new("npx")
            .args(["-y", "@executeautomation/playwright-mcp-server"])
            .spawn_and_initialize().await?,
        "python" => ClientBuilder::new("uvx")
            .arg("mcp-server-time")
            .spawn_and_initialize().await?,
        _ => anyhow::bail!("Invalid runtime. Must be 'node' or 'python'"),
    };

    let tools = client.list_tools().await.unwrap();
    println!("Tools: {:?}", tools.tools);

    Ok(())
}
