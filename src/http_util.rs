//! HTTP Utilities built around `reqwest` library

use http::StatusCode;
use reqwest::Response;
use reqwest::RequestBuilder;
use std::env;

// TODO : wasn't there something directly on Response that could do this similarly?
// TODO : don't panic
/// Check the HTTP status code in the response and `panic!()` if failed
pub trait CheckedResponse {
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

/// Add HTTP basic auth
pub trait AuthRequest {
    /// Using a pair of env vars prefixed by string provided in `var_prefix`
    fn env_auth<T: Into<String>>(self, var_prefix: T) -> Self;
}

impl AuthRequest for RequestBuilder {
    fn env_auth<T: Into<String>>(self, var_prefix: T) -> Self {
        let prefix: String = var_prefix.into();
        let username_var = format!("{}_USERNAME", prefix);
        let password_var = format!("{}_PASSWORD", prefix);
        let (username, password) =
            match (env::var(&username_var),
                   env::var(&password_var)) {
                (Ok(username), Ok(password)) => (username, password),
                _ => panic!("{} and/or {} not set", username_var, password_var),
            };

        self.basic_auth(username, Some(password))
    }
}
