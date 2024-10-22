use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use reqwest::Client;
use serde::de::{DeserializeOwned, StdError};
use serde_json::{Map, Value};
use crate::{CustomField, Document, Field, Mode, Response};
use serde::{Deserialize, Serialize};
use crate::util::normalize_string;

#[derive(Clone, Copy)]
pub enum PaperlessDefaultFieldType {
    Tag,
    DocumentType,
    Correspondent,
}

impl PaperlessDefaultFieldType {
    fn to_string(self) -> &'static str {
        match self {
            PaperlessDefaultFieldType::Tag => "tags",
            PaperlessDefaultFieldType::DocumentType => "document_types",
            PaperlessDefaultFieldType::Correspondent => "correspondents",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultField {
    #[serde(skip_serializing_if = "Option::is_none")] // Skip `id` if it's None
    id: Option<u32>,
    slug: String,
    name: String,
    matching_algorithm: u8,
}

pub async fn get_data_from_paperless(
    client: &Client,
    url: &str,
    filter: &str,
) -> Result<Response<Document>, Box<dyn StdError + Send + Sync>> {
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
            parse_document_response(json)
        }
        Err(e) => {
            slog_scope::error!("Error while fetching documents from paperless: {}",e);
            Err(e.into())
        }
    }
}

pub async fn get_next_data_from_paperless(client: &Client,
                                          url: &str,
) -> Result<Response<Document>, Box<dyn StdError + Send + Sync>> {
    // Read token from environment
    //Define filter string
    slog_scope::info!("Retrieve next page {}", url);
    let response = client.get(format!("{}", url)).send().await?;


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
            parse_document_response(json)
        }
        Err(e) => {
            slog_scope::error!("Error while fetching documents from paperless: {}",e);
            Err(e.into())
        }
    }
}


pub fn parse_document_response(json: &str) -> Result<Response<Document>, Box<dyn StdError + Send + Sync>> {
    let data: Result<Response<Document>, _> = serde_json::from_str(json);
    match data {
        Ok(data) => {
            slog_scope::info!("Successfully retrieved {} Documents", data.results.len());
            Ok(data)
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

pub async fn get_default_fields(
    client: &Client,
    base_url: &str,
    endpoint: PaperlessDefaultFieldType,
) -> Result<Vec<DefaultField>, Box<dyn std::error::Error>>
{
    slog_scope::info!("Fetching custom fields from paperless at {}", base_url);
    let res = client
        .get(format!("{}/api/{}/", base_url, endpoint.to_string()))
        .send()
        .await?;

    let response_result = res.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::debug!("Response from server while fetching documents: {}", body);

            // Remove the "Field: " prefix if necessary
            let json = body.trim_start_matches("Field: ");
            let data: Result<Response<DefaultField>, _> = serde_json::from_str(json);
            match data {
                Ok(data) => {
                    slog_scope::info!("{}: {:?}", endpoint.to_string(), data.results);
                    Ok(data.results)
                }
                Err(e) => {
                    let column = e.column();
                    let start = (column as isize - 30).max(0) as usize;
                    let end = (column + 30).min(json.len());
                    slog_scope::error!("Error occurred parsing custom fields: {}", e);
                    slog_scope::error!("Error at column {}: {}", column, &json[start..end]);
                    slog_scope::debug!("Error occurred in json {}", &json);
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
    mode: Mode,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut custom_fields = Vec::new();

    // Use `if let` to conditionally execute code if the 'tagged' field is found.
    let field = match fields.iter().find(|&f| f.name == "tagged") {
        Some(field) => field,
        None => {
            slog_scope::error!("{} field not found in the provided fields.", "'tagged'");
            return Err(Box::new(fmt::Error::default())); // Use a standard library error type like fmt::Error.
        }
    };

    let tagged_field = CustomField {
        field: field.id,
        value: Some(serde_json::json!(true)),
    };

    // Add this tagged_field to your custom_fields collection or use it as needed.
    custom_fields.push(tagged_field);

    for (key, value) in metadata {
        if key == "title" {
            continue;
        }

        if let Some(field) = fields.iter().find(|&f| f.name == *key) {
            let custom_field = convert_field_to_custom_field(value, field);
            custom_fields.push(custom_field);
        } else {
            if matches!(mode, Mode::Create) {
                slog_scope::info!("Creating field: {}", key);
                let create_field = CreateField {
                    name: key.clone(),
                    data_type: "Text".to_string(),
                    default_value: None,
                };
                match create_custom_field(client, &create_field, base_url).await
                {
                    Ok(new_field) => {
                        let custom_field = convert_field_to_custom_field(value, &new_field);
                        custom_fields.push(custom_field)
                    }
                    Err(e) => {
                        slog_scope::error!("Error: {} creating custom field: {}, skipping...",e, key)
                    }
                }
            }
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
    slog_scope::debug!("Request Payload: {}", map_to_string(&payload));

    for (key, value) in &payload {
        slog_scope::debug!("{}: {}", key, value);
    }
    let res = client.patch(&url).json(&payload).send().await?;
    let response_result = res.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::trace!("{}", body);
            slog_scope::info!("Document with ID: {} successfully updated", document_id);
            Ok(())
        }
        Err(e) => {
            slog_scope::error!("Error while updating document fields: {}", e);
            Err(e.into())
        }
    }
}

/// This function update the default fields like tags, correspondents and document_types in paperless
/// it is checked if a field exists on the server and if not, it is created
/// 
pub async fn update_document_default_fields(
    client: &Client,
    document_id: u32,
    fields: &Vec<DefaultField>,
    data: Vec<String>,
    base_url: &str,
    endpoint: PaperlessDefaultFieldType,
    mode: Mode,
) -> Option<Box<dyn std::error::Error>> {
    let mut default_field_ids = Vec::new();

    for value in data {

        if let Some(field) = fields.iter().find(|&f| normalize_string(&*f.name) == normalize_string(&*value)) {
            let default_field_id = field.id;
            default_field_ids.push(default_field_id);
        } else {
            if matches!(mode, Mode::Create) {
                slog_scope::info!("Creating {}: {}", endpoint.to_string(), value);
                let create_field = DefaultField {
                    id: None,
                    name: value.clone(),
                    slug: value.clone(),
                    matching_algorithm: 6,
                };
                match create_default_field(client, &create_field, base_url, endpoint).await
                {
                    Ok(new_field) => {
                        default_field_ids.push(new_field.id)
                    }
                    Err(e) => {
                        slog_scope::error!("Error: {} creating custom field: {}, skipping...",e, value)
                    }
                }
            }
        }
    }

    let mut payload = serde_json::Map::new();
    payload.insert(endpoint.to_string().to_string(), serde_json::json!(default_field_ids));

    if payload.is_empty() {
        slog_scope::warn!("{}", "payload is empty, not updating fields");
        return None
    }
    let url = format!("{}/api/documents/{}/", base_url, document_id);
    slog_scope::info!("Updating document with ID: {}", document_id);
    slog_scope::debug!("Request Payload: {}", map_to_string(&payload));

    for (key, value) in &payload {
        slog_scope::debug!("{}: {}", key, value);
    }
    let res = client.patch(&url).json(&payload).send().await;
    let response_result = res;
    match response_result {
        Ok(data) => {
            let body = data.text().await;
            slog_scope::info!("Document with ID: {} successfully updated", document_id);
            None
        }
        Err(e) => {
            slog_scope::error!("Error while updating document fields: {}", e);
            Some(Box::new(e))
        }
    }
}

fn convert_field_to_custom_field(value: &Option<Value>, field: &Field) -> CustomField {
    let custom_field = CustomField {
        field: field.id.clone(),
        value: value.as_ref().cloned(),
    };
    custom_field
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateField {
    name: String,
    default_value: Option<String>,
    data_type: String,
}

pub async fn create_custom_field(
    client: &Client,
    field: &CreateField,
    base_url: &str,
) -> Result<Field, Box<dyn std::error::Error>> {
    // Define the URL for creating a custom field
    let url = format!("{}/api/custom_fields/", base_url);


    // Send the request to create the custom field
    let res = client.post(&url).json(&field).send().await?;
    let response_result = res.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::trace!("{}", body);
            let field: Result<Response<Field>, _> = serde_json::from_str(&body);
            match field {
                Ok(field) => {
                    Ok(field.results[0].clone()) // TODO: improve
                }
                Err(e) => {
                    slog_scope::debug!("Creating field response: {}", body);
                    slog_scope::error!("Error parsing response from new field: {}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            slog_scope::error!("Error creating custom field: {}", e);
            Err(e.into())
        }
    }
}
pub async fn create_default_field(
    client: &Client,
    field: &DefaultField,
    base_url: &str,
    endpoint: PaperlessDefaultFieldType,
) -> Result<DefaultField, Box<dyn std::error::Error>> {
    // Define the URL for creating a custom field
    let url = format!("{}/api/{}/", base_url, endpoint.to_string());


    // Send the request to create the custom field
    let res = client.post(&url).json(&field).send().await?;
    let response_result = res.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::trace!("{}", body);
            let field: Result<DefaultField, _> = serde_json::from_str(&body);
            match field {
                Ok(field) => {
                    Ok(field) // TODO: improve
                }
                Err(e) => {
                    slog_scope::debug!("Creating field response: {}", body);
                    slog_scope::error!("Error parsing response from new field: {}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            slog_scope::error!("Error creating custom field: {}", e);
            Err(e.into())
        }
    }
}
fn map_to_string(map: &Map<String, Value>) -> String {
    map.iter()
        .map(|(key, value)| format!("{}: {}", key, value))
        .collect::<Vec<String>>()
        .join(", ")
}

