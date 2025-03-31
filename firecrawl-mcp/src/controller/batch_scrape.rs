use anyhow::Result;
use async_claude::define_tool;
use firecrawl_sdk::{
    batch_scrape::BatchScrapeUrlsInput,
    scrape::{ScrapeFormats, ScrapeOptions},
};
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};

use super::Controller;

const BATCH_SCRAPE_TOOL_NAME: &str = "firecrawl_batch_scrape";
const BATCH_SCRAPE_TOOL_DESCRIPTION: &str = "Scrape multiple URLs in batch mode.";
define_tool!(
    FIRECRAWL_BATCH_SCRAPE,
    BATCH_SCRAPE_TOOL_NAME,
    BATCH_SCRAPE_TOOL_DESCRIPTION,
    BatchScrapeUrlsInput
);

impl Controller {
    pub async fn batch_scrape(&self, input: JsonObject) -> Result<String, rmcp::Error> {
        //deserialize the json object into a ScrapeOptions struct
        let mut options = parse_json_object::<BatchScrapeUrlsInput>(input)?;
        match &mut options.options {
            Some(scrape_options) => {
                scrape_options.formats = Some(vec![ScrapeFormats::Markdown]);
            }
            None => {
                options.options = Some(ScrapeOptions {
                    formats: Some(vec![ScrapeFormats::Markdown]),
                    ..Default::default()
                })
            }
        }

        let result = self
            .client
            .batch_scrape_urls(
                options.urls,
                options.options,
                options.poll_interval,
                None,
                None,
                Some(true),
            )
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;

        let formatted = result
            .data
            .iter()
            .map(|d| {
                format!(
                    "URL: {}\nTitle: {}\nContent: {}\n\n",
                    d.metadata.source_url,
                    d.metadata.title.as_ref().unwrap_or(&"".to_string()),
                    d.markdown.as_ref().unwrap_or(&"".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(formatted)
    }
}
