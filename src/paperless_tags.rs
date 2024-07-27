use reqwest::Client;


struct Tags {
    id: u32,
    slug: String,
    name: String,
    matching_algorithm: u8
}
pub fn create_tag(
    correspondent_name: &str,
    client: &Client,
    base_url: &str,
) {

}


pub fn get_tags(
    client: &Client,
    base_url: &str,
) {

    let url = format!("{}/api/tags/", base_url);
    let res= client.get(url).send();
}
