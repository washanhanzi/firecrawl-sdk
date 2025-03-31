use serde::{Deserialize, Serialize};

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

use crate::{
    error::FirecrawlAPIError, scrape::ScrapeOptions, FirecrawlApp, FirecrawlError, API_VERSION,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
pub struct SearchResult {
    /// The URL of the search result
    pub url: String,
    /// The title of the search result
    pub title: String,
    /// A brief description or snippet of the search result
    pub description: String,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    /// Maximum number of results to return (default: 5)
    pub limit: Option<u32>,

    /// Language code for search results (default: en)
    pub lang: Option<String>,

    /// Country code for search results (default: us)
    pub country: Option<String>,

    /// Time-based search filter
    pub tbs: Option<String>,

    /// Search filter
    pub filter: Option<String>,

    /// Location settings for search
    pub location: Option<LocationOptions>,

    /// Options for scraping search results
    pub scrape_options: Option<ScrapeOptions>,

    /// This field is not in the schema, so we skip it for schema generation
    #[cfg_attr(feature = "mcp_tool", schemars(skip))]
    pub max_results: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LocationOptions {
    /// Country code for geolocation
    pub country: Option<String>,

    /// Language codes for content
    pub languages: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequestBody {
    pub query: String,
    #[serde(flatten)]
    pub options: SearchOptions,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    success: bool,
    data: Option<Vec<SearchResult>>,
    /// Error message when success is false
    error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchInput {
    pub query: String,
    #[serde(flatten)]
    pub options: SearchOptions,
}

impl FirecrawlApp {
    /// Performs a web search using the Firecrawl API.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string
    /// * `options` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Returns a Result containing a vector of SearchResult on success, or a FirecrawlError on failure.
    pub async fn search(
        &self,
        query: impl AsRef<str>,
        options: impl Into<Option<SearchOptions>>,
    ) -> Result<Vec<SearchResult>, FirecrawlError> {
        let body = SearchRequestBody {
            query: query.as_ref().to_string(),
            options: options.into().unwrap_or_default(),
        };

        let headers = self.prepare_headers(None);

        let response = self
            .client
            .post(format!("{}/{}/search", self.api_url, API_VERSION))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                FirecrawlError::HttpError(format!("Searching for {:?}", query.as_ref()), e)
            })?;

        let response = self
            .handle_response::<SearchResponse>(response, "search")
            .await?;

        if !response.success {
            return Err(FirecrawlError::APIError(
                "search request failed".to_string(),
                FirecrawlAPIError {
                    error: response.error.unwrap_or_default(),
                    details: None,
                },
            ));
        }

        Ok(response.data.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_request_deserialization() {
        // JSON to deserialize
        let json_str = r#"{
            "query": "test query",
            "limit": 5,
            "tbs": "qdr:d",
            "lang": "en",
            "country": "us",
            "location": {
                "country": "us",
                "languages": ["en"]
            },
            "timeout": 60000,
            "scrapeOptions": {}
        }"#;

        // Expected struct after deserialization
        let expected = SearchRequestBody {
            query: "test query".to_string(),
            options: SearchOptions {
                limit: Some(5),
                tbs: Some("qdr:d".to_string()),
                lang: Some("en".to_string()),
                country: Some("us".to_string()),
                location: Some(LocationOptions {
                    country: Some("us".to_string()),
                    languages: Some(vec!["en".to_string()]),
                }),
                scrape_options: Some(ScrapeOptions::default()),
                ..Default::default()
            },
        };

        // Deserialize JSON to struct
        let deserialized: SearchRequestBody = serde_json::from_str(json_str).unwrap();

        // Compare the deserialized struct with the expected struct directly
        assert_eq!(deserialized, expected);
    }
}
