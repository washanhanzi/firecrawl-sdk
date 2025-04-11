use async_claude::define_tool;
use firecrawl_sdk::map::MapUrlInput;
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};
use serde_json::json;

use crate::controller::FirecrawlMCP;

pub const MAP_TOOL_NAME: &str = "firecrawl_map";
pub const MAP_TOOL_DESCRIPTION: &str =
    "Discover URLs from a starting point. Can use both sitemap.xml and HTML link discovery.";

define_tool!(
    FIRECRAWL_MAP,
    MAP_TOOL_NAME,
    MAP_TOOL_DESCRIPTION,
    MapUrlInput
);

impl FirecrawlMCP {
    pub async fn map(&self, input: JsonObject) -> Result<String, rmcp::Error> {
        // Deserialize the json object into a MapUrlInput struct
        let options = parse_json_object::<MapUrlInput>(input)?;

        // Call the map_url method from the firecrawl SDK
        let result = self
            .client
            .map_url(options.url, Some(options.options))
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;

        // Format the result as a JSON array of URLs
        let json_result = json!(result);
        serde_json::to_string_pretty(&json_result)
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))
    }
}
