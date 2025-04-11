#[cfg(feature = "batch_scrape")]
pub mod batch_scrape;
#[cfg(feature = "batch_scrape")]
pub use batch_scrape::{BATCH_SCRAPE_TOOL_NAME, get_firecrawl_batch_scrape};
#[cfg(feature = "crawl")]
pub mod crawl;
#[cfg(feature = "crawl")]
pub use crawl::{CRAWL_TOOL_NAME, get_firecrawl_crawl};
#[cfg(feature = "map")]
pub mod map;
#[cfg(feature = "map")]
pub use map::{MAP_TOOL_NAME, get_firecrawl_map};
#[cfg(feature = "scrape")]
pub mod scrape;
#[cfg(feature = "scrape")]
pub use scrape::{SCRAPE_TOOL_NAME, get_firecrawl_scrape};
#[cfg(feature = "search")]
pub mod search;
#[cfg(feature = "search")]
pub use search::{SEARCH_TOOL_NAME, get_firecrawl_search};

use firecrawl_sdk::FirecrawlApp;
use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
};
use std::sync::{Arc, LazyLock};
use tracing::error;

/// Extension trait to convert FirecrawlApp into FirecrawlMCP
pub trait IntoFirecrawlMCP {
    /// Converts a FirecrawlApp instance into a FirecrawlMCP instance
    fn into_mcp(self) -> FirecrawlMCP;
}

impl IntoFirecrawlMCP for FirecrawlApp {
    fn into_mcp(self) -> FirecrawlMCP {
        FirecrawlMCP::new_with_app(self)
    }
}

// Define the static tools using Arc<[Tool]> to avoid cloning
pub static TOOLS: LazyLock<Arc<[Tool]>> = LazyLock::new(|| {
    // Create a Vec and then convert it to Arc<[Tool]>
    Arc::from(vec![
        #[cfg(feature = "batch_scrape")]
        {
            let batch_scrape_tool = get_firecrawl_batch_scrape().unwrap();
            Tool {
                name: batch_scrape_tool.name.clone(),
                description: batch_scrape_tool.description.clone().unwrap_or_default(),
                input_schema: Arc::new(
                    batch_scrape_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        #[cfg(feature = "crawl")]
        {
            let crawl_tool = get_firecrawl_crawl().unwrap();
            Tool {
                name: crawl_tool.name.clone(),
                description: crawl_tool.description.clone().unwrap_or_default(),
                input_schema: Arc::new(
                    crawl_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        #[cfg(feature = "map")]
        {
            let map_tool = get_firecrawl_map().unwrap();
            Tool {
                name: map_tool.name.clone(),
                description: map_tool.description.clone().unwrap_or_default(),
                input_schema: Arc::new(
                    map_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        #[cfg(feature = "scrape")]
        {
            let scrape_tool = get_firecrawl_scrape().unwrap();
            Tool {
                name: scrape_tool.name.clone(),
                description: scrape_tool.description.clone().unwrap_or_default(),
                input_schema: Arc::new(
                    scrape_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        #[cfg(feature = "search")]
        {
            let search_tool = get_firecrawl_search().unwrap();
            Tool {
                name: search_tool.name.clone(),
                description: search_tool.description.clone().unwrap_or_default(),
                input_schema: Arc::new(
                    search_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
    ])
});

#[derive(Clone)]
pub struct FirecrawlMCP {
    pub client: FirecrawlApp,
}

impl FirecrawlMCP {
    pub fn new(api_key: impl AsRef<str>) -> Self {
        Self {
            client: FirecrawlApp::new(api_key).unwrap(),
        }
    }

    pub fn new_with_app(app: FirecrawlApp) -> Self {
        Self { client: app }
    }

    pub fn new_with_client(api_key: impl AsRef<str>, client: reqwest::Client) -> Self {
        Self {
            client: FirecrawlApp::new_with_client(api_key, client).unwrap(),
        }
    }
}

impl ServerHandler for FirecrawlMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides tools to crawl, scrape, and search the web using Firecrawl."
                    .to_string(),
            ),
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name;
        let params = request.arguments.unwrap();

        match tool_name.as_ref() {
            #[cfg(feature = "batch_scrape")]
            BATCH_SCRAPE_TOOL_NAME => match self.batch_scrape(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                Err(err) => {
                    error!("Batch scraping URLs failed: {}", err);
                    Err(err)
                }
            },
            #[cfg(feature = "crawl")]
            CRAWL_TOOL_NAME => {
                match self.crawl(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // crawl now returns rmcp::Error
                }
            }
            #[cfg(feature = "map")]
            MAP_TOOL_NAME => {
                match self.map(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // map now returns rmcp::Error
                }
            }
            #[cfg(feature = "scrape")]
            SCRAPE_TOOL_NAME => {
                match self.scrape(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // scrape already returns rmcp::Error
                }
            }
            #[cfg(feature = "search")]
            SEARCH_TOOL_NAME => match self.search(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                Err(err) => Err(McpError::internal_error(
                    format!("Search error: {}", err),
                    None,
                )),
            },
            _ => Err(McpError::invalid_request(
                format!("Tool not found: {}", tool_name),
                None,
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        // Just clone the Arc pointer, not the actual tools
        Ok(ListToolsResult {
            tools: Vec::from(TOOLS.as_ref()),
            next_cursor: None,
        })
    }
}
