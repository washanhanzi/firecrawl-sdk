use dotenvy::dotenv;
use firecrawl_sdk::scrape::{JsonOptions, ScrapeFormats, ScrapeOptions};
use firecrawl_sdk::{FirecrawlApp, FirecrawlError};
use serde_json::json;
use std::env;

// #[tokio::test]
// async fn test_blocklisted_url() {
//     dotenv().ok();
//     let api_url = env::var("API_URL").unwrap();
//     let api_key = env::var("TEST_API_KEY").ok();
//     let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
//     let blocklisted_url = "https://facebook.com/fake-test";
//     let result = app.scrape_url(blocklisted_url, None).await;

//     assert_matches!(
//         result,
//         Err(e) if e.to_string().contains("Firecrawl currently does not support social media scraping due to policy restrictions")
//     );
// }

#[tokio::test]
async fn test_scrape_url_e2e() {
    dotenv().ok();
    let api_url = env::var("API_URL").unwrap();
    let api_key = env::var("TEST_API_KEY").ok();
    let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
    let result = app
        .scrape_url("https://roastmywebsite.ai", None)
        .await
        .unwrap();
    assert!(result.markdown.is_some());
    assert!(result.markdown.unwrap().contains("_Roast_"));
}

#[tokio::test]
async fn test_successful_response_with_valid_api_key_and_include_html() {
    dotenv().ok();
    let api_url = env::var("API_URL").unwrap();
    let api_key = env::var("TEST_API_KEY").ok();
    let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
    let params = ScrapeOptions {
        formats: vec![ScrapeFormats::Markdown, ScrapeFormats::HTML].into(),
        ..Default::default()
    };
    let result = app
        .scrape_url("https://roastmywebsite.ai", params)
        .await
        .unwrap();
    assert!(result.markdown.is_some());
    assert!(result.html.is_some());
    assert!(result.markdown.unwrap().contains("_Roast_"));
    assert!(result.html.unwrap().contains("<h1"));
}

#[tokio::test]
async fn test_successful_response_for_valid_scrape_with_pdf_file() {
    dotenv().ok();
    let api_url = env::var("API_URL").unwrap();
    let api_key = env::var("TEST_API_KEY").ok();
    let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
    let result = app
        .scrape_url("https://arxiv.org/pdf/astro-ph/9301001.pdf", None)
        .await
        .unwrap();
    assert!(result.markdown.is_some());
    assert!(result
        .markdown
        .unwrap()
        .contains("We present spectrophotometric observations of the Broad Line Radio Galaxy"));
}

#[tokio::test]
async fn test_successful_response_for_valid_scrape_with_pdf_file_without_explicit_extension() {
    dotenv().ok();
    let api_url = env::var("API_URL").unwrap();
    let api_key = env::var("TEST_API_KEY").ok();
    let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
    let result = app
        .scrape_url("https://arxiv.org/pdf/astro-ph/9301001", None)
        .await
        .unwrap();
    assert!(result.markdown.is_some());
    assert!(result
        .markdown
        .unwrap()
        .contains("We present spectrophotometric observations of the Broad Line Radio Galaxy"));
}

// #[tokio::test]
// async fn test_should_return_error_for_blocklisted_url() {
//     dotenv().ok();
//     let api_url = env::var("API_URL").unwrap();
//     let api_key = env::var("TEST_API_KEY").ok();
//     let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
//     let blocklisted_url = "https://twitter.com/fake-test";
//     let result = app.crawl_url(blocklisted_url, None).await;

//     assert_matches!(
//         result,
//         Err(e) if e.to_string().contains("Firecrawl currently does not support social media scraping due to policy restrictions.")
//     );
// }

#[tokio::test]
async fn test_llm_extraction() {
    dotenv().ok();
    let api_url = env::var("API_URL").unwrap();
    let api_key = env::var("TEST_API_KEY").ok();
    let app = FirecrawlApp::new_selfhosted(api_url, api_key).unwrap();
    let options = ScrapeOptions {
        formats: Some(vec![ ScrapeFormats::JSON ]),
        json_options: Some(JsonOptions {
            prompt: "Based on the information on the page, find what the company's mission is and whether it supports SSO, and whether it is open source".to_string().into(),
            schema: json!({
                "type": "object",
                "properties": {
                    "company_mission": {"type": "string"},
                    "supports_sso": {"type": "boolean"},
                    "is_open_source": {"type": "boolean"}
                },
                "required": ["company_mission", "supports_sso", "is_open_source"]
            }).into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    let result = app
        .scrape_url("https://mendable.ai", options)
        .await
        .unwrap();
    assert!(result.json.is_some());
    let json_extract = &result.json.unwrap();
    assert!(json_extract
        .as_object()
        .unwrap()
        .contains_key("company_mission"));
    assert!(json_extract["supports_sso"].is_boolean());
    assert!(json_extract["is_open_source"].is_boolean());
}

#[test]
fn test_api_key_requirements() {
    dotenv().ok();

    let api_url = env::var("API_URL").unwrap_or("http://localhost:3002".to_string());
    let api_key = env::var("TEST_API_KEY").ok();

    match (api_url.contains("api.firecrawl.dev"), api_key) {
        (false, _) => {
            let result = FirecrawlApp::new_selfhosted(&api_url, None::<String>);
            assert!(
                result.is_ok(),
                "Local setup failed: {:?}",
                result.err().unwrap()
            );
        }
        (true, None) => {
            let result = FirecrawlApp::new_selfhosted(&api_url, None::<String>);
            assert!(matches!(
                result,
                Err(FirecrawlError::APIError(msg, _)) if msg == "Configuration"
            ));
        }
        (true, Some(key)) => {
            let result = FirecrawlApp::new_selfhosted(&api_url, Some(&key));
            assert!(result.is_ok());
        }
    }
}
