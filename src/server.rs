
use axum::{
    routing::{post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize};
use tracing_subscriber::fmt::format;

#[derive(Deserialize)]
struct UpdateDocumentRequest {
    id: String,
}

pub async fn init_server() -> Router {
    tracing_subscriber::fmt::init();

    return Router::new()
        .route("/updateDocument", post(update_document));

}

async fn update_document(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<UpdateDocumentRequest>,
) -> StatusCode
{

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    StatusCode::OK
}
