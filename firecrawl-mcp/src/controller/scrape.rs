use anyhow::Result;
use async_claude::define_tool;
use firecrawl_sdk::scrape::{ScrapeFormats, ScrapeUrlInput};
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};

use super::FirecrawlMCP;

pub const SCRAPE_TOOL_NAME: &str = "firecrawl_scrape";
pub const SCRAPE_TOOL_DESCRIPTION: &str = "Scrape a single webpage with advanced options for content extraction. Supports various formats including markdown, HTML, and screenshots. Can execute custom actions like clicking or scrolling before scraping.";
define_tool!(
    FIRECRAWL_SCRAPE,
    SCRAPE_TOOL_NAME,
    SCRAPE_TOOL_DESCRIPTION,
    ScrapeUrlInput
);

impl FirecrawlMCP {
    pub async fn scrape(&self, input: JsonObject) -> Result<String, rmcp::Error> {
        //deserialize the json object into a ScrapeOptions struct
        let mut options = parse_json_object::<ScrapeUrlInput>(input)?;
        options.options.formats = Some(vec![ScrapeFormats::Markdown]);

        let result = self
            .client
            .scrape_url(options.url, Some(options.options))
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;
        Ok(result.markdown.unwrap_or_default())
    }
}
