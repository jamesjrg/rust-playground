use reqwest::{Client, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use http::Extensions;
use futures::join;

pub struct TeeMiddleware {
    base_url: String,
    tee_client: Client,
}

impl TeeMiddleware {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            tee_client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
 impl Middleware for TeeMiddleware {
     async fn handle(
         &self,
         req: Request,
         extensions: &mut Extensions,
         next: Next<'_>,
     ) -> Result<Response> {

        let path = req.url().path();
        let full_url = format!("{}{}", self.base_url, path);

        let body = req.try_clone().and_then(|req| {
            req.body().map(|body| {
                body.as_bytes().map_or_else(
                    || Vec::new(),
                    |bytes| bytes.to_vec()
                )
            })
        });

        let tee_request = self.tee_client
            .request(req.method().clone(), full_url)
            .headers(req.headers().clone());

        let tee_request = if let Some(body) = body {
            tee_request.body(body)
        } else {
            tee_request
        };

        let (r_a, _r_b) = join!(next.run(req, extensions), tee_request.send());


        r_a
     }
 }