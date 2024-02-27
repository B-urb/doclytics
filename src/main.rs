mod llm_api;
mod paperless;

use ollama_rs::{
    Ollama,
};
use substring::Substring;

use reqwest::{Client, };
use std::result::Result;

//function that fetches data from the endpoint
//write function that queries a rest endpoint for a given url
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::collections::HashMap;
use std::env;
use std::error::Error as StdError;
use crate::llm_api::generate_response;
use crate::paperless::{get_data_from_paperless, query_custom_fields};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Document {
    id: u32,
    correspondent: Option<String>,
    document_type: Option<String>,
    storage_path: Option<String>,
    title: String,
    content: String,
    created: String,
    created_date: String,
    modified: String,
    added: String,
    archive_serial_number: Option<String>,
    original_file_name: String,
    archived_file_name: String,
    owner: u32,
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




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("PAPERLESS_BASE_URL").unwrap();

    let token = env::var("PAPERLESS_TOKEN").expect("TOKEN is not set in .env file");
    // Create HeaderMap and add Authorization header
    let mut headers = HeaderMap::new();
    let header_value = HeaderValue::from_str(&format!("Token {}", token)).unwrap();
    headers.insert(AUTHORIZATION, header_value);
    let client = Client::builder().default_headers(headers).build().unwrap();

    // Create a Client with the default headers
    let ollama = Ollama::new("http://localhost".to_string(), 11434);
    //let model = "mistral:latest".to_string();
    let model = "llama2:13b".to_string();
    let prompt_base = "Please extract metadata from the provided document and return it in JSON format. The fields I need are: title,topic,sender,recipient,urgency(with value either n/a or low or medium or high),date_received,category. Analyze the document to find the values for these fields and format the response as a JSON object. Use the most likely answer for each field. The response should contain only JSON data where the key and values are all in simple string format(no nested object) for direct parsing by another program. So now additional text or explanation, no introtext, the answer should start and end with curly brackets delimiting the json object ".to_string();

    let fields = query_custom_fields(&client, &base_url).await?;
    //let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

    // if let Ok(res) = res {
    //     println!("{}", res.response);
    // }

    // Query data from paperless-ngx endpoint
    match get_data_from_paperless(&client, &base_url).await {
        Ok(data) => {
            for document in data {
                let res = generate_response(&ollama, &model, &prompt_base, &document).await;
                if let Ok(res) = res {
                    println!("Response: {}", res.response);
                    if let Some(json_str) = extract_json_object(&res.response) {
                        println!("JSON: {}", json_str);
                        let parsed_json = serde_json::from_str(&json_str);
                        match parsed_json {
                            Ok(json) => {
                                update_document_fields(&client, document.id, &fields, &json, &base_url).await;
                                // Use the parsed JSON here
                            }
                            Err(e) => {
                                eprintln!("Error parsing JSON: {}", e);
                            }
                        }
                    } else {
                        eprintln!("No JSON object found in the response");
                    }
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}

async fn update_document_fields(
    client: &Client,
    document_id: u32,
    fields: &Vec<Field>,
    metadata: &HashMap<String, Option<Value>>,
    base_url: &str
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut custom_fields = Vec::new();

    for (key, value) in metadata {
        if key == "title" {
            continue;
        }
        if let Some(field) = fields.iter().find(|&f| f.name == *key) {
            let custom_field = CustomField {
                field: field.id.clone(),
                value: value.as_ref().cloned(),
            };
            custom_fields.push(custom_field);
        }
    }
    // Add the tagged field, to indicate that the document has been processed
    let custom_field = CustomField {
        field: 1,
        value: Some(serde_json::json!(true)),
    };
    custom_fields.push(custom_field);
    let mut payload = serde_json::Map::new();

    payload.insert("custom_fields".to_string(), serde_json::json!(custom_fields));
    if let Some(value) = metadata.get("title").and_then(|v| v.as_ref().and_then(|v| v.as_str())) {
        payload.insert("title".to_string(), serde_json::json!(value));
    }
    let url = format!("{}/api/documents/{}/", base_url, document_id);
    let res = client.patch(&url).json(&payload).send().await?;
    let body = res.text().await?;
    println!("{}", body);
    Ok(())
}

fn extract_json_object(input: &str) -> Option<String> {
    println!("Input: {}", input);
    let mut brace_count = 0;
    let mut json_start = None;
    let mut json_end = None;

    for (i, c) in input.chars().enumerate() {
        match c {
            '{' | '[' => {
                brace_count += 1;
                if json_start.is_none() {
                    json_start = Some(i);
                }
            }
            '}' | ']' => {
                if brace_count > 0 {
                    brace_count -= 1;
                    if brace_count == 0 {
                        json_end = Some(i); // Include the closing brace
                    }
                }
            }
            _ => {}
        }
    }

    if let (Some(start), Some(end)) = (json_start, json_end) {
        println!("{}", input.substring(start, end + 1));
        Some(input.substring(start, end + 1).to_string()) // Use end with equal sign
    } else {
        None
    }
}
