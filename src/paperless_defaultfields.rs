use ollama_rs::Ollama;
use reqwest::Client;
use serde::de::StdError;
use crate::{extract_json_object, Document, Field, Mode};
use crate::llm_api::generate_response;
use crate::paperless::{get_default_fields, update_document_fields, PaperlessDefaultFieldType};

const ANSWER_INSTRUCTION: String = "The result should be a only a json array of string and nothing else. The answer should start and end with the square bracket. The document is:".to_string();
async fn construct_document_type_prompt(client: &Client, base_url: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let document_types = get_default_fields(client, base_url, PaperlessDefaultFieldType::DocumentType).await?;
    let base_prompt = format!("Determine the type of this document from the following available document types: {:?}, if none of these fit the document, create a new one: ", document_types);
    Ok(base_prompt)
}


async fn construct_tag_prompt(client: &Client, base_url: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let document_types = get_default_fields(client, base_url, PaperlessDefaultFieldType::Tag).await?;
    let base_prompt = format!("Determine the type of this document from the following available document types: {:?}, if none of these fit the document, create a new one: ", document_types);
    Ok(base_prompt)
}
async fn construct_correspondent_prompt(client: &Client, base_url: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let document_types = get_default_fields(client, base_url, PaperlessDefaultFieldType::Correspondent).await?;
    let base_prompt = format!("Determine possible correspondents from this document from the following available correspondents: {:?}, if none of these fit the document, create a new one. The result should be a only a json array of string and nothing else. The answer should start and end with the square bracket. The document is: ", document_types);
    Ok(base_prompt)
}






async fn generate_response_and_extract_data(ollama: &Ollama, model: &str, prompt_base: &String, client: &Client, fields: &Vec<Field>, base_url: &str, mode: Mode, document: &Document) {
    let prompt = format!("{} {}", prompt_base, document.content);

    match generate_response(ollama, &model.to_string(), prompt).await {
        Ok(res) => {
            // Log the response from the generate_response call
            slog_scope::debug!("LLM Response: {}", res.response);

            match extract_json_object(&res.response) {
                Ok(json_str) => {
                    // Log successful JSON extraction
                    slog_scope::debug!("Extracted JSON Object: {}", json_str);

                    match serde_json::from_str(&json_str) {
                        Ok(json) => update_document_fields(client, document.id, &fields, &json, base_url, mode).await?,
                        Err(e) => {
                            slog_scope::error!("Error parsing llm response json {}", e.to_string());
                            slog_scope::debug!("JSON String was: {}", &json_str);
                        }
                    }
                }
                Err(e) => slog_scope::error!("{}", e),
            }
        }
        Err(e) => {
            slog_scope::error!("Error generating llm response: {}", e);
        }
    }
}
pub fn determine_if_type_exists(
    client: &Client,
    base_url: &str,
) {
    //TODO:  
}