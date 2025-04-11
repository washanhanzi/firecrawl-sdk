use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

pub mod batch_scrape;
pub mod crawl;
pub mod document;
mod error;
pub mod map;
pub mod scrape;
pub mod search;

use error::FirecrawlAPIError;
pub use error::FirecrawlError;

#[derive(Clone, Debug)]
pub struct FirecrawlApp {
    api_key: Option<String>,
    api_url: String,
    client: Client,
}

pub(crate) const API_VERSION: &str = "v1";
const CLOUD_API_URL: &str = "https://api.firecrawl.dev";

impl FirecrawlApp {
    pub fn new(api_key: impl AsRef<str>) -> Result<Self, FirecrawlError> {
        FirecrawlApp::new_selfhosted(CLOUD_API_URL, Some(api_key))
    }

    pub fn new_with_client(
        api_key: impl AsRef<str>,
        client: Client,
    ) -> Result<Self, FirecrawlError> {
        Ok(FirecrawlApp {
            api_key: Some(api_key.as_ref().to_string()),
            api_url: CLOUD_API_URL.to_string(),
            client,
        })
    }

    pub fn new_selfhosted(
        api_url: impl AsRef<str>,
        api_key: Option<impl AsRef<str>>,
    ) -> Result<Self, FirecrawlError> {
        let url = api_url.as_ref().to_string();

        if url == CLOUD_API_URL && api_key.is_none() {
            return Err(FirecrawlError::APIError(
                "Configuration".to_string(),
                FirecrawlAPIError {
                    error: "API key is required for cloud service".to_string(),
                    details: None,
                },
            ));
        }

        Ok(FirecrawlApp {
            api_key: api_key.map(|x| x.as_ref().to_string()),
            api_url: url,
            client: Client::new(),
        })
    }

    fn prepare_headers(&self, idempotency_key: Option<&String>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        if let Some(api_key) = self.api_key.as_ref() {
            headers.insert(
                "Authorization",
                format!("Bearer {}", api_key).parse().unwrap(),
            );
        }
        if let Some(key) = idempotency_key {
            headers.insert("x-idempotency-key", key.parse().unwrap());
        }
        headers
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
        action: impl AsRef<str>,
    ) -> Result<T, FirecrawlError> {
        let status = response.status();

        if !status.is_success() {
            // For non-successful status codes, try to extract error details
            match response.json::<FirecrawlAPIError>().await {
                Ok(api_error) => {
                    return Err(FirecrawlError::APIError(
                        action.as_ref().to_string(),
                        api_error,
                    ));
                }
                Err(_) => {
                    return Err(FirecrawlError::HttpRequestFailed(
                        action.as_ref().to_string(),
                        status.as_u16(),
                        status.as_str().to_string(),
                    ));
                }
            }
        }

        // For successful responses, directly deserialize to T
        response.json::<T>().await.map_err(|e| {
            if e.is_decode() {
                FirecrawlError::ResponseParseErrorText(e)
            } else {
                FirecrawlError::HttpError(action.as_ref().to_string(), e)
            }
        })
    }
}
