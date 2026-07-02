#[cfg(feature = "batch-scrape")]
pub mod batch_scrape;
#[cfg(feature = "batch-scrape")]
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
#[cfg(feature = "batch-scrape")]
use rmcp::model::ContentBlock;
#[cfg(feature = "scrape")]
pub use scrape::{SCRAPE_TOOL_NAME, get_firecrawl_scrape};
#[cfg(feature = "search")]
pub mod search;
#[cfg(feature = "search")]
pub use search::{SEARCH_TOOL_NAME, get_firecrawl_search};

use firecrawl_sdk::FirecrawlApp;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams,
        ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
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
    Arc::from(vec![
        #[cfg(feature = "batch-scrape")]
        {
            let batch_scrape_tool = get_firecrawl_batch_scrape().unwrap();
            Tool::new_with_raw(
                batch_scrape_tool.name.clone(),
                batch_scrape_tool.description.clone(),
                Arc::new(
                    batch_scrape_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            )
        },
        #[cfg(feature = "crawl")]
        {
            let crawl_tool = get_firecrawl_crawl().unwrap();
            Tool::new_with_raw(
                crawl_tool.name.clone(),
                crawl_tool.description.clone(),
                Arc::new(
                    crawl_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            )
        },
        #[cfg(feature = "map")]
        {
            let map_tool = get_firecrawl_map().unwrap();
            Tool::new_with_raw(
                map_tool.name.clone(),
                map_tool.description.clone(),
                Arc::new(
                    map_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            )
        },
        #[cfg(feature = "scrape")]
        {
            let scrape_tool = get_firecrawl_scrape().unwrap();
            Tool::new_with_raw(
                scrape_tool.name.clone(),
                scrape_tool.description.clone(),
                Arc::new(
                    scrape_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            )
        },
        #[cfg(feature = "search")]
        {
            let search_tool = get_firecrawl_search().unwrap();
            Tool::new_with_raw(
                search_tool.name.clone(),
                search_tool.description.clone(),
                Arc::new(
                    search_tool
                        .input_schema
                        .as_object()
                        .expect("Tool schema must be an object")
                        .clone(),
                ),
            )
        },
    ])
});

#[derive(Clone)]
pub struct FirecrawlMCP {
    pub client: FirecrawlApp,
}

impl FirecrawlMCP {
    pub fn new(api_key: impl AsRef<str>, client: reqwest::Client) -> Self {
        Self {
            client: FirecrawlApp::new_with_client(api_key, client).unwrap(),
        }
    }

    pub fn new_with_app(app: FirecrawlApp) -> Self {
        Self { client: app }
    }

    pub fn new_selfhosted(
        api_url: impl AsRef<str>,
        api_key: Option<impl AsRef<str>>,
        client: reqwest::Client,
    ) -> Self {
        Self {
            client: FirecrawlApp::new_selfhosted_with_client(api_url, api_key, client).unwrap(),
        }
    }
}

impl ServerHandler for FirecrawlMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "This server provides tools to crawl, scrape, and search the web using Firecrawl.",
            )
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name;
        let params = request.arguments.unwrap();

        match tool_name.as_ref() {
            #[cfg(feature = "batch-scrape")]
            BATCH_SCRAPE_TOOL_NAME => match self.batch_scrape(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![ContentBlock::text(result)])),
                Err(err) => {
                    error!("Batch scraping URLs failed: {}", err);
                    Err(err)
                }
            },
            #[cfg(feature = "crawl")]
            CRAWL_TOOL_NAME => match self.crawl(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![ContentBlock::text(result)])),
                Err(err) => Err(err),
            },
            #[cfg(feature = "map")]
            MAP_TOOL_NAME => match self.map(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![ContentBlock::text(result)])),
                Err(err) => Err(err),
            },
            #[cfg(feature = "scrape")]
            SCRAPE_TOOL_NAME => match self.scrape(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![ContentBlock::text(result)])),
                Err(err) => Err(err),
            },
            #[cfg(feature = "search")]
            SEARCH_TOOL_NAME => match self.search(params).await {
                Ok(result) => Ok(CallToolResult::success(vec![ContentBlock::text(result)])),
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
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(Vec::from(TOOLS.as_ref())))
    }
}
