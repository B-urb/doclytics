use std::collections::HashMap;
use reqwest::Client;
use serde::de::StdError;
use serde_json::Value;
use crate::{CustomField, Document, Field, Response};

pub async fn get_data_from_paperless(
    client: &Client,
    url: &str,
) -> std::result::Result<Vec<Document>, Box<dyn StdError + Send + Sync>> {
    // Read token from environment
    //Define filter string
    let filter = "NOT tagged=true".to_string();

    let response = client.get(format!("{}/api/documents/?query={}", url, filter)).send().await?;


    let response_result = response.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::debug!("Response from server while fetching documents: {}", body);

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
                    slog_scope::error!("Error occured while fetching documents from paperless {}", e);
                    slog_scope::error!("Error at column {}: {}, \n in json {}", column, &json[start..end], &json);
                    Err(e.into()) // Remove the semicolon here
                }
            }
        }
        Err(e) => {
            slog_scope::error!("{}",e);
            Err(e.into())
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

        let response_result = res.error_for_status();
        match response_result {
            Ok(data) => {
                let body = data.text().await?;
                slog_scope::debug!("Response from server while fetching documents: {}", body);

                // Remove the "Document content: " prefix
                let json = body.trim_start_matches("Field: ");
                let data: std::result::Result<Response<Field>, _> = serde_json::from_str(json);
                match data {
                    Ok(data) => {
                        slog_scope::info!("Fields: {:?}", data.results);
                        Ok(data.results)
                    }
                    Err(e) => {
                        let column = e.column();
                        let start = (column as isize - 30).max(0) as usize;
                        let end = (column + 30).min(json.len());
                        slog_scope::error!("Error occured while fetching custom fields: {}", e);
                        slog_scope::error!("Error at column {}: {}, \n in json {}", column, &json[start..end], &json);
                        Err(e.into()) 
                    }
                }
            }
            Err(e) => {
                slog_scope::error!("{}", e);
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
        let response_result = res.error_for_status();
        match response_result {
            Ok(data) => {
                let body = data.text().await?;
                slog_scope::debug!("{}", body);
                Ok(())
            }
            Err(e) => {
                slog_scope::error!("{}", e);
                Err(e.into())
            }
        }
    }