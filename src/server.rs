use axum::{
    routing::{post},
    http::StatusCode,
    Json, Router,
};
use axum::extract::State;
use serde::Deserialize;
use crate::doclytics::process_document_by_id;
use crate::paperless::get_document_by_id;
use crate::types::ServerConfig;

#[derive(Deserialize)]
struct UpdateDocumentRequest {
    id: String,
}

pub async fn init_server(server_config: ServerConfig<'_>) -> Router {
    tracing_subscriber::fmt::init();

    Router::new()
        .route("/updateDocument", post(update_document))
        .with_state(server_config)
}

async fn update_document(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(server_config): State<ServerConfig<'_>>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> StatusCode
{
    match get_document_by_id(server_config.client, server_config.url, payload.id.to_string().as_str()).await {
        Ok(document) => {
            process_document_by_id(document, payload.id).await;
            StatusCode::OK
        }
        Err(e) => {
            slog_scope::error!("{}", e.to_string());
            return StatusCode::INTERNAL_SERVER_ERROR
        }
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    StatusCode::OK
}
