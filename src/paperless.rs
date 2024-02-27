use reqwest::Client;
use serde::de::StdError;
use crate::{Document, Field, Response};

pub async fn get_data_from_paperless(
    client: &Client,
    url: &str,
) -> std::result::Result<Vec<Document>, Box<dyn StdError + Send + Sync>> {
    // Read token from environment
    //Define filter string
    let filter = "NOT tagged=true".to_string();

    let response = client.get(format!("{}/api/documents/?query={}", url, filter)).send().await?;
    let body = response.text().await?;

    // Remove the "Document content: " prefix
    let json = body.trim_start_matches("Document content: ");
    //println!("{}",json);
    // Parse the JSON string into a generic JSON structure
    //let value: serde_json::Value = serde_json::from_str(json).unwrap();

    // Print the part of the JSON structure that's causing the error
    //let error_part = value.pointer("/results/0").unwrap();
    //println!("Error part: {}", error_part);
    // Parse the JSON string into the Response struct
    let data: std::result::Result<Response<Document>, _> = serde_json::from_str(json);
    match data {
        Ok(data) => Ok(data.results),
        Err(e) => {
            let column = e.column();
            let start = (column as isize - 30).max(0) as usize;
            let end = (column + 30).min(json.len());
            println!("Error at column {}: {}", column, &json[start..end]);
            Err(e.into()) // Remove the semicolon here
        }
    }
}
pub async fn query_custom_fields(
    client: &Client,
    base_url: &str,
) -> std::result::Result<Vec<Field>, Box<dyn std::error::Error>> {
    let res = client
        .get(format!("{}/api/custom_fields/", base_url))
        .send()
        .await?;
    let body = res.text().await?;
    // Remove the "Document content: " prefix
    let json = body.trim_start_matches("Field: ");
    let data: std::result::Result<Response<Field>, _> = serde_json::from_str(json);
    match data {
        Ok(data) => {
            println!("Fields: {:?}", data.results);
            Ok(data.results)
        },
        Err(e) => {
            let column = e.column();
            let start = (column as isize - 30).max(0) as usize;
            let end = (column + 30).min(json.len());
            println!("Error at column {}: {}", column, &json[start..end]);
            Err(e.into()) // Remove the semicolon here
        }
    }
}