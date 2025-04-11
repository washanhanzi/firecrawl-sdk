use anyhow::Result;
use firecrawl_mcp::FirecrawlMCP;
use firecrawl_sdk::FirecrawlApp;
use rmcp::transport::sse_server::SseServer;
use std::env;
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting MCP server with SSE transport");

    // Get API key from environment variable
    let api_key =
        env::var("FIRECRAWL_API_KEY").expect("FIRECRAWL_API_KEY environment variable must be set");

    // Get port from env or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    // Create a Controller instance
    let controller = FirecrawlMCP::new(api_key);

    // Create the bind address
    let bind_address = format!("0.0.0.0:{}", port);
    info!("Listening on {}", bind_address);

    // Start the SSE server with our controller
    let server_handle = SseServer::serve(bind_address.parse()?)
        .await?
        .with_service(move || controller.clone());

    // Wait for Ctrl+C to shutdown
    tokio::signal::ctrl_c().await?;
    server_handle.cancel();
    Ok(())
}
