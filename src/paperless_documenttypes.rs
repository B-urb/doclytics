use reqwest::Client;



struct DocumentType {
    id: u32,
    slug: String,
    name: String,
    matching_algorithm: u8
}
pub fn create_document_type(
    document_types: &str,
    client: &Client,
    base_url: &str,
) {
    
    

}


pub fn get_document_types(
    client: &Client,
    base_url: &str,
) {

    let url = format!("{}/api/document_types/", base_url);
    let res= client.get(url).send();
    let body = match { 
        Ok(data) => {
            
        },
        Err(e) => {
            slog_scope::error!("Error getting document types: {}", {})
        }
    };
}



pub fn determine_if_type_exists(
    client: &Client,
    base_url: &str,
) {
    
}