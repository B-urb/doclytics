use reqwest::Client;
use crate::paperless::CreateField;
struct DocumentType {
    id: u32,
    slug: String,
    name: String,
    matching_algorithm: u8
}
pub fn create_correspondent(
    correspondent_name: &str,
    client: &Client,
    base_url: &str,
) {
        
}


pub fn get_correspondents(
    client: &Client,
    base_url: &str,
) {

    let url = format!("{}/api/correspondents/", base_url);
    let res= client.get(url).send();
}
