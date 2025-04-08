#[cfg(feature = "batch_scrape")]
pub mod batch_scrape;
#[cfg(feature = "crawl")]
pub mod crawl;
#[cfg(feature = "map")]
pub mod map;
#[cfg(feature = "scrape")]
pub mod scrape;
#[cfg(feature = "search")]
pub mod search;

use batch_scrape::get_firecrawl_batch_scrape;
use crawl::get_firecrawl_crawl;
use firecrawl_sdk::FirecrawlApp;
use map::get_firecrawl_map;
use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
};
use scrape::get_firecrawl_scrape;
use search::get_firecrawl_search;
use std::sync::{Arc, LazyLock};
use tracing::error;

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
pub struct Controller {
    pub client: FirecrawlApp,
}

impl Controller {
    pub fn new(client: FirecrawlApp) -> Self {
        Self { client }
    }
}

impl ServerHandler for Controller {
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
            "firecrawl_batch_scrape" => match self.batch_scrape(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                Err(err) => {
                    error!("Batch scraping URLs failed: {}", err);
                    Err(err)
                }
            },
            #[cfg(feature = "crawl")]
            "firecrawl_crawl" => {
                match self.crawl(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // crawl now returns rmcp::Error
                }
            }
            #[cfg(feature = "map")]
            "firecrawl_map" => {
                match self.map(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // map now returns rmcp::Error
                }
            }
            #[cfg(feature = "scrape")]
            "firecrawl_scrape" => {
                match self.scrape(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // scrape already returns rmcp::Error
                }
            }
            #[cfg(feature = "search")]
            "firecrawl_search" => match self.search(params).await {
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
            tools: Vec::from(TOOLS.as_ref().clone()),
            next_cursor: None,
        })
    }
}
