//! Jira REST client
//!
//! Authentication is performed using HTTP Basic auth given the
//! values from the `JIRA_USERNAME` and `JIRA_PASSWORD` env vars.

// c.f. https://serde.rs/derive.html

// TODO : better debugging of errors like this:
// Error: Error { kind: Json(Error("premature end of input", line: 1, column: 8617)), url: None }
// Had a DateTime type instead of Date for duedate

use std::env;

use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDate;
use http::StatusCode;
use reqwest::Client;
use reqwest::RequestBuilder;
use reqwest::Response;

use crate::http_util::AuthRequest;
use crate::http_util::CheckedResponse;

type Url = String;

static URL_BASE: &str = "https://localhost/jira/rest/api/2/";

static FILTER_ID_OVERDUE_ISSUES: &str = "10300";

static FILTER_ID_DUE_IN_NEXT_2_WEEKS: &str = "10107";

static DEFAULT_MAX_RESULTS: u32 = 100;

static ENV_VAR_PREFIX: &str = "JIRA";

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
    pub id: String,
    #[serde(rename = "self")]
    pub url: Url,
    pub key: String,
    // TODO : can I flatten this without a custom serde method? (just embed the IssueFields members here)
    // like if I could have "summary" and say it comes from "fields.summary"
    pub fields: IssueFields,
}

#[derive(Deserialize, Debug)]
pub struct IssueFields {
    pub summary: String,
    // TODO : don't need a nested object here, just the status name
    pub status: Status,
    pub created: DateTime<Local>,
    pub duedate: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct Filter {
    id: String,
    #[serde(rename = "self")]
    url: Url,
    name: String,
    jql: String,
}

fn get_filter(id: &str) -> Result<Filter, reqwest::Error> {
    let url = format!("{}/filter/{}", URL_BASE, id);
    Client::new().get(&url)
        .env_auth(ENV_VAR_PREFIX)
        .send()?
        .check_ok()
        .json()
}

fn get_issues(req: SearchRequest) -> Result<Vec<Issue>, reqwest::Error> {
    let url = format!("{}/search", URL_BASE);
    Ok(Client::new().post(&url)
        .env_auth(ENV_VAR_PREFIX)
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
            .env_auth(ENV_VAR_PREFIX)
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
