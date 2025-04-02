pub mod crawl;
pub mod scrape;
pub mod search;

use firecrawl_sdk::FirecrawlApp;

#[derive(Clone)]
pub struct Controller {
    client: FirecrawlApp,
}

impl Controller {
    pub fn new(client: FirecrawlApp) -> Self {
        Self { client }
    }
}
