//! The real network of `--allow-network`: sends a request that no `[HttpClientHandler]` answers with ureq.

use al_host::network::{Network, NetworkRequest, NetworkResponse};
use std::time::Duration;

/// A response body larger than this is cut off with an error: a test never needs it whole.
const MAX_BODY_BYTES: u64 = 100 * 1024 * 1024;

pub struct UreqNetwork {
    agent: ureq::Agent,
}

impl UreqNetwork {
    pub fn new() -> UreqNetwork {
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(20)))
            .user_agent(concat!("al-runner/", env!("CARGO_PKG_VERSION")))
            .http_status_as_error(false)
            .build();
        UreqNetwork {
            agent: ureq::Agent::new_with_config(config),
        }
    }

    fn run<B: ureq::AsSendBody>(
        &self,
        request: ureq::http::Request<B>,
        timeout_ms: i64,
    ) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
        let timeout = Duration::from_millis(timeout_ms.max(1) as u64);
        let request = self
            .agent
            .configure_request(request)
            .timeout_global(Some(timeout))
            .build();
        self.agent.run(request)
    }
}

impl Default for UreqNetwork {
    fn default() -> Self {
        UreqNetwork::new()
    }
}

/// `content-type` as `Content-Type`: the http crate keeps names in lower case, BC shows them as the server sent them,
/// which is this form for the usual headers.
fn title_case(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join("-")
}

impl Network for UreqNetwork {
    fn send(&self, r: &NetworkRequest) -> Result<NetworkResponse, String> {
        let mut builder = ureq::http::Request::builder()
            .method(r.method.as_str())
            .uri(r.url.as_str());
        for (name, value) in &r.headers {
            // ureq sets these from the request itself.
            if name.eq_ignore_ascii_case("Content-Length") || name.eq_ignore_ascii_case("Host") {
                continue;
            }
            builder = builder.header(name.as_str(), value.as_str());
        }
        let sent = match &r.body {
            Some(body) => self.run(builder.body(body.clone()).map_err(|e| e.to_string())?, r.timeout_ms),
            None => self.run(builder.body(()).map_err(|e| e.to_string())?, r.timeout_ms),
        };
        let mut response = sent.map_err(|e| e.to_string())?;
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    title_case(name.as_str()),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect();
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_BYTES)
            .read_to_vec()
            .map_err(|e| e.to_string())?;
        Ok(NetworkResponse {
            status: status.as_u16(),
            reason: status.canonical_reason().unwrap_or("").to_string(),
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::title_case;

    #[test]
    fn header_names_come_back_in_title_case() {
        assert_eq!(title_case("content-type"), "Content-Type");
        assert_eq!(title_case("x-ms-request-id"), "X-Ms-Request-Id");
        assert_eq!(title_case("etag"), "Etag");
    }
}
