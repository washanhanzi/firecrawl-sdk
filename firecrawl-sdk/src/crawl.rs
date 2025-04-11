use serde::{Deserialize, Serialize};

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{
    API_VERSION, FirecrawlApp, FirecrawlError,
    batch_scrape::Webhook,
    document::Document,
    scrape::{ScrapeFormats, ScrapeOptions},
};

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
pub enum CrawlScrapeFormats {
    /// Will result in a copy of the Markdown content of the page.
    #[serde(rename = "markdown")]
    Markdown,

    /// Will result in a copy of the filtered, content-only HTML of the page.
    #[serde(rename = "html")]
    HTML,

    /// Will result in a copy of the raw HTML of the page.
    #[serde(rename = "rawHtml")]
    RawHTML,

    /// Will result in a Vec of URLs found on the page.
    #[serde(rename = "links")]
    Links,

    /// Will result in a URL to a screenshot of the page.
    ///
    /// Can not be used in conjunction with `CrawlScrapeFormats::ScreenshotFullPage`.
    #[serde(rename = "screenshot")]
    Screenshot,

    /// Will result in a URL to a full-page screenshot of the page.
    ///
    /// Can not be used in conjunction with `CrawlScrapeFormats::Screenshot`.
    #[serde(rename = "screenshot@fullPage")]
    ScreenshotFullPage,
}

impl From<CrawlScrapeFormats> for ScrapeFormats {
    fn from(value: CrawlScrapeFormats) -> Self {
        match value {
            CrawlScrapeFormats::Markdown => Self::Markdown,
            CrawlScrapeFormats::HTML => Self::HTML,
            CrawlScrapeFormats::RawHTML => Self::RawHTML,
            CrawlScrapeFormats::Links => Self::Links,
            CrawlScrapeFormats::Screenshot => Self::Screenshot,
            CrawlScrapeFormats::ScreenshotFullPage => Self::ScreenshotFullPage,
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CrawlOptions {
    /// Options for scraping each page
    pub scrape_options: Option<ScrapeOptions>,

    /// Only crawl these URL paths
    pub include_paths: Option<Vec<String>>,

    /// URL paths to exclude from crawling
    pub exclude_paths: Option<Vec<String>>,

    /// Maximum link depth to crawl. (default: `2`)
    pub max_depth: Option<u32>,

    /// Skip sitemap.xml discovery. (default: `true`)
    pub ignore_sitemap: Option<bool>,

    /// Maximum number of pages to crawl. (default: `10`)
    pub limit: Option<u32>,

    /// Allow crawling links that point to parent directories. (default: `false`)
    pub allow_backward_links: Option<bool>,

    /// Allow crawling links to external domains. (default: `false`)
    pub allow_external_links: Option<bool>,

    /// Remove similar URLs during crawl
    #[serde(rename = "deduplicateSimilarURLs")]
    pub deduplicate_similar_urls: Option<bool>,

    /// Ignore query parameters when comparing URLs
    pub ignore_query_parameters: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRequestBody {
    /// Starting URL for the crawl
    pub url: String,

    #[serde(flatten)]
    pub options: CrawlOptions,

    /// Webhook URL to notify when crawl is complete
    pub webhook: Webhook,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlResponse {
    /// This will always be `true` due to `FirecrawlApp::handle_response`.
    /// No need to expose.
    pub success: bool,

    /// The resulting document.
    pub data: Document,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum CrawlStatusTypes {
    /// The crawl job is in progress.
    Scraping,

    /// The crawl job has been completed successfully.
    Completed,

    /// The crawl job has failed.
    Failed,

    /// The crawl job has been cancelled.
    Cancelled,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStatus {
    /// The status of the crawl.
    pub status: CrawlStatusTypes,

    /// Number of pages that will be scraped in total. This number may grow as the crawler discovers new pages.
    pub total: u32,

    /// Number of pages that have been successfully scraped.
    pub completed: u32,

    /// Amount of credits used by the crawl job.
    pub credits_used: u32,

    /// Expiry time of crawl data. After this date, the crawl data will be unavailable from the API.
    pub expires_at: String, // TODO: parse into date

    /// URL to call to get the next batch of documents.
    /// Unless you are sidestepping the SDK, you do not need to deal with this.
    pub next: Option<String>,

    /// List of documents returned by the crawl
    pub data: Vec<Document>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CrawlAsyncResponse {
    success: bool,

    /// Crawl ID
    pub id: String,

    /// URL to get the status of the crawl job
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CrawlUrlInput {
    /// Starting URL for the crawl
    pub url: String,

    #[serde(flatten)]
    pub options: CrawlOptions,

    /// How often the status of the job should be checked, in milliseconds. (default: `2000`)
    pub poll_interval: Option<u64>,

    #[serde(skip)]
    pub idempotency_key: Option<String>,

    /// Webhook URL to notify when crawl is complete, default to a dummy url
    pub webhook: Option<Webhook>,
}

impl FirecrawlApp {
    /// Initiates a crawl job for a URL using the Firecrawl API.
    pub async fn crawl_url_async(
        &self,
        url: impl AsRef<str>,
        options: Option<CrawlOptions>,
        idempotency_key: Option<String>,
        webhook: Webhook,
    ) -> Result<CrawlAsyncResponse, FirecrawlError> {
        let body = CrawlRequestBody {
            url: url.as_ref().to_string(),
            options: options.unwrap_or_default(),
            webhook,
        };

        let headers = self.prepare_headers(idempotency_key.as_ref());

        let response = self
            .client
            .post(format!("{}/{}/crawl", self.api_url, API_VERSION))
            .headers(headers.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| FirecrawlError::HttpError(format!("Crawling {:?}", url.as_ref()), e))?;

        self.handle_response::<CrawlAsyncResponse>(response, "start crawl job")
            .await
    }

    /// Performs a crawl job for a URL using the Firecrawl API, waiting for the end result. This may take a long time depending on the size of the target page and your options (namely `CrawlOptions.limit`).
    pub async fn crawl_url(
        &self,
        url: impl AsRef<str>,
        options: impl Into<Option<CrawlOptions>>,
        webhook: Webhook,
        poll_interval: Option<u64>,
        idempotency_key: Option<String>,
    ) -> Result<CrawlStatus, FirecrawlError> {
        let options = options.into();
        let poll_interval = poll_interval.unwrap_or(2000);

        let res = self
            .crawl_url_async(url, options, idempotency_key, webhook)
            .await?;

        self.monitor_crawl_status(&res.id, poll_interval).await
    }

    async fn check_crawl_status_next(
        &self,
        next: impl AsRef<str>,
    ) -> Result<CrawlStatus, FirecrawlError> {
        let response = self
            .client
            .get(next.as_ref())
            .headers(self.prepare_headers(None))
            .send()
            .await
            .map_err(|e| {
                FirecrawlError::HttpError(
                    format!("Paginating crawl using URL {:?}", next.as_ref()),
                    e,
                )
            })?;

        self.handle_response(
            response,
            format!("Paginating crawl using URL {:?}", next.as_ref()),
        )
        .await
    }

    /// Checks for the status of a crawl, based on the crawl's ID. To be used in conjunction with `FirecrawlApp::crawl_url_async`.
    pub async fn check_crawl_status(
        &self,
        id: impl AsRef<str>,
    ) -> Result<CrawlStatus, FirecrawlError> {
        let response = self
            .client
            .get(format!(
                "{}/{}/crawl/{}",
                self.api_url,
                API_VERSION,
                id.as_ref()
            ))
            .headers(self.prepare_headers(None))
            .send()
            .await
            .map_err(|e| {
                FirecrawlError::HttpError(format!("Checking status of crawl {}", id.as_ref()), e)
            })?;

        let mut status: CrawlStatus = self
            .handle_response(
                response,
                format!("Checking status of crawl {}", id.as_ref()),
            )
            .await?;

        if status.status == CrawlStatusTypes::Completed {
            while let Some(next) = status.next {
                let new_status = self.check_crawl_status_next(next).await?;
                status.data.extend_from_slice(&new_status.data);
                status.next = new_status.next;
            }
        }

        Ok(status)
    }

    async fn monitor_crawl_status(
        &self,
        id: &str,
        poll_interval: u64,
    ) -> Result<CrawlStatus, FirecrawlError> {
        let mut all_data = Vec::new();
        let mut current_cursor: Option<String> = None;

        loop {
            // Get status data, either from the base endpoint or using the next cursor
            let mut status_data = if let Some(ref cursor) = current_cursor {
                self.check_crawl_status_next(cursor).await?
            } else {
                self.check_crawl_status(id).await?
            };

            // Collect data from this page
            all_data.append(&mut status_data.data);

            // Check if we need to paginate
            if let Some(next) = status_data.next {
                current_cursor = Some(next);
                continue;
            }

            // Check job status
            match status_data.status {
                CrawlStatusTypes::Completed => {
                    // Put all collected data back into the status
                    status_data.data = all_data;
                    break Ok(status_data);
                }
                CrawlStatusTypes::Scraping => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(poll_interval)).await;
                    // Keep the cursor as is, to continue from where we left off
                }
                CrawlStatusTypes::Failed => {
                    // Put all collected data back into the status for error context
                    status_data.data = all_data;
                    break Err(FirecrawlError::CrawlJobFailed(
                        "Crawl job failed.".to_string(),
                        status_data,
                    ));
                }
                CrawlStatusTypes::Cancelled => {
                    // Put all collected data back into the status for error context
                    status_data.data = all_data;
                    break Err(FirecrawlError::CrawlJobCancelled(status_data));
                }
            }
        }
    }
}

#[cfg(all(test, feature = "mcp_tool"))]
mod schema_tests {
    use super::*;
    use async_claude;

    #[test]
    fn test_crawl_options_schema() {
        let actual_schema = async_claude::tool::parse_input_schema::<CrawlOptions>().unwrap();

        // For debugging
        println!(
            "Schema properties: {}",
            serde_json::to_string_pretty(&actual_schema["properties"]).unwrap()
        );

        // Check basic structure
        assert_eq!(actual_schema["type"], "object");

        // Get properties object
        let properties = &actual_schema["properties"];
        assert!(properties.is_object());

        // Get the actual property keys from the schema
        let property_keys: Vec<String> = properties
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.to_string())
            .collect();

        println!("Actual property keys: {:?}", property_keys);

        // Check that important properties exist, being flexible with URL vs Url
        assert!(
            property_keys.contains(&"scrapeOptions".to_string()),
            "scrapeOptions not found"
        );
        assert!(
            property_keys.contains(&"includePaths".to_string()),
            "includePaths not found"
        );
        assert!(
            property_keys.contains(&"excludePaths".to_string()),
            "excludePaths not found"
        );
        assert!(
            property_keys.contains(&"maxDepth".to_string()),
            "maxDepth not found"
        );
        assert!(
            property_keys.contains(&"ignoreSitemap".to_string()),
            "ignoreSitemap not found"
        );
        assert!(
            property_keys.contains(&"limit".to_string()),
            "limit not found"
        );
        assert!(
            property_keys.contains(&"allowBackwardLinks".to_string()),
            "allowBackwardLinks not found"
        );
        assert!(
            property_keys.contains(&"allowExternalLinks".to_string()),
            "allowExternalLinks not found"
        );
        assert!(
            property_keys.contains(&"webhook".to_string()),
            "webhook not found"
        );

        // Check for deduplicateSimilarURLs or deduplicateSimilarUrls
        assert!(
            property_keys
                .iter()
                .any(|k| k.to_lowercase() == "deduplicatesimilarurls"),
            "deduplicateSimilarURLs not found"
        );

        // Check for ignoreQueryParameters
        assert!(
            property_keys.contains(&"ignoreQueryParameters".to_string()),
            "ignoreQueryParameters not found"
        );

        // Check expected property types and descriptions for properties that certainly exist
        assert_eq!(properties["scrapeOptions"]["type"], "object");

        // Check array properties
        assert_eq!(properties["includePaths"]["type"], "array");
        assert_eq!(properties["includePaths"]["items"]["type"], "string");
        assert_eq!(properties["excludePaths"]["type"], "array");
        assert_eq!(properties["excludePaths"]["items"]["type"], "string");

        // Check boolean properties
        assert_eq!(properties["ignoreSitemap"]["type"], "boolean");
        assert_eq!(properties["allowBackwardLinks"]["type"], "boolean");
        assert_eq!(properties["allowExternalLinks"]["type"], "boolean");

        // Check numeric properties
        assert!(
            properties["maxDepth"]["type"] == "integer"
                || properties["maxDepth"]["type"] == "number",
            "Property maxDepth should be numeric"
        );
        assert!(
            properties["limit"]["type"] == "integer" || properties["limit"]["type"] == "number",
            "Property limit should be numeric"
        );
    }
}
