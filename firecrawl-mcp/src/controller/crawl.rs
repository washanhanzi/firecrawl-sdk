use anyhow::Result;
use firecrawl_sdk::crawl::{CrawlRequestBody, CrawlScrapeFormats, CrawlScrapeOptions};
use rmcp::{handler::server::tool::parse_json_object, model::JsonObject};

use super::Controller;

impl Controller {
    pub async fn crawl(&self, input: JsonObject) -> Result<String> {
        let mut options = parse_json_object::<CrawlRequestBody>(input)?;

        // Set the formats to Markdown regardless of whether scrape_options exists
        match &mut options.options.scrape_options {
            Some(scrape_options) => {
                scrape_options.formats = Some(vec![CrawlScrapeFormats::Markdown]);
            }
            None => {
                options.options.scrape_options = Some(CrawlScrapeOptions {
                    formats: Some(vec![CrawlScrapeFormats::Markdown]),
                    ..Default::default()
                });
            }
        }

        let results = self
            .client
            .crawl_url(options.url, Some(options.options))
            .await?;

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
            .join("\n");

        Ok(formatted)
    }
}
