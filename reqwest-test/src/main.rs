mod reqwest_tee_middleware;

use reqwest_middleware::{ClientBuilder, Error};
use reqwest_tee_middleware::TeeMiddleware;

#[tokio::main]
async fn main() -> Result<(), Error> {

    let client: reqwest_middleware::ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
        .with(TeeMiddleware::new("http://localhost:8080"))
        .build();

    let body = client
    .get("https://www.rust-lang.org/cheese")
    .send()
    .await?
    .text()
    .await?;

    println!("body = {body:?}");

    Ok(())
}
