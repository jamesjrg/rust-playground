use async_trait::async_trait;
use anyhow::Result;

use crate::{brave::BraveClient, exa::ExaClient};

pub struct WebSearchRequest {
    pub query: String,
    pub limit: i32,
    pub summary_query: Option<String>,
    pub full_text: bool,
    pub search_type: String,
}

#[derive(Clone)]
pub enum WebSearchAPIEnum {
    Brave(BraveClient),
    Exa(ExaClient),
}

#[async_trait]
pub trait WebSearchAPI {
    async fn perform_web_search(
        &self,
        request: WebSearchRequest,
    ) -> Result<String>;

    fn cache_prefix(&self) -> &'static str;
}

#[async_trait]
impl WebSearchAPI for WebSearchAPIEnum {
    async fn perform_web_search(&self, request: WebSearchRequest) -> Result<String> {
        match self {
            WebSearchAPIEnum::Brave(client) => client.perform_web_search(request).await,
            WebSearchAPIEnum::Exa(client) => client.perform_web_search(request).await,
        }
    }

    fn cache_prefix(&self) -> &'static str {
        match self {
            WebSearchAPIEnum::Brave(client) => client.cache_prefix(),
            WebSearchAPIEnum::Exa(client) => client.cache_prefix(),
        }
    }
}
