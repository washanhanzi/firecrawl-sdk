# Firecrawl SDK

## [firecrawl-sdk](./firecrawl-sdk)

The core Rust SDK for interacting with the Firecrawl API. Provides functionality for web scraping, crawling, searching, and more.

Similar to the official SDK and mimic the JS SDK api.

## [firecrawl-mcp](./firecrawl-mcp)

A Model Context Protocol (MCP) server implementation that exposes Firecrawl functionality to AI models through various transport mechanisms.

### features

- default: include all tools
- batch_scrape: include batch scrape tool
- crawl: include crawl tool
- map: include map tool
- scrape: include scrape tool
- search: include search tool

### Example

- stdio transport
```bash
cargo run --package firecrawl-mcp --bin std_io
```

- sse transport
```bash
cargo run --package firecrawl-mcp --bin sse
```

- sse transport build with only scrape tool
```bash
cargo build --package firecrawl-mcp --bin sse --no-default-features --features scrape
```

- for in-process transport, use [rmcp-in-process-transport](https://github.com/washanhanzi/rmcp-in-process-transport)