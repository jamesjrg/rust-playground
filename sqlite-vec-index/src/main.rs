use std::env;
use async_trait;
use crate::embedder::embedder::LocalEmbedder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_path = env::current_dir().unwrap();

    let embedder = LocalEmbedder::new(&current_path.join("onnx_models/all-MiniLM-L6-v2/")).unwrap();
    println!("LocalEmbedder initialized successfully");
    Ok(())
}
