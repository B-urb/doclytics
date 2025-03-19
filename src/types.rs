use std::sync::Arc;
use ollama_rs::Ollama;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::util::create_mode_from_env;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    pub id: u32,
    pub correspondent: Option<u32>,
    pub document_type: Option<u32>,
    pub storage_path: Option<u32>,
    pub title: String,
    pub content: String,
    pub created: String,
    pub created_date: Option<String>,
    pub modified: String,
    pub added: String,
    pub archive_serial_number: Option<u32>,
    pub original_file_name: Option<String>,
    pub archived_file_name: Option<String>,
    pub owner: Option<u32>,
    pub notes: Vec<String>,
    pub tags: Vec<u32>,
    pub user_can_change: bool,
    pub custom_fields: Vec<CustomField>, // Change this to match the structure of the custom_fields array
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T> {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub all: Vec<u32>,
    pub results: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomField {
    pub value: Option<Value>,
    pub field: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Field {
    pub id: u32,
    pub name: String,
    pub data_type: String,
}

#[derive(Clone, Copy)]
pub enum Mode {
    NoAnalyze,
    Create,
    NoCreate,
}
impl Mode {
    pub fn from_int(value: i32) -> Self {
        match value {
            2 => Mode::Create,
            1 => Mode::NoCreate,
            0 => Mode::NoAnalyze,
            _ => Mode::NoCreate,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PaperlessDefaultFieldType {
    Tag,
    DocumentType,
    Correspondent,
}

impl PaperlessDefaultFieldType {
    fn to_string(self) -> &'static str {
        match self {
            PaperlessDefaultFieldType::Tag => "tags",
            PaperlessDefaultFieldType::DocumentType => "document_types",
            PaperlessDefaultFieldType::Correspondent => "correspondents",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultField {
    #[serde(skip_serializing_if = "Option::is_none")] // Skip `id` if it's None
    pub id: Option<u32>,
    pub slug: String,
    pub name: String,
    pub matching_algorithm: u8,
}

pub struct PaperlessClient {
    pub client: Client,
    pub mode: Mode,
    pub tag_mode: Mode,
    pub doctype_mode: Mode,
    pub correspondent_mode: Mode,
    pub default_fields: Option<Vec<DefaultField>>,
    pub fields: Option<Vec<DefaultField>>,
    pub base_url: String,
}




pub struct OllamaClient {
    pub ollama: Ollama,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig<'a> {
    pub paperless_client: &'a PaperlessClient,
    pub ollama: &'a Ollama
    pub model:
}
