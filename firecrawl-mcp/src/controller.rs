pub mod batch_scrape;
pub mod crawl;
pub mod map;
pub mod scrape;
pub mod search;

use batch_scrape::get_firecrawl_batch_scrape;
use crawl::get_firecrawl_crawl;
use firecrawl_sdk::FirecrawlApp;
use map::get_firecrawl_map;
use once_cell::sync::Lazy;
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    Error as McpError, RoleServer, ServerHandler,
};
use scrape::get_firecrawl_scrape;
use search::get_firecrawl_search;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::error;

// Define static tools with Clone implementation
#[derive(Clone)]
struct ToolsContainer {
    tools: Vec<Tool>,
}

static TOOLS: Lazy<ToolsContainer> = Lazy::new(|| ToolsContainer {
    tools: vec![
        {
            let batch_scrape_tool = get_firecrawl_batch_scrape().unwrap();
            Tool {
                name: Cow::Owned(batch_scrape_tool.name.clone()),
                description: Cow::Owned(batch_scrape_tool.description.clone().unwrap_or_default()),
                input_schema: Arc::new(
                    batch_scrape_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        {
            let crawl_tool = get_firecrawl_crawl().unwrap();
            Tool {
                name: Cow::Owned(crawl_tool.name.clone()),
                description: Cow::Owned(crawl_tool.description.clone().unwrap_or_default()),
                input_schema: Arc::new(
                    crawl_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        {
            let map_tool = get_firecrawl_map().unwrap();
            Tool {
                name: Cow::Owned(map_tool.name.clone()),
                description: Cow::Owned(map_tool.description.clone().unwrap_or_default()),
                input_schema: Arc::new(
                    map_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        {
            let scrape_tool = get_firecrawl_scrape().unwrap();
            Tool {
                name: Cow::Owned(scrape_tool.name.clone()),
                description: Cow::Owned(scrape_tool.description.clone().unwrap_or_default()),
                input_schema: Arc::new(
                    scrape_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
        {
            let search_tool = get_firecrawl_search().unwrap();
            Tool {
                name: Cow::Owned(search_tool.name.clone()),
                description: Cow::Owned(search_tool.description.clone().unwrap_or_default()),
                input_schema: Arc::new(
                    search_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            }
        },
    ],
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
            "firecrawl_batch_scrape" => match self.batch_scrape(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                Err(err) => {
                    error!("Batch scraping URLs failed: {}", err);
                    Err(err)
                }
            },
            "firecrawl_crawl" => {
                match self.crawl(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // crawl now returns rmcp::Error
                }
            }
            "firecrawl_map" => {
                match self.map(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // map now returns rmcp::Error
                }
            }
            "firecrawl_scrape" => {
                match self.scrape(params).await {
                    Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
                    Err(err) => Err(err), // scrape already returns rmcp::Error
                }
            }
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
        // Simply return a clone of the static tools
        // The clone is efficient because:
        // 1. We're using Cow for strings which avoids cloning static strings
        // 2. We're using Arc for input_schema which only clones the pointer
        // 3. The ToolsContainer has a proper Clone implementation
        Ok(ListToolsResult {
            tools: TOOLS.clone().tools,
            next_cursor: None,
        })
    }
}
