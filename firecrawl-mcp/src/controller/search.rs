use async_claude::define_tool;
use firecrawl_sdk::search::SearchInput;
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};

use crate::controller::FirecrawlMCP;

pub const SEARCH_TOOL_NAME: &str = "firecrawl_search";
pub const SEARCH_TOOL_DESCRIPTION: &str = "Search and retrieve content from web pages with optional scraping. Returns SERP results by default (url, title, description) or full page content when scrapeOptions are provided.";

define_tool!(
    FIRECRAWL_SEARCH,
    SEARCH_TOOL_NAME,
    SEARCH_TOOL_DESCRIPTION,
    SearchInput
);

impl FirecrawlMCP {
    pub async fn search(&self, input: JsonObject) -> Result<String, rmcp::Error> {
        // Deserialize the json object into a SearchInput struct
        let options = parse_json_object::<SearchInput>(input)?;

        // Call the search method from the firecrawl SDK
        let results = self
            .client
            .search(options.query, Some(options.options))
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;

        // Format the results as a readable string
        if results.is_empty() {
            return Ok("No search results found.".to_string());
        }

        let formatted = results
            .iter()
            .map(|r| {
                format!(
                    "Title: {}\nURL: {}\nDescription: {}\n",
                    r.title, r.url, r.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n\n");

        Ok(formatted)
    }
}
