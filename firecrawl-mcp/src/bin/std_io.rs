use anyhow::Result;
use firecrawl_mcp::FirecrawlMCP;
use rmcp::{ServiceExt, transport::stdio};
use std::env;
use tracing::{error, info};
use tracing_subscriber::{self, EnvFilter};

/// npx @modelcontextprotocol/inspector cargo run -p mcp-server-examples --example std_io
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting MCP server");

    // Get API key from environment variable
    let api_key =
        env::var("FIRECRAWL_API_KEY").expect("FIRECRAWL_API_KEY environment variable must be set");

    // Create a Controller instance
    let controller = FirecrawlMCP::new(api_key);

    // Create the service with our controller using stdio transport
    let service = controller.serve(stdio()).await.inspect_err(|e| {
        error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
