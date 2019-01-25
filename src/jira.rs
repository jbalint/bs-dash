//! Jira client to retrieve items to display in the dashboard.
//!
//! Authentication is performed using HTTP Basic auth given the
//! values from the `JIRA_USERNAME` and `JIRA_PASSWORD` env vars.

// c.f. https://serde.rs/derive.html

use std::env;
use std::fmt::Display;
use std::str::FromStr;

use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserializer;
use serde_json::from_str;
use reqwest::RequestBuilder;

type Url = String;

#[derive(Serialize, Debug)]
struct SearchRequest {
    jql: String,
    start_at: u32,
    max_results: u32,
    fields: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct IssueResponse {
    issues: Vec<Issue>,
}

#[derive(Deserialize, Debug)]
struct Issue {
    // TODO : these are integers? How to get serde to parse them easily
    // #[serde(deserialize_with = "from_str")]
    // https://github.com/serde-rs/json/issues/317
    id: String,
    #[serde(rename = "self")]
    url: Url,
    key: String,
    // TODO : can I flatten this without a custom serde method? (just embed the IssueFields members here)
    // like if I could have "summary" and say it comes from "fields.summary"
    fields: IssueFields,
}

#[derive(Deserialize, Debug)]
struct IssueFields {
    summary: String,
    // TODO : don't need a nested object here, just the status name
    status: Status,
}

#[derive(Deserialize, Debug)]
struct Status {
    id: String,
    name: String,
}

trait JiraRequest {
    fn jira_auth(self) -> Self;
}

impl JiraRequest for RequestBuilder {
    fn jira_auth(self) -> Self {
        let (username, password) =
            match (env::var("JIRA_USERNAME"),
                   env::var("JIRA_PASSWORD")) {
                (Ok(username), Ok(password)) => (username, password),
                _ => panic!("JIRA_USERNAME and/or JIRA_PASSWORD not set"),
            };

        self.basic_auth(username, Some(password))
    }
}

fn auth(req: &mut Client) {}

pub fn do_search() {
    let req = SearchRequest { jql: String::from("jql=1"), start_at: 0, max_results: 0, fields: Vec::new() };
    let string = serde_json::to_string(&req).unwrap();
    println!("{}", string);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search() -> Result<(), reqwest::Error> {
        do_search();

        // https://docs.atlassian.com/software/jira/docs/api/REST/7.12.0/#api/2/search
        let client = Client::new();
        let mut res = client.post("https://localhost/jira/rest/api/2/search")
            .jira_auth()
            .header(http::header::CONTENT_TYPE, "application/json")
            // TODO : could also use reqwest's json() method here with a HashMap
            .body(r#"{
             "jql": "project = BS",
             "startAt": 0,
             "maxResults": 2,
             "fields": [ "summary", "status", "assignee" ] }"#)
            .send()?;

        println!("result {:?}", res);

        assert_eq!(StatusCode::OK, res.status());

        if true {
            let jira_resp: IssueResponse = res.json()?;
            //println!("text {:?}", res.text());
            assert_eq!(2, jira_resp.issues.len());
            println!("deserialized {:?}", jira_resp);
        } else {
            // using generic Value
            let json: serde_json::Value = res.json()?;
            println!("deserialized {:?}", json);
        }

        Ok(())
    }
}
