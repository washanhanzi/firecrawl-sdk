use anyhow::Result;
use firecrawl_sdk::scrape::{ScrapeFormats, ScrapeRequestBody};
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};
use serde::{Deserialize, Serialize};

use super::Controller;

const SCRAPE_TOOL_NAME: &str = "firecrawl_scrape";
const SCRAPE_TOOL_DESCRIPTION: &str = "Scrape a single webpage with advanced options for content extraction. Supports various formats including markdown, HTML, and screenshots. Can execute custom actions like clicking or scrolling before scraping.";

impl Controller {
    pub async fn scrape(&self, input: JsonObject) -> Result<String, rmcp::Error> {
        //deserialize the json object into a ScrapeOptions struct
        let mut options = parse_json_object::<ScrapeRequestBody>(input)?;
        options.options.formats = Some(vec![ScrapeFormats::Markdown]);

        let result = self
            .client
            .scrape_url(options.url, Some(options.options))
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;
        Ok(result.markdown.unwrap_or_default())
    }
}
