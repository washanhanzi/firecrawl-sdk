use anyhow::Result;
use firecrawl_mcp::FirecrawlMCP;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting MCP server with Streamable HTTP transport");

    let api_key =
        env::var("FIRECRAWL_API_KEY").expect("FIRECRAWL_API_KEY environment variable must be set");

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let bind_addr: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .expect("invalid bind address");

    let client = reqwest::Client::new();
    let api_key_for_factory = api_key.clone();
    let client_for_factory = client.clone();

    let cancel = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .disable_allowed_hosts()
        .with_cancellation_token(cancel.clone());

    let session_manager = Arc::new(LocalSessionManager::default());
    let service: StreamableHttpService<FirecrawlMCP, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(FirecrawlMCP::new(&api_key_for_factory, client_for_factory.clone())),
            session_manager,
            config,
        );

    let router = axum::Router::new().nest_service("/mcp", service);

    let tcp_listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!(
        "Listening on http://{}/mcp",
        tcp_listener.local_addr().expect("local addr")
    );

    axum::serve(
        tcp_listener,
        router,
    )
    .with_graceful_shutdown(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl_c");
        cancel.cancel();
    })
    .await?;

    Ok(())
}
