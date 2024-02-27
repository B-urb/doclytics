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
    res.map_err(|e| e.into()) // Map the Err variant to a Box<dyn std::error::Error>
}