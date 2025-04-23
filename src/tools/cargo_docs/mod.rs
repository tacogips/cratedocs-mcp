mod service_impl;

use std::sync::Arc;

use html2md::parse_html;

use reqwest::Client;
use tokio::sync::Mutex;

use rmcp::{model::*, schemars, tool, ServerHandler};

// Cache for documentation lookups to avoid repeated requests
#[derive(Clone)]
pub struct DocCache {
    cache: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl Default for DocCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DocCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get(key).cloned()
    }

    pub async fn set(&self, key: String, value: String) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, value);
    }
}

#[derive(Clone)]
pub struct CargoDocRouter {
    pub client: Client,
    pub cache: DocCache,
}

impl Default for CargoDocRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[tool(tool_box)]
impl CargoDocRouter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: DocCache::new(),
        }
    }

    #[tool(description = "Look up documentation for a Rust crate (returns markdown)")]
    async fn lookup_crate(
        &self,
        #[tool(param)]
        #[schemars(description = "The name of the crate to look up")]
        crate_name: String,

        #[tool(param)]
        #[schemars(description = "The version of the crate (optional, defaults to latest)")]
        version: Option<String>,
    ) -> String {
        // Check cache first
        let cache_key = if let Some(ver) = &version {
            format!("{}:{}", crate_name, ver)
        } else {
            crate_name.clone()
        };

        if let Some(doc) = self.cache.get(&cache_key).await {
            return doc;
        }

        // Construct the docs.rs URL for the crate
        let url = if let Some(ver) = version {
            format!("https://docs.rs/crate/{}/{}/", crate_name, ver)
        } else {
            format!("https://docs.rs/crate/{}/", crate_name)
        };

        // Fetch the documentation page
        let response = match self
            .client
            .get(&url)
            .header(
                "User-Agent",
                "CrateDocs/0.1.0 (https://github.com/d6e/cratedocs-mcp)",
            )
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => return format!("Failed to fetch documentation: {}", e),
        };

        if !response.status().is_success() {
            return format!(
                "Failed to fetch documentation. Status: {}",
                response.status()
            );
        }

        let html_body = match response.text().await {
            Ok(body) => body,
            Err(e) => return format!("Failed to read response body: {}", e),
        };

        // Convert HTML to markdown
        let markdown_body = parse_html(&html_body);

        // Cache the markdown result
        self.cache.set(cache_key, markdown_body.clone()).await;

        markdown_body
    }

    #[tool(
        description = "Look up documentation for a specific item in a Rust crate (returns markdown)"
    )]
    async fn lookup_item_tool(
        &self,
        #[tool(param)]
        #[schemars(description = "The name of the crate")]
        crate_name: String,

        #[tool(param)]
        #[schemars(
            description = "Path to the item (e.g., 'vec::Vec' or 'crate_name::vec::Vec' - crate prefix will be automatically stripped)"
        )]
        item_path: String,

        #[tool(param)]
        #[schemars(description = "The version of the crate (optional, defaults to latest)")]
        version: Option<String>,
    ) -> String {
        self.lookup_item(crate_name, item_path, version).await
    }

    #[tool(description = "Search for Rust crates on crates.io (returns JSON or markdown)")]
    async fn search_crates(
        &self,
        #[tool(param)]
        #[schemars(description = "The search query")]
        query: String,

        #[tool(param)]
        #[schemars(
            description = "Maximum number of results to return (optional, defaults to 10, max 100)"
        )]
        limit: Option<u32>,
    ) -> String {
        let limit = limit.unwrap_or(10).min(100); // Cap at 100 results

        let url = format!(
            "https://crates.io/api/v1/crates?q={}&per_page={}",
            query, limit
        );

        let response = match self
            .client
            .get(&url)
            .header(
                "User-Agent",
                "CrateDocs/0.1.0 (https://github.com/d6e/cratedocs-mcp)",
            )
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => return format!("Failed to search crates.io: {}", e),
        };

        if !response.status().is_success() {
            return format!("Failed to search crates.io. Status: {}", response.status());
        }

        let body = match response.text().await {
            Ok(text) => text,
            Err(e) => return format!("Failed to read response body: {}", e),
        };

        // Check if response is JSON (API response) or HTML (web page)
        if body.trim().starts_with('{') {
            // This is likely JSON data, return as is
            body
        } else {
            // This is likely HTML, convert to markdown
            parse_html(&body)
        }
    }

    // This function is not directly exposed as a tool but used internally
    async fn lookup_item(
        &self,
        crate_name: String,
        mut item_path: String,
        version: Option<String>,
    ) -> String {
        // Strip crate name prefix from the item path if it exists
        let crate_prefix = format!("{}::", crate_name);
        if item_path.starts_with(&crate_prefix) {
            item_path = item_path[crate_prefix.len()..].to_string();
        }

        // Check cache first
        let cache_key = if let Some(ver) = &version {
            format!("{}:{}:{}", crate_name, ver, item_path)
        } else {
            format!("{}:{}", crate_name, item_path)
        };

        if let Some(doc) = self.cache.get(&cache_key).await {
            return doc;
        }

        // Process the item path to determine the item type
        // Format: module::path::ItemName
        // Need to split into module path and item name, and guess item type
        let parts: Vec<&str> = item_path.split("::").collect();

        if parts.is_empty() {
            return "Invalid item path. Expected format: module::path::ItemName".to_string();
        }

        let item_name = parts.last().unwrap().to_string();
        let module_path = if parts.len() > 1 {
            parts[..parts.len() - 1].join("/")
        } else {
            String::new()
        };

        // Try different item types (struct, enum, trait, fn)
        let item_types = ["struct", "enum", "trait", "fn", "macro"];
        let mut last_error = None;

        for item_type in item_types.iter() {
            // Construct the docs.rs URL for the specific item
            let url = if let Some(ver) = version.clone() {
                if module_path.is_empty() {
                    format!(
                        "https://docs.rs/{}/{}/{}/{}.{}.html",
                        crate_name, ver, crate_name, item_type, item_name
                    )
                } else {
                    format!(
                        "https://docs.rs/{}/{}/{}/{}/{}.{}.html",
                        crate_name, ver, crate_name, module_path, item_type, item_name
                    )
                }
            } else if module_path.is_empty() {
                format!(
                    "https://docs.rs/{}/latest/{}/{}.{}.html",
                    crate_name, crate_name, item_type, item_name
                )
            } else {
                format!(
                    "https://docs.rs/{}/latest/{}/{}/{}.{}.html",
                    crate_name, crate_name, module_path, item_type, item_name
                )
            };

            // Try to fetch the documentation page
            let response = match self
                .client
                .get(&url)
                .header(
                    "User-Agent",
                    "CrateDocs/0.1.0 (https://github.com/d6e/cratedocs-mcp)",
                )
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = Some(e.to_string());
                    continue;
                }
            };

            // If found, process and return
            if response.status().is_success() {
                let html_body = match response.text().await {
                    Ok(body) => body,
                    Err(e) => return format!("Failed to read response body: {}", e),
                };

                // Convert HTML to markdown
                let markdown_body = parse_html(&html_body);

                // Cache the markdown result
                self.cache.set(cache_key, markdown_body.clone()).await;

                return markdown_body;
            }

            last_error = Some(format!("Status code: {}", response.status()));
        }

        // If we got here, none of the item types worked
        format!(
            "Failed to fetch item documentation. No matching item found. Last error: {}",
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )
    }
}

#[tool(tool_box)]
impl ServerHandler for CargoDocRouter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Rust Documentation MCP Server for accessing Rust crate documentation.".to_string(),
            ),
        }
    }
}
