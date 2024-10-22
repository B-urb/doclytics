use ollama_rs::Ollama;
use reqwest::Client;
use serde::de::StdError;
use crate::{extract_json_object, Document, Field, Mode};
use crate::llm_api::generate_response;
use crate::paperless::{get_default_fields, update_document_default_fields, update_document_fields, DefaultField, PaperlessDefaultFieldType};

 const ANSWER_INSTRUCTION: &'static str = "The result should be a only a json array of string and nothing else. The answer should start and end with the square bracket. The document is: ";
async fn construct_document_type_prompt(client: &Client, base_url: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let document_types = get_default_fields(client, base_url, PaperlessDefaultFieldType::DocumentType).await;
    let base_prompt = format!("Determine the type of this document from the following available document types: {:?}, if none of these fit the document, create a new one. ", document_types);
    Ok(base_prompt)
}


async fn construct_tag_prompt(client: &Client, base_url: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let document_types = get_default_fields(client, base_url, PaperlessDefaultFieldType::Tag).await;
    let base_prompt = format!("Determine the type of this document from the following available document types: {:?}, if none of these fit the document, create a new one. ", document_types);
    Ok(base_prompt)
}
async fn construct_correspondent_prompt(client: &Client, base_url: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let document_types = get_default_fields(client, base_url, PaperlessDefaultFieldType::Correspondent).await;
    let base_prompt = format!("Determine possible correspondents from this document from the following available correspondents: {:?}, if none of these fit the document, create a new one. The result should be a only a json array of string and nothing else. The answer should start and end with the square bracket. ", document_types);
    Ok(base_prompt)
}


pub async fn extract_default_fields(ollama: &Ollama, model: &str, prompt_base: &String, client: &Client, fields: &Vec<DefaultField>, base_url: &str, document: &Document, mode: Mode, field_type: PaperlessDefaultFieldType) -> Option<Box<dyn StdError>> {
    let prompt = match field_type {
        PaperlessDefaultFieldType::Correspondent => construct_correspondent_prompt(client, base_url).await,
        PaperlessDefaultFieldType::Tag => construct_tag_prompt(client, base_url).await,
        PaperlessDefaultFieldType::DocumentType => construct_document_type_prompt(client, base_url).await,
    };
    match prompt {
        Ok(prompt) => {
            let prompt_with_document = prompt + &*ANSWER_INSTRUCTION + &*document.content;
            match generate_response(ollama, &model.to_string(), prompt_with_document).await {
                Ok(res) => {
                    // Log the response from the generate_response call
                    slog_scope::debug!("LLM Response: {}", res.response);

                    match extract_json_object(&res.response) {
                        Ok(json_str) => {
                            // Log successful JSON extraction
                            slog_scope::debug!("Extracted JSON Object: {}", json_str);

                            match serde_json::from_str(&json_str) {
                                Ok(json) => update_document_default_fields(client, document.id, &fields, json, base_url, field_type, mode).await,
                                Err(e) => {
                                    slog_scope::error!("Error parsing llm response json {}", e.to_string());
                                    slog_scope::debug!("JSON String was: {}", & json_str);
                                    Some(Box::new(e))
                                }
                            }
                        }
                        Err(e) => {
                            slog_scope::error ! ("{}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    slog_scope::error ! ("Error generating llm response: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            Some(e)
        }
    }
}
pub fn determine_if_type_exists(
    client: &Client,
    base_url: &str,
) {
    //TODO:  
}