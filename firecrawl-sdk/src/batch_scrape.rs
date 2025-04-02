use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{
    document::Document,
    scrape::{Action, JsonOptions, LocationOptions, ScrapeFormats, ScrapeOptions},
    FirecrawlApp, FirecrawlError, API_VERSION,
};

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
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

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BatchScrapeOptions {
    /// Content formats to extract
    pub formats: Option<Vec<ScrapeFormats>>,

    /// Extract only the main content, filtering out navigation, footers, etc.
    pub only_main_content: Option<bool>,

    /// HTML tags to specifically include in extraction
    pub include_tags: Option<Vec<String>>,

    /// HTML tags to exclude from extraction
    pub exclude_tags: Option<Vec<String>>,

    /// Additional HTTP headers to use when loading the page.
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub headers: Option<HashMap<String, String>>,

    /// Time in milliseconds to wait for dynamic content to load
    pub wait_for: Option<u32>,

    /// Use mobile viewport
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub mobile: Option<bool>,

    /// Skip TLS certificate verification
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub skip_tls_verification: Option<bool>,

    /// Maximum time in milliseconds to wait for the page to load
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub timeout: Option<u32>,

    /// JSON options for structured data extraction
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    #[serde(rename = "jsonOptions")]
    pub json_options: Option<JsonOptions>,

    /// List of actions to perform before scraping
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub actions: Option<Vec<Action>>,

    /// Location settings for scraping
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub location: Option<LocationOptions>,

    /// Remove base64 encoded images from output
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub remove_base64_images: Option<bool>,

    /// Block ads during page loading
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub block_ads: Option<bool>,

    /// Proxy configuration to use (values: "none", "basic", "residential")
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub proxy: Option<String>,
}

impl Default for BatchScrapeOptions {
    fn default() -> Self {
        Self {
            formats: None,
            only_main_content: None,
            include_tags: None,
            exclude_tags: None,
            headers: None,
            json_options: None,
            actions: None,
            location: None,
            wait_for: None,
            mobile: None,
            skip_tls_verification: None,
            timeout: None,
            remove_base64_images: None,
            block_ads: None,
            proxy: None,
        }
    }
}

impl From<BatchScrapeOptions> for ScrapeOptions {
    fn from(options: BatchScrapeOptions) -> Self {
        ScrapeOptions {
            formats: options.formats,
            only_main_content: options.only_main_content,
            include_tags: options.include_tags,
            exclude_tags: options.exclude_tags,
            headers: options.headers,
            json_options: options.json_options,
            actions: options.actions,
            location: options.location,
            wait_for: options.wait_for,
            mobile: options.mobile,
            skip_tls_verification: options.skip_tls_verification,
            timeout: options.timeout,
            remove_base64_images: options.remove_base64_images,
            block_ads: options.block_ads,
            proxy: options.proxy,
            extract: None,
            language: None,
            parse_pdf: None,
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
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
    pub options: BatchScrapeOptions,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
struct BatchScrapeResponse {
    /// This will always be `true` due to `FirecrawlApp::handle_response`.
    success: bool,

    /// The resulting documents.
    data: Vec<Document>,
}

impl FirecrawlApp {
    /// Scrapes multiple URLs in a single request using the Firecrawl API.
    pub async fn batch_scrape_urls(
        &self,
        urls: Vec<String>,
        webhook: Option<BatchScrapeWebhook>,
        ignore_invalid_urls: Option<bool>,
        options: impl Into<Option<BatchScrapeOptions>>,
    ) -> Result<Vec<Document>, FirecrawlError> {
        let request_body = BatchScrapeRequestBody {
            urls,
            webhook,
            ignore_invalid_urls,
            options: options.into().unwrap_or_default(),
        };

        let headers = self.prepare_headers(None);

        let response = self
            .client
            .post(format!("{}/{}/batch-scrape", self.api_url, API_VERSION))
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| FirecrawlError::HttpError("Batch scraping URLs".to_string(), e))?;

        let response = self
            .handle_response::<BatchScrapeResponse>(response, "batch scrape URLs")
            .await?;

        Ok(response.data)
    }
}

#[cfg(all(test, feature = "mcp_tool"))]
mod schema_tests {
    use super::*;
    use async_claude;
    use serde_json::json;

    #[test]
    fn test_batch_scrape_request_schema() {
        let actual_schema =
            async_claude::tool::parse_input_schema::<BatchScrapeRequestBody>().unwrap();
        println!("Schema: {:#?}", actual_schema);

        // Check basic structure
        assert_eq!(actual_schema["type"], "object");

        // Get properties object
        let properties = &actual_schema["properties"];
        assert!(properties.is_object());

        // Check required fields
        let required = &actual_schema["required"];
        assert!(required.is_array());
        assert!(required.as_array().unwrap().contains(&json!("urls")));

        // Check urls property
        assert_eq!(properties["urls"]["type"], "array");
        assert_eq!(properties["urls"]["items"]["type"], "string");
        assert_eq!(properties["urls"]["description"], "List of URLs to scrape");

        // Since options is flattened, we check the flattened properties directly
        // Check formats property (from flattened options)
        assert_eq!(properties["formats"]["type"], "array");

        // Check onlyMainContent property (from flattened options)
        assert_eq!(properties["onlyMainContent"]["type"], "boolean");

        // Check array properties (from flattened options)
        assert_eq!(properties["includeTags"]["type"], "array");
        assert_eq!(properties["includeTags"]["items"]["type"], "string");
        assert_eq!(properties["excludeTags"]["type"], "array");
        assert_eq!(properties["excludeTags"]["items"]["type"], "string");

        // Check numeric properties (from flattened options)
        assert!(
            properties["waitFor"]["type"] == "integer" || properties["waitFor"]["type"] == "number"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentMetadata;
    use crate::scrape::ActionType;
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
            options: BatchScrapeOptions {
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
        let batch_options = BatchScrapeOptions {
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

        let scrape_options: ScrapeOptions = batch_options.into();

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
