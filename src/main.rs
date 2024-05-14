mod llm_api;
mod paperless;
mod logger;

use ollama_rs::{
    Ollama,
};

use reqwest::{Client};
use std::result::Result;

//function that fetches data from the endpoint
//write function that queries a rest endpoint for a given url
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::env;
use crate::llm_api::generate_response;
use crate::paperless::{get_data_from_paperless, query_custom_fields, update_document_fields};
use substring::Substring;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Document {
    id: u32,
    correspondent: Option<u32>,
    document_type: Option<u32>,
    storage_path: Option<String>,
    title: String,
    content: String,
    created: String,
    created_date: Option<String>,
    modified: String,
    added: String,
    archive_serial_number: Option<String>,
    original_file_name: String,
    archived_file_name: Option<String>,
    owner: Option<u32>,
    notes: Vec<String>,
    tags: Vec<u32>,
    user_can_change: bool,
    custom_fields: Vec<CustomField>, // Change this to match the structure of the custom_fields array
}

#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    count: u32,
    next: Option<String>,
    previous: Option<String>,
    all: Vec<u32>,
    results: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CustomField {
    value: Option<Value>,
    field: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Field {
    id: u32,
    name: String,
    data_type: String,
}


// Initialize the HTTP client with Paperless API token and base URL
fn init_paperless_client(token: &str) -> Client {
    let mut headers = HeaderMap::new();
    let header_value = HeaderValue::from_str(&format!("Token {}", token))
        .expect("Invalid header value for TOKEN");
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
async fn process_documents(client: &Client, ollama: &Ollama, model: &str, base_url: &str, filter: &str) -> Result<(), Box<dyn std::error::Error>> {
    let prompt_base = env::var("BASE_PROMPT").unwrap_or_else(|_| "Please extract metadata\
     from the provided document and return it in JSON format.\
     The fields I need are:\
      title,topic,sender,recipient,urgency(with value either n/a or low or medium or high),\
      date_received,category.\
       Analyze the document to find the values for these fields and format the response as a \
       JSON object. Use the most likely answer for each field. \
       The response should contain only JSON data where the key and values are all in simple string \
       format(no nested object) for direct parsing by another program. So now additional text or \
       explanation, no introtext, the answer should start and end with curly brackets \
       delimiting the json object ".to_string()
    );
    let fields = query_custom_fields(client, base_url).await?;
    match get_data_from_paperless(&client, &base_url, filter).await {
        Ok(data) => {
            for document in data {
                slog_scope::trace!("Document Content: {}", document.content);
                slog_scope::info!("Generate Response with LLM {}", "model");
                slog_scope::debug!("with Prompt: {}", prompt_base);

                match generate_response(ollama, &model.to_string(), &prompt_base.to_string(), &document).await {
                    Ok(res) => {
                        // Log the response from the generate_response call
                        slog_scope::debug!("LLM Response: {}", res.response);

                        match extract_json_object(&res.response) {
                            Ok(json_str) => {
                                // Log successful JSON extraction
                                slog_scope::debug!("Extracted JSON Object: {}", json_str);

                                match serde_json::from_str(&json_str) {
                                    Ok(json) => update_document_fields(client, document.id, &fields, &json, base_url).await?,
                                    Err(e) => {
                                        slog_scope::error!("Error parsing llm response json {}", e.to_string());
                                        slog_scope::debug!("JSON String was: {}", &json_str);
                                    }
                                }
                            }
                            Err(e) => slog_scope::error!("{}", e),
                        }
                    },
                    Err(e) => {
                        slog_scope::error!("Error generating llm response: {}", e);
                        continue;
                    }
                }
            }
        },
        Err(e) => slog_scope::error!("Error while interacting with paperless: {}", e),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init(); // Initializes the global logger
    slog_scope::info!("Application started {}", "!");
    let token = env::var("PAPERLESS_TOKEN").expect("PAPERLESS_TOKEN is not set in .env file");
    let base_url = env::var("PAPERLESS_BASE_URL").expect("PAPERLESS_BASE_URL is not set in .env file");
    let client = init_paperless_client(&token);

    let ollama_host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string());
    let ollama_port = env::var("OLLAMA_PORT")
        .unwrap_or_else(|_| "11434".to_string())
        .parse::<u16>().unwrap_or(11434);
    let ollama_secure_endpoint = env::var("OLLAMA_SECURE_ENDPOINT")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>().unwrap_or(false);

    let ollama = init_ollama_client(&ollama_host, ollama_port, ollama_secure_endpoint);

    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2:13b".to_string());

    let default_filter = env::var("PAPERLESS_FILTER").unwrap_or_else(|_| "NOT tagged=true".to_string());

    process_documents(&client, &ollama, &model, &base_url, default_filter.as_str()).await
}

fn extract_json_object(input: &str) -> Result<String, String> {
    let mut brace_count = 0;
    let mut json_start = None;
    let mut json_end = None;

    for (i, c) in input.chars().enumerate() {
        match c {
            '{' | '[' => {
                if brace_count == 0 {
                    json_start = Some(i);
                }
                brace_count += 1;
            }
            '}' | ']' => {
                brace_count -= 1;
                if brace_count == 0 {
                    json_end = Some(i);
                    break; // Found the complete JSON object
                }
            }
            _ => {}
        }
    }

    if let (Some(start), Some(end)) = (json_start, json_end) {
        slog_scope::debug!("{}", input.substring(start, end + 1));
        Ok(input.substring(start, end + 1).to_string())
    } else {
        let error_msg = "No JSON object found in the response!".to_string();
        slog_scope::debug!("{}", error_msg);
        Err(error_msg)
    }
}

