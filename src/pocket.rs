// https://localhost/mediawiki/index.php/Pocket_REST_API

use std::collections::HashMap;

use reqwest::Client;
use serde_derive::Deserialize;
//use serde_derive::Serialize;

//#[derive(Serialize, Debug)]
//struct SearchRequest {}

#[derive(Deserialize, Debug)]
struct RetrieveResponse {
    status: u8,
    complete: u8,
    list: HashMap<String, SavedItem>,
}

#[derive(Deserialize, Debug)]
pub struct SavedItem {
    item_id: String,
    resolved_id: String,
    given_url: String,
    given_title: String,
    time_added: String,
    time_updated: String,
    resolved_url: String,
    resolved_title: String,
    excerpt: String,
}

#[test]
fn basic_query() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut res = client.post("https://getpocket.com/v3/get")
        .header(http::header::CONTENT_TYPE, "application/json")
// TODO : include consumer_key and access_token:
        .body(r#"{
                "consumer_key" : "",
                "access_token" : "",
                "count": 2,
                "detailType":"complete"
                }"#)
        .send()?;

    println!("result {:?}", res);

    println!("result {:?}", res.json::<RetrieveResponse>());

    Ok(())
}
