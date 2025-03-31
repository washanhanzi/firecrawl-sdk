use serde::{Deserialize, Serialize};

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{error::FirecrawlAPIError, FirecrawlApp, FirecrawlError, API_VERSION};

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MapOptions {
    /// Optional search term to filter URLs
    pub search: Option<String>,

    /// Skip sitemap.xml discovery and only use HTML links
    pub ignore_sitemap: Option<bool>,

    /// Only use sitemap.xml for discovery, ignore HTML links
    pub sitemap_only: Option<bool>,

    /// Include URLs from subdomains in results
    pub include_subdomains: Option<bool>,

    /// Maximum number of URLs to return
    pub limit: Option<u32>,

    /// Timeout in milliseconds. There is no timeout by default.
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub timeout: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MapRequestBody {
    pub url: String,

    #[serde(flatten)]
    pub options: MapOptions,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct MapResponse {
    success: Option<bool>,
    links: Option<Vec<String>>,
    error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MapUrlInput {
    pub url: String,

    #[serde(flatten)]
    pub options: MapOptions,
}

impl FirecrawlApp {
    /// Returns links from a URL using the Firecrawl API.
    pub async fn map_url(
        &self,
        url: impl AsRef<str>,
        options: impl Into<Option<MapOptions>>,
    ) -> Result<Vec<String>, FirecrawlError> {
        let body = MapRequestBody {
            url: url.as_ref().to_string(),
            options: options.into().unwrap_or_default(),
        };

        let headers = self.prepare_headers(None);

        let response = self
            .client
            .post(format!("{}/{}/map", self.api_url, API_VERSION))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| FirecrawlError::HttpError(format!("Mapping {:?}", url.as_ref()), e))?;

        let response = self
            .handle_response::<MapResponse>(response, "map URL")
            .await?;

        if matches!(response.success, Some(false)) {
            return Err(FirecrawlError::APIError(
                "map request failed".to_string(),
                FirecrawlAPIError {
                    error: response.error.unwrap_or_default(),
                    details: None,
                },
            ));
        }

        Ok(response.links.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_map_options_deserialization() {
        // Create test JSON data
        let json_data = json!({
            "search": "keyword",
            "ignoreSitemap": true,
            "sitemapOnly": false,
            "includeSubdomains": true,
            "limit": 100,
            "timeout": 5000,
        });

        // Deserialize the JSON to our struct
        let options: MapOptions =
            serde_json::from_value(json_data).expect("Failed to deserialize MapOptions");

        // Create expected struct directly
        let expected_options = MapOptions {
            search: Some("keyword".to_string()),
            ignore_sitemap: Some(true),
            sitemap_only: Some(false),
            include_subdomains: Some(true),
            limit: Some(100),
            timeout: Some(5000),
        };

        // Compare the entire structs
        assert_eq!(options, expected_options);
    }

    #[test]
    fn test_map_request_deserialization() {
        // Create test JSON data
        let json_data = json!({
            "url": "https://example.com",
            "search": "keyword",
            "ignoreSitemap": true,
            "sitemapOnly": false,
            "includeSubdomains": true,
            "limit": 100,
            "timeout": 5000,
        });

        // Deserialize the JSON to our struct
        let request_body: MapRequestBody =
            serde_json::from_value(json_data).expect("Failed to deserialize MapRequestBody");

        // Create expected struct directly
        let expected_request_body = MapRequestBody {
            url: "https://example.com".to_string(),
            options: MapOptions {
                search: Some("keyword".to_string()),
                ignore_sitemap: Some(true),
                sitemap_only: Some(false),
                include_subdomains: Some(true),
                limit: Some(100),
                timeout: Some(5000),
            },
        };

        // Compare the entire structs
        assert_eq!(request_body, expected_request_body);
    }

    #[test]
    fn test_map_response_deserialization() {
        // Create test JSON data
        let json_data = json!({
            "success": true,
            "links": [
                "https://example.com/page1",
                "https://example.com/page2",
                "https://example.com/page3"
            ]
        });

        // Deserialize the JSON to our struct
        let response: MapResponse =
            serde_json::from_value(json_data).expect("Failed to deserialize MapResponse");

        // Create expected struct directly
        let expected_response = MapResponse {
            success: Some(true),
            links: Some(vec![
                "https://example.com/page1".to_string(),
                "https://example.com/page2".to_string(),
                "https://example.com/page3".to_string(),
            ]),
            error: None,
        };

        // Compare the entire structs
        assert_eq!(response, expected_response);
    }
}
