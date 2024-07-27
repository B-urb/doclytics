use reqwest::Response;
use serde::Deserialize;
use crate::error::ResponseError;
use crate::{Response as PaperlessResponse};


pub async fn parse_response<T>(response: Response) -> Result<T, ResponseError > where T: Deserialize<'static> {

    let response_result = response.error_for_status();
    match response_result {
        Ok(data) => {
            let body = data.text().await?;
            slog_scope::trace!("Response from server while fetching documents: {}", body);

            let json = body.trim_start_matches("Document content: ");

            let data: Result<PaperlessResponse<T>, _> = serde_json::from_str(json);
            match data {
                Ok(data) => {
                    slog_scope::info!("Successfully retrieved {} Documents", data.results.len());
                    Ok(data.results)
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
        Err(e) => {
            slog_scope::error!("Error while fetching documents from paperless: {}",e);
            Err(e.into())
        }
    }
}