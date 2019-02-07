//! SPARQL client

// result {"head":{"vars":["p"]},"results":{"bindings":[{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.IllegalArgumentException"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.RuntimeException"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.Exception"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.Throwable"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.Object"}}]}}

// https://www.w3.org/TR/sparql11-results-json/

use std::collections::HashMap;

use reqwest::Client;

use crate::http_util::AuthRequest;
use crate::http_util::CheckedResponse;

static APPLICATION_JSON: &str = "application/json";
static APPLICATION_SPARQL_RESULTS_JSON: &str = "application/sparql-results+json";

static QUERY_PARAMETER: &str = "query";
static REASONING_PARAMETER: &str = "reasoning";

/// Env var prefixed used with [`crate::http_util::AuthRequest`]
static ENV_VAR_PREFIX: &str = "STARDOG";

#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct SelectResult {
    results: Bindings,
}

// TODO : this is temp until I figure out how to tell serde
//        to get results.bindings directly in SelectResults
#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct Bindings {
    bindings: Vec<Solution>,
}

pub type Solution = HashMap<String, RdfTerm>;

#[derive(PartialEq, Eq, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RdfTerm {
    #[serde(rename = "uri")]
    // TODO : if serde would accept `content = "value"` here
    //        we could just do Iri(String)
    Iri { value: String },
    #[serde(rename = "literal")]
    // TODO : handle langString
    Literal { value: String, datatype: String },
    #[serde(rename = "bnode")]
    Bnode { value: String },
}

/// Context used to interact with SPARQL endpoint
pub struct SparqlContext {
    /// Endpoint URL include DB name, but not query/update.
    /// e.g. `https://localhost/stardog/db`
    endpoint: String,

    reasoning: bool,
}

impl SparqlContext {
    pub fn query<T: Into<String>>(&self, sparql_query: T) -> Result<SelectResult, reqwest::Error> {
        Client::new().get(&format!("{}/query", self.endpoint))
            .env_auth(ENV_VAR_PREFIX)
            .header(http::header::ACCEPT, APPLICATION_SPARQL_RESULTS_JSON)
            .query(&[("query", sparql_query.into()),
                ("reasoning", format!("{}", self.reasoning))])
            .send()?
            .check_ok()
            .json()
    }
}

// TODO : tests are assuming some existing data, should be self-contained
#[cfg(test)]
mod test {
    use super::*;

    static BASE_URL: &str = "https://localhost/stardog/jora";

    #[test]
    fn first_query() -> Result<(), reqwest::Error> {
        let client = Client::new();
        let mut res: Result<SelectResult, reqwest::Error> =
            client.get(&format!("{}/query", BASE_URL))
                .env_auth(ENV_VAR_PREFIX)
                .header(http::header::ACCEPT, APPLICATION_SPARQL_RESULTS_JSON)
                .query(&[("query", r#"
                               select * where {  javap:java.sql.Date javap:subClassOf ?p.  }
                               "#)])
                .send()?
                .check_ok()
                .json();

        println!("Result deserialized: {:?}", res.unwrap());

        Ok(())
    }

    #[test]
    fn basic_query() -> Result<(), reqwest::Error> {
        let ctx = SparqlContext {
            endpoint: String::from(BASE_URL),
            reasoning: false,
        };
        let query = "select * where {  javap:java.sql.Date javap:subClassOf ?p.  }";
        let res = ctx.query(query);
        assert_eq!(true, res.is_ok());
        let solutions = res.unwrap().results.bindings;
        assert_eq!(1, solutions.len());
        let only = solutions.first().unwrap();
        assert_eq!(true, only.contains_key("p"));
        assert_eq!(&RdfTerm::Iri { value: String::from("http://jbalint/javap#java.util.Date") }, only.get("p").unwrap());
        Ok(())
    }

    #[test]
    fn reasoning_query() -> Result<(), reqwest::Error> {
        let ctx = SparqlContext {
            endpoint: String::from(BASE_URL),
            reasoning: true,
        };
        let query = "select * where {  javap:java.sql.Date javap:subClassOf ?p.  } order by ?p";
        let res = ctx.query(query);
        assert_eq!(true, res.is_ok());
        let solutions = res.unwrap().results.bindings;
        assert_eq!(2, solutions.len());
        let v = solutions.get(0).unwrap();
        assert_eq!(&RdfTerm::Iri { value: String::from("http://jbalint/javap#java.lang.Object") }, v.get("p").unwrap());
        let v = solutions.get(1).unwrap();
        assert_eq!(&RdfTerm::Iri { value: String::from("http://jbalint/javap#java.util.Date") }, v.get("p").unwrap());
        Ok(())
    }
}
