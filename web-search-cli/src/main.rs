mod brave;
mod cache;
mod exa;
mod rate_limit;
mod search_service;
mod types;

use clap::Parser;
use crate::exa::ExaClient;
use crate::types::{WebSearchRequest, WebSearchAPIEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "exa")]
    provider: String,

    #[arg(long)]
    query: String,

    #[arg(long, default_value_t=3)]
    limit: i32,

    #[arg(long, default_value="auto")]
    search_type: String,

    #[arg(long)]
    summary_query: Option<String>,

    #[arg(long, default_value_t=false)]
    full_text: bool,
}

#[tokio::main]
async fn main() {
    colog::init();
    let args = Args::parse();

    let client = match args.provider.as_str() {
        "exa" => WebSearchAPIEnum::Exa(ExaClient::new()),
        "brave" | _ => WebSearchAPIEnum::Brave(brave::BraveClient::new()),
    };

    let service = search_service::SearchService::new(client);

    let req = WebSearchRequest {
        query: args.query,
        limit: args.limit,
        summary_query: args.summary_query,
        full_text: args.full_text,
        search_type: args.search_type,
    };
    match service.perform_web_search(req).await {
        Ok(response) => {
            service.save_to_disk().unwrap();
            println!("{}", response);
            let j: serde_json::Value = serde_json::from_str(&response).unwrap();
            let pretty = serde_json::to_string_pretty(&j).unwrap();
            println!("{}", pretty)
        },
        Err(e) => eprintln!("Error performing web search: {}", e),
    }
}
