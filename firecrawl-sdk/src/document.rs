use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "mcp_tool")]
use schemars::JsonSchema;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mcp_tool", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DocumentMetadata {
    // Required fields from the API
    #[serde(rename = "sourceURL")]
    pub source_url: String,
    pub status_code: u16,
    pub error: Option<String>,

    // Common metadata fields - all are optional and can be either strings or arrays
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_vec")]
    pub title: Option<String>,
    
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_vec")]
    pub description: Option<String>,
    
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_vec")]
    pub language: Option<String>,
    
    // All other metadata fields are captured here
    #[serde(flatten)]
    pub additional_fields: std::collections::HashMap<String, Value>,
}

// Helper function to deserialize a field that could be either a string or an array of strings
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> serde::de::Visitor<'de> for StringOrVec {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            // Take the first element if it's an array
            if let Some(first) = seq.next_element::<String>()? {
                return Ok(Some(first));
            }
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

/// Represents a scrape result from an action
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeActionResult {
    /// The URL that was scraped
    pub url: String,
    /// The HTML content of the scraped URL
    pub html: String,
}

/// Represents a JavaScript return value from an action
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JavaScriptReturnValue {
    /// The type of the returned value
    #[serde(rename = "type")]
    pub value_type: String,
    /// The actual value returned
    pub value: Value,
}

/// Represents the results of actions performed during scraping
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionResults {
    /// URLs to screenshots taken during actions
    pub screenshots: Option<Vec<String>>,
    /// Results of scrape actions
    pub scrapes: Option<Vec<ScrapeActionResult>>,
    /// Results of JavaScript execution actions
    pub javascript_returns: Option<Vec<JavaScriptReturnValue>>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// A list of the links on the page, present if `ScrapeFormats::Markdown` is present in `ScrapeOptions.formats`. (default)
    pub markdown: Option<String>,

    /// The HTML of the page, present if `ScrapeFormats::HTML` is present in `ScrapeOptions.formats`.
    ///
    /// This contains HTML that has non-content tags removed. If you need the original HTML, use `ScrapeFormats::RawHTML`.
    pub html: Option<String>,

    /// The raw HTML of the page, present if `ScrapeFormats::RawHTML` is present in `ScrapeOptions.formats`.
    ///
    /// This contains the original, untouched HTML on the page. If you only need human-readable content, use `ScrapeFormats::HTML`.
    pub raw_html: Option<String>,

    /// The URL to the screenshot of the page, present if `ScrapeFormats::Screenshot` or `ScrapeFormats::ScreenshotFullPage` is present in `ScrapeOptions.formats`.
    pub screenshot: Option<String>,

    /// A list of the links on the page, present if `ScrapeFormats::Links` is present in `ScrapeOptions.formats`.
    pub links: Option<Vec<String>>,

    /// The extracted data from the page, present if `ScrapeFormats::Extract` is present in `ScrapeOptions.formats`.
    /// If `ScrapeOptions.extract.schema` is `Some`, this `Value` is guaranteed to match the provided schema.
    #[serde(alias = "llm_extraction")]
    pub extract: Option<Value>,

    /// The structured JSON data from the page, present if `ScrapeFormats::JSON` is present in `ScrapeOptions.formats`.
    /// If `ScrapeOptions.jsonOptions.schema` is `Some`, this `Value` is guaranteed to match the provided schema.
    pub json: Option<Value>,

    /// Results of actions performed during scraping, present if `actions` parameter was provided in the request.
    pub actions: Option<ActionResults>,

    /// The metadata from the page.
    pub metadata: DocumentMetadata,

    /// Can be present if `ScrapeFormats::Extract` is present in `ScrapeOptions.formats`.
    /// The warning message will contain any errors encountered during the extraction.
    pub warning: Option<String>,
}
