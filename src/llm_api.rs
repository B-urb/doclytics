use ollama_rs::generation::completion::GenerationResponse;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use crate::Document;

pub async fn generate_response(
    ollama: &Ollama,
    model: &String,
    prompt_base: &String,
    document: &Document,
) -> std::result::Result<GenerationResponse, Box<dyn std::error::Error>> {
    let prompt = format!("{} {}", document.content, prompt_base);
    let res = ollama
        .generate(GenerationRequest::new(model.clone(), prompt))
        .await;
    match res { 
        Ok(res) => {
           slog_scope::debug!("Response from ollama {}", res.response);
            Ok(res)
        },
        Err(e) => {
            slog_scope::error!("{}", e);
            Err(e.into())
        }
    }
}