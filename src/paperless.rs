use std::collections::HashMap;
use std::fmt;
use reqwest::Client;
use serde::de::StdError;
use serde_json::Value;
use crate::{CustomField, Document, Field, Response};

pub async fn get_data_from_paperless(
    client: &Client,
    url: &str,
    filter: &str,
) -> Result<Vec<Document>, Box<dyn StdError + Send + Sync>> {
    // Read token from environment
    //Define filter string
    let filter = filter;
    slog_scope::info!("Retrieve Documents from paperless at: {}, with query: {}",url, filter);
    let response = client.get(format!("{}/api/documents/?query={}", url, filter)).send().await?;


    let response_result = response.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::trace!("Response from server while fetching documents: {}", body);

            // Remove the "Document content: " prefix
            let json = body.trim_start_matches("Document content: ");
            //println!("{}",json);
            // Parse the JSON string into a generic JSON structure
            //let value: serde_json::Value = serde_json::from_str(json).unwrap();

            // Print the part of the JSON structure that's causing the error
            //let error_part = value.pointer("/results/0").unwrap();
            //println!("Error part: {}", error_part);
            // Parse the JSON string into the Response struct
            let data: Result<Response<Document>, _> = serde_json::from_str(json);
            match data {
                Ok(data) => {
                    slog_scope::info!("Successfully retrieved {} Documents", data.results.len());
                    Ok(data.results)
                }
                Err(e) => {
                    let column = e.column();
                    let start = (column as isize - 30).max(0) as usize;
                    let end = (column + 30).min(json.len());
                    slog_scope::error!("Error while creating json of document response from paperless {}", e);
                    slog_scope::error!("Error at column {}: {}", column, &json[start..end]);
                    slog_scope::trace!("Error occured in json {}", &json);
                    Err(e.into()) // Remove the semicolon here
                }
            }
        }
        Err(e) => {
            slog_scope::error!("Error while fetching documents from paperless: {}",e);
            Err(e.into())
        }
    }
}

pub async fn query_custom_fields(
    client: &Client,
    base_url: &str,
) -> Result<Vec<Field>, Box<dyn std::error::Error>> {
    slog_scope::info!("Fetching custom fields from paperless at {}", base_url);
    let res = client
        .get(format!("{}/api/custom_fields/", base_url))
        .send()
        .await?;

    let response_result = res.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::debug!("Response from server while fetching documents: {}", body);

            // Remove the "Document content: " prefix
            let json = body.trim_start_matches("Field: ");
            let data: Result<Response<Field>, _> = serde_json::from_str(json);
            match data {
                Ok(data) => {
                    slog_scope::info!("Fields: {:?}", data.results);
                    Ok(data.results)
                }
                Err(e) => {
                    let column = e.column();
                    let start = (column as isize - 30).max(0) as usize;
                    let end = (column + 30).min(json.len());
                    slog_scope::error!("Error occured parsing custom fields: {}", e);
                    slog_scope::error!("Error at column {}: {}", column, &json[start..end]);
                    slog_scope::debug!("Error occured in json {}", &json);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            slog_scope::error!("Error retrieving custom fields: {}", e);
            Err(e.into())
        }
    }
}

pub async fn update_document_fields(
    client: &Client,
    document_id: u32,
    fields: &Vec<Field>,
    metadata: &HashMap<String, Option<Value>>,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut custom_fields = Vec::new();

// Use `if let` to conditionally execute code if the 'tagged' field is found.
    if let Some(field) = fields.iter().find(|&f| f.name == "tagged") {
        let tagged_field = CustomField {
            field: field.id,
            value: Some(serde_json::json!(true)),
        };

        // Add this tagged_field to your custom_fields collection or use it as needed.
        custom_fields.push(tagged_field);

        // Continue with your logic, such as preparing the payload and sending the request.
    } else {
        // Handle the case where tagged_field_id is None, which means the "tagged" field wasn't found.
        slog_scope::error!("{} field not found in the provided fields.", "'tagged'");
        return Err(Box::new(fmt::Error::default())); // Use a standard library error type like fmt::Error.

    }
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
// Check if tagged_field_id has a value and then proceed.
     
    let mut payload = serde_json::Map::new();

    payload.insert("custom_fields".to_string(), serde_json::json!(custom_fields));
    if let Some(value) = metadata.get("title").and_then(|v| v.as_ref().and_then(|v| v.as_str())) {
        payload.insert("title".to_string(), serde_json::json!(value));
    }
    if payload.is_empty() {
        slog_scope::warn!("{}", "payload is empty, not updating fields");
        return Err(Box::new(fmt::Error::default())); // Use a standard library error type like fmt::Error.
    }
    let url = format!("{}/api/documents/{}/", base_url, document_id);
    slog_scope::info!("Updating document with ID: {}", document_id);
    slog_scope::debug!("Request Payload: {}", "");
    for (key, value) in &payload {
        slog_scope::debug!("{}: {}", key, value);
    }
    let res = client.patch(&url).json(&payload).send().await?;
    let response_result = res.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::debug!("{}", body);
            slog_scope::info!("Document with ID: {} successfully updated", document_id);
            Ok(())
        }
        Err(e) => {
            slog_scope::error!("Error while updating document fields: {}", e);
            Err(e.into())
        }
    }
}