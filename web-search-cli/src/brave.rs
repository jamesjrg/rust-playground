use std::env;
use anyhow::Result;
use async_trait::async_trait;

use crate::types::{WebSearchAPI, WebSearchRequest};


#[derive(Clone)]
pub struct BraveClient {
    client: reqwest::Client,
}

impl BraveClient {
    const WEB_URL: &str = "https://api.search.brave.com/res/v1/web/search";
    //const SUMMARIZER_URL: &str = "https://api.search.brave.com/res/v1/summarizer/search";

    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn api_key(&self) -> Result<String, anyhow::Error> {
        env::var("AIDE_BRAVE_API_KEY")
        .map_err(|_| anyhow::anyhow!("Missing AIDE_BRAVE_API_KEY"))
    }
}

#[async_trait]
impl WebSearchAPI for BraveClient {
    fn cache_prefix(&self) -> &'static str {
        "brave::"
    }

    async fn perform_web_search(
        &self,
        request: WebSearchRequest,
    ) -> Result<String> {
        let access_token = self.api_key()?;

        let response = self
            .client
            .get(Self::WEB_URL)
            .header("Accept-Encoding", "gzip")
            .header("Accept", "application/json")
            .header("X-Subscription-Token", access_token)
            .query(&[("q", request.query), ("count", request.limit.to_string())])
            .send()
            .await?;

        Ok(response.text().await?)
    }
}
