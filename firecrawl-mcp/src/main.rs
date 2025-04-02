use anyhow::Result;
use axum::http::StatusCode;
use dotenv::dotenv;
use tracing::{error, info};

fn main() -> Result<()> {
    println!("Hello, world!");
    Ok(())
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     // Load environment variables
//     dotenv().ok();

//     // Initialize logging
//     tracing_subscriber::fmt::init();

//     // Get Firecrawl API key
//     let api_key = std::env::var("FIRECRAWL_API_KEY").expect("FIRECRAWL_API_KEY must be set");

//     // Get port from env or use default
//     let port = std::env::var("PORT")
//         .unwrap_or_else(|_| "3000".to_string())
//         .parse()
//         .expect("PORT must be a valid number");

//     // Run the server
//     run_server(api_key, port).await
// }

// async fn handle_mcp_request(
//     State(state): State<AppState>,
//     Json(request): Json<MCPRequest>,
// ) -> Result<Json<MCPResponse>, (StatusCode, Json<MCPResponse>)> {
//     info!("Received request for tool: {}", request.tool_name);

//     let result = match request.tool_name.as_str() {
//         "scrape" => state.controller.scrape(request.parameters).await,
//         "search" => state.controller.search(request.parameters).await,
//         "crawl" => state.controller.crawl(request.parameters).await,
//         _ => Err(anyhow::anyhow!(
//             "Unsupported operation: {}",
//             request.tool_name
//         )),
//     };

//     match result {
//         Ok(text) => {
//             let response = MCPResponse {
//                 content: vec![MCPContent {
//                     content_type: "text".to_string(),
//                     text,
//                 }],
//                 is_error: false,
//             };
//             Ok(Json(response))
//         }
//         Err(err) => {
//             error!("Error handling request: {}", err);
//             let error_response = MCPResponse {
//                 content: vec![MCPContent {
//                     content_type: "text".to_string(),
//                     text: format!("Error: {}", err),
//                 }],
//                 is_error: true,
//             };
//             Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
//         }
//     }
// }
