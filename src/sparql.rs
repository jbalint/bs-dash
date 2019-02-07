//! SPARQL client

// result {"head":{"vars":["p"]},"results":{"bindings":[{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.IllegalArgumentException"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.RuntimeException"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.Exception"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.Throwable"}},{"p":{"type":"uri","value":"http://jbalint/javap#java.lang.Object"}}]}}

// https://www.w3.org/TR/sparql11-results-json/

use std::collections::HashMap;

use reqwest::Client;

use crate::http_util::AuthRequest;
use crate::http_util::CheckedResponse;

static APPLICATION_JSON: &str = "application/json";
static APPLICATION_SPARQL_RESULTS_JSON: &str = "application/sparql-results+json";

/// Env var prefixed used with [`crate::http_util::AuthRequest`]
static ENV_VAR_PREFIX: &str = "STARDOG";

#[derive(PartialEq, Eq, Deserialize, Debug)]
struct SelectResult {
    results: Bindings,
}

// TODO : this is temp until I figure out how to tell serde
//        to get results.bindings directly in SelectResults
#[derive(PartialEq, Eq, Deserialize, Debug)]
struct Bindings {
    bindings: Vec<Solution>,
}

type Solution = HashMap<String, RdfTerm>;

#[derive(PartialEq, Eq, Deserialize, Debug)]
#[serde(tag = "type")]
enum RdfTerm {
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

fn query(sparql_query: String) -> Result<SelectResult, reqwest::Error> {
    let client = Client::new();
    let mut res: Result<SelectResult, reqwest::Error> =
        client.get("https://localhost/stardog/jora/query")
            .env_auth(ENV_VAR_PREFIX)
            .header(http::header::ACCEPT, APPLICATION_SPARQL_RESULTS_JSON)
            .query(&[("query", r#"
                               select * where {  javap:java.sql.Date javap:subClassOf ?p.  }
                               "#)])
            .send()?
            .check_ok()
            .json()
        ;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn first_query() -> Result<(), reqwest::Error> {
        let client = Client::new();
        let mut res: Result<SelectResult, reqwest::Error> =
            client.get("https://localhost/stardog/jora/query")
                .env_auth(ENV_VAR_PREFIX)
                .header(http::header::ACCEPT, APPLICATION_SPARQL_RESULTS_JSON)
                .query(&[("query", r#"
                               select * where {  javap:java.sql.Date javap:subClassOf ?p.  }
                               "#)])
                .send()?
                .check_ok()
                .json()
            ;

        println!("Result deserialized: {:?}", res.unwrap());

        Ok(())
    }
}
