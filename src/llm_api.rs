use ollama_rs::generation::completion::GenerationResponse;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;

pub async fn generate_response(
    ollama: &Ollama,
    model: &String,
    prompt: String,
) -> std::result::Result<GenerationResponse, Box<dyn std::error::Error>> {
    let res = ollama
        .generate(GenerationRequest::new(model.clone(), prompt))
        .await;
    match res { 
        Ok(res) => {
           slog_scope::debug!("Response from ollama:\n {}", res.response);
            Ok(res)
        },
        Err(e) => {
            slog_scope::error!("{}", e);
            Err(e.into())
        }
    }
}