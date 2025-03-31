use serde::{Deserialize, Serialize};

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{FirecrawlApp, FirecrawlError, API_VERSION};

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
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct MapRequestBody {
    url: String,

    #[serde(flatten)]
    options: MapOptions,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct MapResponse {
    success: bool,

    links: Vec<String>,
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
            .handle_response::<MapResponse>(response, "scrape URL")
            .await?;

        Ok(response.links)
    }
}

#[cfg(all(test, feature = "mcp_tool"))]
mod schema_tests {
    use super::*;
    use async_claude;

    #[test]
    fn test_map_options_schema() {
        let actual_schema = async_claude::tool::parse_input_schema::<MapOptions>().unwrap();

        // Check basic structure
        assert_eq!(actual_schema["type"], "object");

        // Get properties object
        let properties = &actual_schema["properties"];
        assert!(properties.is_object());

        // Check all expected properties exist
        let expected_properties = [
            "search",
            "ignoreSitemap",
            "sitemapOnly",
            "includeSubdomains",
            "limit",
        ];

        for prop in expected_properties.iter() {
            assert!(
                properties.get(*prop).is_some(),
                "Property {} not found",
                prop
            );
        }

        // Check property types
        assert_eq!(properties["search"]["type"], "string");
        assert_eq!(properties["ignoreSitemap"]["type"], "boolean");
        assert_eq!(properties["sitemapOnly"]["type"], "boolean");
        assert_eq!(properties["includeSubdomains"]["type"], "boolean");
        assert!(
            properties["limit"]["type"] == "integer" || properties["limit"]["type"] == "number",
            "Property limit should be numeric"
        );

        // Check descriptions
        assert_eq!(
            properties["search"]["description"],
            "Optional search term to filter URLs"
        );
        assert_eq!(
            properties["ignoreSitemap"]["description"],
            "Skip sitemap.xml discovery and only use HTML links"
        );
        assert_eq!(
            properties["sitemapOnly"]["description"],
            "Only use sitemap.xml for discovery, ignore HTML links"
        );
        assert_eq!(
            properties["includeSubdomains"]["description"],
            "Include URLs from subdomains in results"
        );
        assert_eq!(
            properties["limit"]["description"],
            "Maximum number of URLs to return"
        );
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
            "limit": 100
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
            "limit": 100
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
            success: true,
            links: vec![
                "https://example.com/page1".to_string(),
                "https://example.com/page2".to_string(),
                "https://example.com/page3".to_string(),
            ],
        };

        // Compare the entire structs
        assert_eq!(response, expected_response);
    }
}
