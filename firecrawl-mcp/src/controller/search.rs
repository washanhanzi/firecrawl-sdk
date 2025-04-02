use anyhow::Result;
use firecrawl_sdk::search::{SearchOptions, SearchRequestBody};
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};
use serde_json::Value;
use tracing::info;

use super::Controller;

impl Controller {
    pub async fn search(&self, input: JsonObject) -> Result<String> {
        let options = parse_json_object::<SearchRequestBody>(input)?;

        let results = self.client.search(options.query, options.options).await?;

        let formatted = results
            .iter()
            .map(|r| {
                format!(
                    "Title: {}\nURL: {}\nSnippet: {}\n---",
                    r.title, r.url, r.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(formatted)
    }
}
