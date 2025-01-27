use std::env;
use anyhow::Result;
use async_trait::async_trait;

use crate::types::{WebSearchAPI, WebSearchRequest};

#[derive(serde::Serialize, Debug, Clone)]
struct Summary {
    query: Option<String>,
}

#[derive(serde::Serialize, Debug, Clone)]
struct Contents {
    text: bool,
    summary: Summary,
    // There are  other options, see the Exa API documentation
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ExaSearchRequest {
    query: String,
    num_results: i32,
    contents: Contents,
    r#type: String,
    // There are many other options, see the Exa API documentation
}

impl ExaSearchRequest {
    fn from_web_search_request(request: WebSearchRequest) -> Self {
        ExaSearchRequest {
            query: request.query,
            num_results: request.limit,
            r#type: request.search_type,
            contents: Contents {
                text: request.full_text,
                summary: Summary {
                    query: request.summary_query
                },
            },
        }
    }
}

#[derive(Clone)]
pub struct ExaClient {
    client: reqwest::Client,
}

impl ExaClient {
    const API_URL: &str = "https://api.exa.ai/search";

    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn api_key(&self) -> Result<String, anyhow::Error> {
        env::var("AIDE_EXA_API_KEY")
        .map_err(|_| anyhow::anyhow!("Missing AIDE_EXA_API_KEY"))
    }
}

#[async_trait]
impl WebSearchAPI for ExaClient {
    fn cache_prefix(&self) -> &'static str {
        "exa::"
    }

    async fn perform_web_search(
        &self,
        request: WebSearchRequest,
    ) -> Result<String> {
        let access_token = self.api_key()?;

        // TODO headers and request structure
        let request = ExaSearchRequest::from_web_search_request(request);
        let response = self
            .client
            .post(Self::API_URL)
            .header("x-api-key", access_token)
            .json(&request)
            .send()
            .await?;

        Ok(response.text().await?)

    }
}
