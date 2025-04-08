use anyhow::Result;
use async_claude::define_tool;
use firecrawl_sdk::{
    batch_scrape::Webhook,
    crawl::CrawlUrlInput,
    scrape::{ScrapeFormats, ScrapeOptions},
};
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject, Error};

use super::Controller;

const CRAWL_TOOL_NAME: &str = "firecrawl_crawl";
const CRAWL_TOOL_DESCRIPTION: &str =
    "Crawl multiple pages from a starting URL. Supports depth control, path filtering, and webhook notifications.";
define_tool!(
    FIRECRAWL_CRAWL,
    CRAWL_TOOL_NAME,
    CRAWL_TOOL_DESCRIPTION,
    CrawlUrlInput
);

impl Controller {
    pub async fn crawl(&self, input: JsonObject) -> Result<String, Error> {
        let mut options = parse_json_object::<CrawlUrlInput>(input)?;

        if options.webhook.is_none() {
            options.webhook = Some(Webhook::dummy());
        }

        // Set the formats to Markdown regardless of whether scrape_options exists
        match &mut options.options.scrape_options {
            Some(scrape_options) => {
                scrape_options.formats = Some(vec![ScrapeFormats::Markdown]);
            }
            None => {
                options.options.scrape_options = Some(ScrapeOptions {
                    formats: Some(vec![ScrapeFormats::Markdown]),
                    ..Default::default()
                });
            }
        }

        let results = self
            .client
            .crawl_url(
                options.url,
                Some(options.options),
                options.webhook.unwrap(),
                options.poll_interval,
                None,
            )
            .await
            .map_err(|e| Error::internal_error(e.to_string(), None))?;

        let formatted = results
            .data
            .iter()
            .map(|d| {
                format!(
                    "URL: {}\nTitle: {}\nContent: {}",
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
