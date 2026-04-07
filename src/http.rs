//! This module is an adapter for HTTP related operations.
//! In the future, if we want to migrate to a different HTTP library,
//! we can easily change this facade instead of multiple places in the crate.

use reqwest::{blocking::Client, StatusCode};
use std::sync::OnceLock;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_RETRIES: u8 = 2;
const RETRY_BACKOFF: Duration = Duration::from_millis(250);

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error(transparent)]
#[diagnostic(code("nvc::http::error"))]
pub struct Error(#[from] reqwest::Error);

pub type Response = reqwest::blocking::Response;

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build HTTP client")
    })
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn should_retry_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout() || error.is_request()
}

pub fn get(url: &str) -> Result<Response, Error> {
    for attempt in 0..=MAX_RETRIES {
        let response = client()
            .get(url)
            // Some sites require a user agent.
            .header("User-Agent", concat!("nvc ", env!("CARGO_PKG_VERSION")))
            .send();

        match response {
            Ok(response) if attempt < MAX_RETRIES && should_retry_status(response.status()) => {
                log::warn!(
                    "HTTP {} from {} (attempt {}/{})",
                    response.status(),
                    response.url(),
                    attempt + 1,
                    MAX_RETRIES + 1
                );
                std::thread::sleep(RETRY_BACKOFF * u32::from(attempt + 1));
            }
            Ok(response) => return Ok(response),
            Err(error) if attempt < MAX_RETRIES && should_retry_error(&error) => {
                log::warn!(
                    "HTTP request retry for attempt {}/{} after {}",
                    attempt + 1,
                    MAX_RETRIES + 1,
                    error
                );
                std::thread::sleep(RETRY_BACKOFF * u32::from(attempt + 1));
            }
            Err(error) => return Err(Error::from(error)),
        }
    }

    unreachable!("retry loop must return or error")
}
