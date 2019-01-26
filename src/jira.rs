//! Jira client to retrieve items to display in the dashboard.
//!
//! Authentication is performed using HTTP Basic auth given the
//! values from the `JIRA_USERNAME` and `JIRA_PASSWORD` env vars.

// c.f. https://serde.rs/derive.html

// TODO : better debugging of errors like this:
// Error: Error { kind: Json(Error("premature end of input", line: 1, column: 8617)), url: None }
// Had a DateTime type instead of Date for duedate

use std::env;
use std::fmt::Display;
use std::str::FromStr;

use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDate;
use chrono::Utc;
use http::StatusCode;
use reqwest::Client;
use reqwest::RequestBuilder;
use reqwest::Response;
use serde::Deserializer;
use serde_json::from_str;

type Url = String;

static URL_BASE: &str = "https://localhost/jira/rest/api/2/";

static FILTER_ID_OVERDUE_ISSUES: &str = "10300";

static FILTER_ID_DUE_IN_NEXT_2_WEEKS: &str = "10107";

static DEFAULT_MAX_RESULTS: u32 = 100;

#[derive(Serialize, Debug)]
struct SearchRequest {
    jql: String,
    #[serde(rename = "startAt")]
    start_at: u32,
    #[serde(rename = "maxResults")]
    max_results: u32,
    fields: Vec<String>,
}

impl SearchRequest {
    fn for_jql(jql: String) -> SearchRequest {
        SearchRequest {
            jql,
            start_at: 0,
            max_results: DEFAULT_MAX_RESULTS,
            fields: vec![String::from("status"),
                         String::from("summary"),
                         String::from("*all")],
        }
    }
}

#[derive(Deserialize, Debug)]
struct IssueResponse {
    issues: Vec<Issue>,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
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
pub struct IssueFields {
    summary: String,
    // TODO : don't need a nested object here, just the status name
    status: Status,
    created: DateTime<Local>,
    duedate: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Filter {
    id: String,
    #[serde(rename = "self")]
    url: Url,
    name: String,
    jql: String,
}

trait Request {
    fn jira_auth(self) -> Self;
}

impl Request for RequestBuilder {
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

trait CheckedResponse {
    fn check_ok(self) -> Self;
}

impl CheckedResponse for Response {
    fn check_ok(mut self) -> Self {
        // TODO : could also use Response::error_for_status()
        if self.status() != StatusCode::OK {
            panic!("Request failed {}", self.text().unwrap());
        }
        if false {
            // TODO : this still consumes the response and is not an effective logging facility
            let mut body = Vec::new();
            self.copy_to(&mut body).unwrap();
            println!("Response:\n{}", String::from_utf8(body).unwrap());
        }
        self
    }
}

fn get_filter(id: &str) -> Result<Filter, reqwest::Error> {
    let url = format!("{}/filter/{}", URL_BASE, id);
    Client::new().get(&url)
        .jira_auth()
        .send()?
        .check_ok()
        .json()
}

fn get_issues(req: SearchRequest) -> Result<Vec<Issue>, reqwest::Error> {
    let url = format!("{}/search", URL_BASE);
    Ok(Client::new().post(&url)
        .jira_auth()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&req).unwrap())
        .send()?
        .check_ok()
        .json::<IssueResponse>()?
        .issues)
}

pub fn get_overdue_issues() -> Result<Vec<Issue>, reqwest::Error> {
    get_issues(SearchRequest::for_jql(get_filter(FILTER_ID_OVERDUE_ISSUES)?.jql))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_jira_query() -> Result<(), reqwest::Error> {
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
             "fields": [ "summary", "status", "assignee", "created" ] }"#)
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

    #[test]
    fn test_get_filter() -> Result<(), reqwest::Error> {
        let filter = get_filter(FILTER_ID_OVERDUE_ISSUES)?;
        assert_eq!(FILTER_ID_OVERDUE_ISSUES, filter.id);
        println!("filter: {:?}", filter);
        Ok(())
    }

    #[test]
    fn test_get_overdue() -> Result<(), reqwest::Error> {
        let issues = get_overdue_issues()?;
        println!("issues: {:?}", issues);
        Ok(())
    }
}
