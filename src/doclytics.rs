use crate::llm_api::generate_response;
use crate::paperless::{
    get_data_from_paperless, get_default_fields, get_next_data_from_paperless, query_custom_fields,
    update_document_fields,
};
use crate::paperless_defaultfields::extract_default_fields;
use crate::types::{Document, Field, Mode, PaperlessDefaultFieldType};
use crate::util::create_mode_from_env;
use crate::util::extract_json_object;
use ollama_rs::Ollama;
use reqwest::Client;
use std::env;

pub async fn process_documents_batch(
    documents: &Vec<Document>,
    ollama: &Ollama,
    model: &str,
    prompt_base: &String,
    client: &Client,
    fields: &Vec<Field>,
    base_url: &str,
    mode: Mode,
) -> Result<(), Box<dyn std::error::Error>> {
    let tag_mode = create_mode_from_env("DOCLYTICS_TAGS");
    let doctype_mode = create_mode_from_env("DOCLYTICS_DOCTYPE");
    let correspondent_mode = create_mode_from_env("DOCLYTICS_CORRESPONDENT");

    Ok(for document in documents {
        slog_scope::trace!("Document Content: {}", document.content);
        slog_scope::info!("Generate Response with LLM {}", "model");
        slog_scope::debug!("with Prompt: {}", prompt_base);

        generate_response_and_extract_data(
            ollama,
            &model,
            &prompt_base,
            client,
            &fields,
            base_url,
            mode,
            &document,
        )
        .await;
        let default_fields =
            get_default_fields(client, base_url, PaperlessDefaultFieldType::Tag).await;
        match default_fields {
            Ok(default_fields) => {
                match tag_mode {
                    Mode::NoAnalyze => (),
                    _ => {
                        if let Some(err) = extract_default_fields(
                            ollama,
                            &model,
                            &prompt_base,
                            client,
                            &default_fields,
                            base_url,
                            &document,
                            tag_mode,
                            PaperlessDefaultFieldType::Tag,
                        )
                        .await
                        {
                            slog_scope::error!("Error while getting tags: {:?}", err);
                        }
                    }
                }
                match doctype_mode {
                    Mode::NoAnalyze => (),
                    _ => {
                        if let Some(err) = extract_default_fields(
                            ollama,
                            &model,
                            &prompt_base,
                            client,
                            &default_fields,
                            base_url,
                            &document,
                            doctype_mode,
                            PaperlessDefaultFieldType::DocumentType,
                        )
                        .await
                        {
                            slog_scope::error!("Error while getting doctype: {:?}", err);
                        }
                    }
                }
                match correspondent_mode {
                    Mode::NoAnalyze => (),
                    _ => {
                        if let Some(err) = extract_default_fields(
                            ollama,
                            &model,
                            &prompt_base,
                            client,
                            &default_fields,
                            base_url,
                            &document,
                            correspondent_mode,
                            PaperlessDefaultFieldType::Correspondent,
                        )
                        .await
                        {
                            slog_scope::error!("Error while getting correspondents: {:?}", err);
                        }
                    }
                }
            }
            Err(e) => slog_scope::error!("Error while interacting with paperless: {}", e),
        }
    })
}

pub async fn process_documents(
    client: &Client,
    ollama: &Ollama,
    model: &str,
    base_url: &str,
    filter: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let language = env::var("LANGUAGE")
        .unwrap_or_else(|_| "EN".to_string())
        .to_uppercase();
    let base_prompt;

    match language.as_ref() {
        "DE" => base_prompt = "Bitte ziehe die Metadaten aus dem bereitgestelltem Dokument \
        und antworte im JSON format. \
        Die Felder, welche ich brauche sind:\
         title,topic,sender,recipient,urgency(mit werten entweder n/a oder low oder medium oder high),\
         date_received(im maschinenlesbarem format),category.\
         Analysiere das Dokument, um die Werte für diese Felder zu finden und forme die Antwort als JSON-Objekt. \
         Verwende die wahrscheinlichste Antwort für jedes Feld in der gleichen Sprache wie das Dokument. \
         Die Antwort sollte nur JSON-Daten enthalten, bei denen die Schlüssel und Werte alle in einfacher Textform \
         (keine verschachtelten Objekte) vorliegen, um von einem anderen Programm direkt analysiert werden zu können. \
         Also keine zusätzlichen Texte oder Erklärungen, der Antworttext sollte mit eckigen Klammern beginnen und enden, \
         die das JSON-Objekt umfassen ".to_string(),
        _ => base_prompt = "Please extract metadata\
        from the provided document and return it in JSON format.\
        The fields I need are:\
         title,topic,sender,recipient,urgency(with value either n/a or low or medium or high),\
         date_received(in machine-readable format),category.\
          Analyze the document to find the values for these fields and format the response as a \
          JSON object. Use the most likely answer for each field. \
          The response should contain only JSON data where the key and values are all in simple string \
          format(no nested object) for direct parsing by another program. So no additional text or \
          explanation, no introtext, the answer should start and end with curly brackets \
          delimiting the json object ".to_string()
    };

    let prompt_base = env::var("BASE_PROMPT").unwrap_or_else(|_| base_prompt.to_string());

    let mode_env = env::var("MODE").unwrap_or_else(|_| "0".to_string());
    let mode_int = mode_env.parse::<i32>().unwrap_or(0);
    let mode = Mode::from_int(mode_int);
    let fields = query_custom_fields(client, base_url).await?;
    match get_data_from_paperless(&client, &base_url, filter).await {
        Ok(mut data) => loop {
            process_documents_batch(
                &data.results,
                ollama,
                model,
                &prompt_base,
                client,
                &fields,
                base_url,
                mode,
            )
            .await?;

            if let Some(url) = data.next {
                match get_next_data_from_paperless(&client, url.as_str()).await {
                    Ok(next_data) => {
                        data = next_data;
                    }
                    Err(e) => {
                        slog_scope::error!("Error while interacting with paperless: {}", e);
                        break;
                    }
                }
            } else {
                break;
            }
        },
        Err(e) => slog_scope::error!("Error while interacting with paperless: {}", e),
    }
    Ok(())
}

async fn generate_response_and_extract_data(
    ollama: &Ollama,
    model: &str,
    prompt_base: &String,
    client: &Client,
    fields: &Vec<Field>,
    base_url: &str,
    mode: Mode,
    document: &Document,
) {
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
                        Ok(json) => update_document_fields(
                            client,
                            document.id,
                            &fields,
                            &json,
                            base_url,
                            mode,
                        )
                        .await
                        .unwrap_or_default(), //TODO: Fix unwrap
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
