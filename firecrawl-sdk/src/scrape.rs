use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{API_VERSION, FirecrawlApp, FirecrawlError, document::Document};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
pub enum ScrapeFormats {
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
    /// Can not be used in conjunction with `ScrapeFormats::ScreenshotFullPage`.
    #[serde(rename = "screenshot")]
    Screenshot,

    /// Will result in a URL to a full-page screenshot of the page.
    ///
    /// Can not be used in conjunction with `ScrapeFormats::Screenshot`.
    #[serde(rename = "screenshot@fullPage")]
    ScreenshotFullPage,

    /// Will result in structured JSON data based on the schema provided in `jsonOptions`.
    ///
    /// See `ScrapeOptions.json_options` for more options.
    #[serde(rename = "json")]
    JSON,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ExtractOptions {
    /// Schema for structured data extraction
    pub schema: Option<Value>,

    /// System prompt for LLM extraction
    pub system_prompt: Option<String>,

    /// User prompt for LLM extraction
    pub prompt: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct JsonOptions {
    /// Schema the output should adhere to, provided in JSON Schema format.
    pub schema: Option<Value>,

    /// System prompt to send to the LLM agent for schema extraction
    pub system_prompt: Option<String>,

    /// Extraction prompt to send to the LLM agent
    pub prompt: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
    #[default]
    #[serde(rename = "click")]
    Click,

    #[serde(rename = "type")]
    Type,

    #[serde(rename = "wait")]
    Wait,

    #[serde(rename = "screenshot")]
    Screenshot,

    #[serde(rename = "write")]
    Write,

    #[serde(rename = "press")]
    Press,

    #[serde(rename = "scroll")]
    Scroll,

    #[serde(rename = "scrape")]
    Scrape,

    #[serde(rename = "executeJavascript")]
    ExecuteJavascript,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Action {
    /// Type of action to perform
    #[serde(rename = "type")]
    pub action_type: ActionType,

    /// CSS selector for the target element
    pub selector: Option<String>,

    /// Text to write (for write action)
    pub text: Option<String>,

    /// Time to wait in milliseconds (for wait action)
    pub milliseconds: Option<u32>,

    /// Key to press (for press action)
    pub key: Option<String>,

    /// Scroll direction (up or down)
    pub direction: Option<String>,

    /// JavaScript code to execute (for executeJavascript action)
    pub script: Option<String>,

    /// Take full page screenshot (for screenshot action)
    pub full_page: Option<bool>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LocationOptions {
    /// Country code for location emulation
    pub country: String,

    /// Language preferences
    pub languages: Vec<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ScrapeOptions {
    /// Content formats to extract (default: ['markdown'])
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub formats: Option<Vec<ScrapeFormats>>,

    /// Extract only the main content, filtering out navigation, footers, etc. (default: `true`)
    pub only_main_content: Option<bool>,

    /// HTML tags to specifically include in extraction
    pub include_tags: Option<Vec<String>>,

    /// HTML tags to exclude from extraction
    pub exclude_tags: Option<Vec<String>>,

    /// Additional HTTP headers to use when loading the page.
    pub headers: Option<HashMap<String, String>>,

    /// Time in milliseconds to wait for dynamic content to load. (default: `0`)
    pub wait_for: Option<u32>,

    /// Maximum time in milliseconds to wait for the page to load. (default: `60000`)
    pub timeout: Option<u32>,

    /// The JSON options to use for the final extract.
    #[serde(rename = "jsonOptions")]
    pub json_options: Option<JsonOptions>,

    /// Location settings for scraping
    #[cfg_attr(feature = "self_host", schemars(skip))]
    pub location: Option<LocationOptions>,

    /// List of actions to perform before scraping
    #[cfg_attr(feature = "self_host", schemars(skip))]
    pub actions: Option<Vec<Action>>,

    /// Use mobile viewport. (default: `false`)
    pub mobile: Option<bool>,

    /// Skip TLS certificate verification. (default: `false`)
    pub skip_tls_verification: Option<bool>,

    /// Remove base64 encoded images from output. (default: `false`)
    pub remove_base64_images: Option<bool>,

    /// Block ads during page loading (default: `true`)
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub block_ads: Option<bool>,

    /// Proxy configuration to use (values: "none", "basic", "residential") (default: `"none"`)
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub proxy: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeRequestBody {
    /// The URL to scrape
    pub url: String,

    #[serde(flatten)]
    pub options: ScrapeOptions,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ScrapeResponse {
    /// This will always be `true` due to `FirecrawlApp::handle_response`.
    /// No need to expose.
    success: bool,

    /// The resulting document.
    data: Document,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ScrapeUrlInput {
    /// The URL to scrape
    pub url: String,

    #[serde(flatten)]
    pub options: ScrapeOptions,
}

impl FirecrawlApp {
    /// Scrapes a URL using the Firecrawl API.
    pub async fn scrape_url(
        &self,
        url: impl AsRef<str>,
        options: impl Into<Option<ScrapeOptions>>,
    ) -> Result<Document, FirecrawlError> {
        let body = ScrapeRequestBody {
            url: url.as_ref().to_string(),
            options: options.into().unwrap_or_default(),
        };

        let headers = self.prepare_headers(None);

        let response = self
            .client
            .post(format!("{}/{}/scrape", self.api_url, API_VERSION))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| FirecrawlError::HttpError(format!("Scraping {:?}", url.as_ref()), e))?;

        let response = self
            .handle_response::<ScrapeResponse>(response, "scrape URL")
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
    fn test_scrape_options_schema() {
        let actual_schema = async_claude::tool::parse_input_schema::<ScrapeOptions>().unwrap();

        // Create expected schema using json! macro
        let expected_schema = json!({
            "actions": {
                "description": "List of actions to perform before scraping",
                "items": {
                    "properties": {
                        "direction": {
                            "description": "Scroll direction (up or down)",
                            "type": "string"
                        },
                        "fullPage": {
                            "description": "Take full page screenshot (for screenshot action)",
                            "type": "boolean"
                        },
                        "key": {
                            "description": "Key to press (for press action)",
                            "type": "string"
                        },
                        "milliseconds": {
                            "description": "Time to wait in milliseconds (for wait action)",
                            "format": "uint32",
                            "minimum": 0.0,
                            "type": "integer"
                        },
                        "script": {
                            "description": "JavaScript code to execute (for executeJavascript action)",
                            "type": "string"
                        },
                        "selector": {
                            "description": "CSS selector for the target element",
                            "type": "string"
                        },
                        "text": {
                            "description": "Text to write (for write action)",
                            "type": "string"
                        },
                        "type": {
                            "description": "Type of action to perform",
                            "enum": [
                                "click",
                                "type",
                                "wait",
                                "screenshot",
                                "write",
                                "press",
                                "scroll",
                                "scrape",
                                "executeJavascript"
                            ],
                            "type": "string"
                        }
                    },
                    "required": [
                        "type"
                    ],
                    "type": "object"
                },
                "type": "array"
            },
            "excludeTags": {
                "description": "HTML tags to exclude from extraction",
                "items": {
                    "type": "string"
                },
                "type": "array"
            },
            "headers": {
                "additionalProperties": {
                    "type": "string"
                },
                "description": "Additional HTTP headers to use when loading the page.",
                "type": "object"
            },
            "includeTags": {
                "description": "HTML tags to specifically include in extraction",
                "items": {
                    "type": "string"
                },
                "type": "array"
            },
            "jsonOptions": {
                "description": "The JSON options to use for the final extract.",
                "properties": {
                    "prompt": {
                        "description": "Extraction prompt to send to the LLM agent",
                        "type": "string"
                    },
                    "schema": {
                        "description": "Schema the output should adhere to, provided in JSON Schema format."
                    },
                    "systemPrompt": {
                        "description": "System prompt to send to the LLM agent for schema extraction",
                        "type": "string"
                    }
                },
                "type": "object"
            },
            "location": {
                "description": "Location settings for scraping",
                "properties": {
                    "country": {
                        "description": "Country code for location emulation",
                        "type": "string"
                    },
                    "languages": {
                        "description": "Language preferences",
                        "items": {
                            "type": "string"
                        },
                        "type": "array"
                    }
                },
                "required": [
                    "country",
                    "languages"
                ],
                "type": "object"
            },
            "mobile": {
                "description": "Use mobile viewport. (default: `false`)",
                "type": "boolean"
            },
            "onlyMainContent": {
                "description": "Extract only the main content, filtering out navigation, footers, etc. (default: `true`)",
                "type": "boolean"
            },
            "removeBase64Images": {
                "description": "Remove base64 encoded images from output. (default: `false`)",
                "type": "boolean"
            },
            "skipTlsVerification": {
                "description": "Skip TLS certificate verification. (default: `false`)",
                "type": "boolean"
            },
            "timeout": {
                "description": "Maximum time in milliseconds to wait for the page to load. (default: `60000`)",
                "format": "uint32",
                "minimum": 0.0,
                "type": "integer"
            },
            "waitFor": {
                "description": "Time in milliseconds to wait for dynamic content to load. (default: `0`)",
                "format": "uint32",
                "minimum": 0.0,
                "type": "integer"
            }
        });

        // Convert both to strings for comparison
        let actual_json_str = serde_json::to_string_pretty(&actual_schema["properties"]).unwrap();
        let expected_json_str = serde_json::to_string_pretty(&expected_schema).unwrap();

        // Compare the serialized strings
        assert_eq!(
            actual_json_str, expected_json_str,
            "Schema properties don't match"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scrape_request_body_deserialization() {
        let json_data = json!({
            "url": "https://example.com",
            "formats": [
                "markdown"
            ],
            "onlyMainContent": true,
            "includeTags": [
                "div"
            ],
            "excludeTags": [
                "img"
            ],
            "headers": {
                "User-Agent": "Custom User Agent"
            },
            "waitFor": 1000,
            "mobile": false,
            "skipTlsVerification": false,
            "timeout": 30000,
            "jsonOptions": {
                "schema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string"
                        }
                    }
                },
                "systemPrompt": "Extract data from the page",
                "prompt": "Pull out the title"
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
                "languages": [
                    "en-US"
                ]
            },
            "removeBase64Images": true,
            "blockAds": true,
            "proxy": "basic"
        });

        // Deserialize the JSON to our struct
        let req_body: ScrapeRequestBody =
            serde_json::from_value(json_data).expect("Failed to deserialize ScrapeRequestBody");

        // Create expected headers map
        let mut expected_headers = HashMap::new();
        expected_headers.insert("User-Agent".to_string(), "Custom User Agent".to_string());

        // Create expected request body directly
        let expected_req_body = ScrapeRequestBody {
            url: "https://example.com".to_string(),
            options: ScrapeOptions {
                formats: Some(vec![ScrapeFormats::Markdown]),
                include_tags: Some(vec!["div".to_string()]),
                exclude_tags: Some(vec!["img".to_string()]),
                only_main_content: Some(true),
                headers: Some(expected_headers),
                wait_for: Some(1000),
                mobile: Some(false),
                skip_tls_verification: Some(false),
                timeout: Some(30000),
                json_options: Some(JsonOptions {
                    schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" }
                        }
                    })),
                    system_prompt: Some("Extract data from the page".to_string()),
                    prompt: Some("Pull out the title".to_string()),
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
                location: Some(LocationOptions {
                    country: "US".to_string(),
                    languages: vec!["en-US".to_string()],
                }),
                remove_base64_images: Some(true),
                block_ads: Some(true),
                proxy: Some("basic".to_string()),
            },
        };

        // Since req_body has Value fields, we need to compare them separately
        // First compare the entire structs except for json_options
        let json_opts_actual = req_body.options.json_options.clone();
        let json_opts_expected = expected_req_body.options.json_options.clone();

        // Set json_options to None before comparison
        let mut req_body_compare = req_body.clone();
        let mut expected_req_body_compare = expected_req_body.clone();
        req_body_compare.options.json_options = None;
        expected_req_body_compare.options.json_options = None;

        // Compare the structs without the Value fields
        assert_eq!(req_body_compare, expected_req_body_compare);

        // Now compare the json_options fields
        assert_eq!(
            json_opts_actual.as_ref().unwrap().system_prompt,
            json_opts_expected.as_ref().unwrap().system_prompt
        );
        assert_eq!(
            json_opts_actual.as_ref().unwrap().prompt,
            json_opts_expected.as_ref().unwrap().prompt
        );

        // Compare schema values by serializing them to strings
        let schema_actual =
            serde_json::to_string(&json_opts_actual.as_ref().unwrap().schema).unwrap();
        let schema_expected =
            serde_json::to_string(&json_opts_expected.as_ref().unwrap().schema).unwrap();
        assert_eq!(schema_actual, schema_expected);
    }

    #[test]
    fn test_json_options_deserialization() {
        let json_data = json!({
            "schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" }
                }
            },
            "systemPrompt": "Custom system prompt for extraction",
            "prompt": "Extract the title from the page"
        });

        // Deserialize the JSON
        let json_options: JsonOptions =
            serde_json::from_value(json_data).expect("Failed to deserialize JsonOptions");

        // Create expected struct
        let expected_json_options = JsonOptions {
            schema: Some(json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" }
                }
            })),
            system_prompt: Some("Custom system prompt for extraction".to_string()),
            prompt: Some("Extract the title from the page".to_string()),
        };

        // Compare non-Value fields
        assert_eq!(
            json_options.system_prompt,
            expected_json_options.system_prompt
        );
        assert_eq!(json_options.prompt, expected_json_options.prompt);

        // Compare schema values by serializing them to strings
        let schema_actual = serde_json::to_string(&json_options.schema).unwrap();
        let schema_expected = serde_json::to_string(&expected_json_options.schema).unwrap();
        assert_eq!(schema_actual, schema_expected);
    }

    #[test]
    fn test_action_deserialization() {
        // Test wait action
        let wait_action_json = json!({
            "type": "wait",
            "milliseconds": 3000,
            "selector": "#loading"
        });

        let wait_action: Action =
            serde_json::from_value(wait_action_json).expect("Failed to deserialize wait Action");

        let expected_wait_action = Action {
            action_type: ActionType::Wait,
            milliseconds: Some(3000),
            selector: Some("#loading".to_string()),
            text: None,
            key: None,
            direction: None,
            script: None,
            full_page: None,
        };

        // Direct comparison works since Action doesn't contain Value fields
        assert_eq!(wait_action, expected_wait_action);

        // Test click action
        let click_action_json = json!({
            "type": "click",
            "selector": "#submit-button"
        });

        let click_action: Action =
            serde_json::from_value(click_action_json).expect("Failed to deserialize click Action");

        let expected_click_action = Action {
            action_type: ActionType::Click,
            milliseconds: None,
            selector: Some("#submit-button".to_string()),
            text: None,
            key: None,
            direction: None,
            script: None,
            full_page: None,
        };

        assert_eq!(click_action, expected_click_action);

        // Test type action
        let type_action_json = json!({
            "type": "type",
            "selector": "#search-input",
            "text": "search query"
        });

        let type_action: Action =
            serde_json::from_value(type_action_json).expect("Failed to deserialize type Action");

        let expected_type_action = Action {
            action_type: ActionType::Type,
            milliseconds: None,
            selector: Some("#search-input".to_string()),
            text: Some("search query".to_string()),
            key: None,
            direction: None,
            script: None,
            full_page: None,
        };

        assert_eq!(type_action, expected_type_action);
    }
}
