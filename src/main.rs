mod doclytics;
mod error;
mod llm_api;
mod logger;
mod paperless;
mod paperless_defaultfields;
mod server;
mod types;
mod util;

use ollama_rs::Ollama;

use reqwest::Client;
use std::result::Result;

use crate::server::init_server;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::env;

// Initialize the HTTP client with Paperless API token and base URL
fn init_paperless_client(token: &str) -> Client {
    let mut headers = HeaderMap::new();
    let header_value =
        HeaderValue::from_str(&format!("Token {}", token)).expect("Invalid header value for TOKEN");
    headers.insert(AUTHORIZATION, header_value);

    Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client")
}

// Initialize Ollama client
fn init_ollama_client(host: &str, port: u16, secure_endpoint: bool) -> Ollama {
    let protocol = if secure_endpoint { "https" } else { "http" };
    let ollama_base_url = format!("{}://{}", protocol, host);
    Ollama::new(ollama_base_url, port)
}

// Refactor the main process into a function for better readability

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init(); // Initializes the global logger
    slog_scope::info!(
        "Application started, version: {}",
        env!("CARGO_PKG_VERSION")
    );
    let token = env::var("PAPERLESS_TOKEN").expect("PAPERLESS_TOKEN is not set in .env file");
    let base_url =
        env::var("PAPERLESS_BASE_URL").expect("PAPERLESS_BASE_URL is not set in .env file");
    let client = init_paperless_client(&token);

    let ollama_host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string());
    let ollama_port = env::var("OLLAMA_PORT")
        .unwrap_or_else(|_| "11434".to_string())
        .parse::<u16>()
        .unwrap_or(11434);
    let ollama_secure_endpoint = env::var("OLLAMA_SECURE_ENDPOINT")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let ollama = init_ollama_client(&ollama_host, ollama_port, ollama_secure_endpoint);

    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2:13b".to_string());

    let default_filter =
        env::var("PAPERLESS_FILTER").unwrap_or_else(|_| "NOT tagged=true".to_string());

    //TODO: IF enabled
    let router = init_server().await;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 3000))
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
    doclytics::process_documents(&client, &ollama, &model, &base_url, default_filter.as_str()).await
}
