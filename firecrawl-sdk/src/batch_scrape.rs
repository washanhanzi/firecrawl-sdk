use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{document::Document, scrape::ScrapeOptions, FirecrawlApp, FirecrawlError, API_VERSION};

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchScrapeWebhook {
    /// Webhook URL to notify when scraping is complete
    pub url: String,

    /// Custom headers to send with webhook
    pub headers: Option<HashMap<String, String>>,

    /// Custom metadata to include in webhook payload
    pub metadata: Option<HashMap<String, Value>>,

    /// Events that trigger the webhook
    pub events: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchScrapeRequestBody {
    /// List of URLs to scrape
    pub urls: Vec<String>,

    /// Webhook configuration for notifications
    pub webhook: Option<BatchScrapeWebhook>,

    /// Whether to ignore invalid URLs
    pub ignore_invalid_urls: Option<bool>,

    /// Scraping options
    #[serde(flatten)]
    pub options: ScrapeOptions,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct BatchScrapeResponse {
    /// This will always be `true` due to `FirecrawlApp::handle_response`.
    success: bool,

    /// The ID of the batch scrape job
    id: String,

    /// The URL to check the status of the batch scrape job
    url: String,

    /// If ignoreInvalidURLs is true, this is an array containing the invalid URLs
    /// that were specified in the request. If there were no invalid URLs, this will
    /// be an empty array. If ignoreInvalidURLs is false, this field will be undefined.
    #[serde(skip_serializing_if = "Option::is_none")]
    invalid_urls: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BatchScrapeUrlsInput {
    /// List of URLs to scrape
    pub urls: Vec<String>,

    /// Webhook configuration for notifications
    #[serde(skip)]
    pub webhook: Option<BatchScrapeWebhook>,

    /// Whether to ignore invalid URLs
    #[serde(skip)]
    pub ignore_invalid_urls: Option<bool>,

    /// Poll interval in milliseconds. (default: 2000)
    pub poll_interval: Option<u64>,

    #[serde(skip)]
    pub idempotency_key: Option<String>,

    /// Scraping options
    #[serde(flatten)]
    pub options: Option<ScrapeOptions>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum BatchScrapeStatusTypes {
    Scraping,
    Completed,
    Failed,
}

impl Default for BatchScrapeStatusTypes {
    fn default() -> Self {
        Self::Scraping
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchScrapeStatus {
    /// This will always be `true` due to `FirecrawlApp::handle_response`.
    pub success: bool,

    /// The status of the batch scrape job
    pub status: BatchScrapeStatusTypes,

    /// The total number of URLs in the batch
    #[serde(default)]
    pub total: usize,

    /// The number of completed URLs in the batch
    #[serde(default)]
    pub completed: usize,

    /// The number of credits used for this batch scrape
    #[serde(default)]
    pub credits_used: usize,

    /// When the batch scrape results expire
    pub expires_at: Option<String>,

    /// Cursor for the next page of results, if any
    pub next: Option<String>,

    /// The resulting documents if the status is Completed
    #[serde(default)]
    pub data: Vec<Document>,
}

impl FirecrawlApp {
    /// Scrapes multiple URLs in a single request using the Firecrawl API.
    pub async fn batch_scrape_urls(
        &self,
        urls: Vec<String>,
        options: impl Into<Option<ScrapeOptions>>,
        poll_interval: Option<u64>,
        idempotency_key: Option<String>,
        webhook: Option<BatchScrapeWebhook>,
        ignore_invalid_urls: Option<bool>,
    ) -> Result<BatchScrapeStatus, FirecrawlError> {
        let request_body = BatchScrapeRequestBody {
            urls,
            webhook,
            ignore_invalid_urls,
            options: options.into().unwrap_or_default(),
        };

        let headers = self.prepare_headers(idempotency_key.as_ref());

        let response = self
            .client
            .post(format!("{}/{}/batch/scrape", self.api_url, API_VERSION))
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| FirecrawlError::HttpError("Batch scraping URLs".to_string(), e))?;

        let response = self
            .handle_response::<BatchScrapeResponse>(response, "batch scrape URLs")
            .await?;

        let poll_interval = poll_interval.unwrap_or(2000);
        self.monitor_batch_scrape_status(&response.id, poll_interval)
            .await
    }

    /// Checks the status of a batch scrape job.
    pub async fn check_batch_scrape_status(
        &self,
        id: &str,
    ) -> Result<BatchScrapeStatus, FirecrawlError> {
        let headers = self.prepare_headers(None);

        let response = self
            .client
            .get(format!(
                "{}/{}/batch/scrape/{}",
                self.api_url, API_VERSION, id
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                FirecrawlError::HttpError("Checking batch scrape status".to_string(), e)
            })?;

        self.handle_response::<BatchScrapeStatus>(response, "check batch scrape status")
            .await
    }

    /// Monitors a batch scrape job until it completes, fails, or is cancelled.
    pub async fn monitor_batch_scrape_status(
        &self,
        id: &str,
        poll_interval: u64,
    ) -> Result<BatchScrapeStatus, FirecrawlError> {
        let mut all_data = Vec::new();
        let mut current_cursor: Option<String> = None;

        loop {
            let mut status_data = if let Some(ref cursor) = current_cursor {
                self.check_batch_scrape_status_with_cursor(id, cursor)
                    .await?
            } else {
                self.check_batch_scrape_status(id).await?
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
                BatchScrapeStatusTypes::Completed => {
                    // Put all collected data back into the status
                    status_data.data = all_data;
                    break Ok(status_data);
                }
                BatchScrapeStatusTypes::Scraping => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(poll_interval)).await;
                    // Keep the cursor as is, to continue from where we left off
                }
                BatchScrapeStatusTypes::Failed => {
                    break Err(FirecrawlError::BatchScrapeJobFailed(
                        "Batch scrape job failed.".to_string(),
                    ));
                }
            }
        }
    }

    /// Checks the status of a batch scrape job with a cursor for pagination.
    pub async fn check_batch_scrape_status_with_cursor(
        &self,
        id: &str,
        cursor: &str,
    ) -> Result<BatchScrapeStatus, FirecrawlError> {
        let headers = self.prepare_headers(None);

        let response = self
            .client
            .get(format!(
                "{}/{}/batch/scrape/{}?cursor={}",
                self.api_url, API_VERSION, id, cursor
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                FirecrawlError::HttpError("Checking batch scrape status".to_string(), e)
            })?;

        self.handle_response::<BatchScrapeStatus>(response, "check batch scrape status")
            .await
    }

    /// Scrapes multiple URLs in a single request using the Firecrawl API and waits for the results.
    pub async fn batch_scrape_urls_and_wait(
        &self,
        urls: Vec<String>,
        options: impl Into<Option<ScrapeOptions>>,
        poll_interval: Option<u64>,
        idempotency_key: Option<String>,
        webhook: Option<BatchScrapeWebhook>,
        ignore_invalid_urls: Option<bool>,
    ) -> Result<Vec<Document>, FirecrawlError> {
        let status = self
            .batch_scrape_urls(
                urls,
                options,
                poll_interval,
                idempotency_key,
                webhook,
                ignore_invalid_urls,
            )
            .await?;

        Ok(status.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrape::{Action, ActionType, JsonOptions, ScrapeFormats};
    use serde_json::json;

    #[test]
    fn test_batch_scrape_request_serialization() {
        // API example JSON
        let json_data = json!({
            "urls": ["https://example.com"],
            "webhook": {
                "url": "https://webhook.example.com",
                "headers": {},
                "metadata": {},
                "events": ["completed"]
            },
            "formats": ["markdown"],
            "onlyMainContent": true,
            "includeTags": ["div"],
            "excludeTags": ["img"],
            "headers": {},
            "waitFor": 0,
            "mobile": false,
            "skipTlsVerification": false,
            "timeout": 30000,
            "jsonOptions": {
                "schema": { "type": "object" },
                "systemPrompt": "Extract data",
                "prompt": "Extract title"
            },
            "actions": [
                {
                    "type": "wait",
                    "milliseconds": 2000,
                    "selector": "#my-element"
                }
            ],
            "location": {
                "country": "US",
                "languages": ["en-US"]
            },
            "removeBase64Images": true,
            "blockAds": true,
            "proxy": "basic"
        });

        // Deserialize the JSON to our request body struct
        let req_body: BatchScrapeRequestBody =
            serde_json::from_value(json_data).expect("Failed to deserialize JSON");

        // Create the expected complete request body struct
        let expected_req_body = BatchScrapeRequestBody {
            urls: vec!["https://example.com".to_string()],
            webhook: Some(BatchScrapeWebhook {
                url: "https://webhook.example.com".to_string(),
                headers: Some(HashMap::new()),
                metadata: Some(HashMap::new()),
                events: Some(vec!["completed".to_string()]),
            }),
            ignore_invalid_urls: None, // This field wasn't in the JSON, so it should be None
            options: ScrapeOptions {
                formats: Some(vec![ScrapeFormats::Markdown]),
                only_main_content: Some(true),
                include_tags: Some(vec!["div".to_string()]),
                exclude_tags: Some(vec!["img".to_string()]),
                headers: Some(HashMap::new()),
                wait_for: Some(0),
                mobile: Some(false),
                skip_tls_verification: Some(false),
                timeout: Some(30000),
                json_options: Some(JsonOptions {
                    schema: Some(json!({"type": "object"})),
                    system_prompt: Some("Extract data".to_string()),
                    prompt: Some("Extract title".to_string()),
                }),
                actions: Some(vec![Action {
                    action_type: ActionType::Wait,
                    milliseconds: Some(2000),
                    selector: Some("#my-element".to_string()),
                    text: None,
                    key: None,
                    direction: None,
                    script: None,
                    full_page: None,
                }]),
                location: Some(crate::scrape::LocationOptions {
                    country: "US".to_string(),
                    languages: vec!["en-US".to_string()],
                }),
                remove_base64_images: Some(true),
                block_ads: Some(true),
                proxy: Some("basic".to_string()),
            },
        };

        // Compare the entire structs
        assert_eq!(req_body, expected_req_body);
    }

    #[test]
    fn test_batch_scrape_options_to_scrape_options() {
        let scrape_options = ScrapeOptions {
            formats: Some(vec![ScrapeFormats::Markdown]),
            only_main_content: Some(true),
            include_tags: Some(vec!["div".to_string()]),
            exclude_tags: Some(vec!["img".to_string()]),
            headers: Some(HashMap::new()),
            wait_for: Some(1000),
            mobile: Some(true),
            skip_tls_verification: Some(false),
            timeout: Some(2000),
            json_options: Some(crate::scrape::JsonOptions::default()),
            actions: Some(vec![]),
            location: Some(crate::scrape::LocationOptions::default()),
            remove_base64_images: Some(true),
            block_ads: Some(true),
            proxy: Some("basic".to_string()),
        };

        assert_eq!(scrape_options.formats.as_ref().unwrap().len(), 1);
        assert!(matches!(
            scrape_options.formats.as_ref().unwrap()[0],
            ScrapeFormats::Markdown
        ));
        assert!(scrape_options.only_main_content.unwrap());
        assert_eq!(scrape_options.include_tags.as_ref().unwrap()[0], "div");
        assert_eq!(scrape_options.exclude_tags.as_ref().unwrap()[0], "img");
        assert_eq!(scrape_options.wait_for.unwrap(), 1000);
        assert!(scrape_options.headers.is_some());
        assert!(scrape_options.mobile.unwrap());
        assert!(!scrape_options.skip_tls_verification.unwrap());
        assert_eq!(scrape_options.timeout.unwrap(), 2000);
        assert!(scrape_options.json_options.is_some());
        assert!(scrape_options.actions.is_some());
        assert!(scrape_options.location.is_some());
        assert!(scrape_options.remove_base64_images.unwrap());
        assert!(scrape_options.block_ads.unwrap());
        assert_eq!(scrape_options.proxy.as_ref().unwrap(), "basic");
    }
}
