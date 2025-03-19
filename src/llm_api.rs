use crate::types::OllamaClient;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::completion::GenerationResponse;
use ollama_rs::Ollama;

impl OllamaClient {
    fn init_ollama_client(host: &str, port: u16, secure_endpoint: bool, model: &str) -> Self {
        let protocol = if secure_endpoint { "https" } else { "http" };
        let ollama_base_url = format!("{}://{}", protocol, host);
        let ollama = Ollama::new(ollama_base_url, port);
        OllamaClient {
            ollama,
            model: model.to_string(),
        }
    }
    pub async fn generate_response(
        &self,
        model: &String,
        prompt: String,
    ) -> std::result::Result<GenerationResponse, Box<dyn std::error::Error>> {
        let res = self
            .ollama
            .generate(GenerationRequest::new(model.clone(), prompt))
            .await;
        match res {
            Ok(res) => {
                slog_scope::debug!("Response from ollama:\n {}", res.response);
                Ok(res)
            }
            Err(e) => {
                slog_scope::error!("{}", e);
                Err(e.into())
            }
        }
    }
}
