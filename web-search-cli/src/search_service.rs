use crate::{cache::{CachedResponse, WebSearchCache}, rate_limit::check_rate_limit, types::{WebSearchAPI, WebSearchAPIEnum, WebSearchRequest}};
use anyhow::Result;

pub struct SearchService {
    client: WebSearchAPIEnum,
    cache: WebSearchCache,
}

impl SearchService {
    pub fn new(client: WebSearchAPIEnum) -> Self {
        Self { client, cache: WebSearchCache::with_cache_file("cache.json") }
    }

    pub fn save_to_disk(&self) -> Result<(), anyhow::Error> {
        self.cache.save_to_disk()
    }

    pub async fn perform_web_search(&self, req: WebSearchRequest) -> Result<String> {

        let cache_key = self.client.cache_prefix().to_owned() + &req.query;
        let cached = self.cache.get(&cache_key);

        if let Some(cached_value) = cached {
            println!("Cache hit");
            return Ok(cached_value.response.clone());
        }

        // this is kind of pointless at the moment
        check_rate_limit()?;

        let text = self.client.perform_web_search(req).await?;

        let cached_response = CachedResponse {
            response: text.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.cache.set(cache_key, cached_response);
        Ok(text)
    }
}