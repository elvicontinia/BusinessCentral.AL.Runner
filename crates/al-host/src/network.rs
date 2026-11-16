//! Real outbound HTTP (`--allow-network`): what a request that no `[HttpClientHandler]` answers sends, and what comes
//! back. Without a [`Network`] the runtime never opens a connection (ADR-027) and refuses such a request. The core
//! crates only call the trait; the front end gives the implementation that opens the connection (ADR-006).
//!
//! A request can carry secrets (an `Authorization` header from a SecretText, a body with a password), so neither type
//! implements `Debug`.

/// A request that goes out for real.
pub struct NetworkRequest {
    /// Upper case: `GET`, `POST`, ...
    pub method: String,
    /// The absolute uri.
    pub url: String,
    /// The client's default headers, the request's headers and the content's headers, in that order, values as sent.
    pub headers: Vec<(String, String)>,
    /// The content's bytes; `None` when the request has no content.
    pub body: Option<Vec<u8>>,
    /// The client's `Timeout`, in milliseconds.
    pub timeout_ms: i64,
}

/// What the server answered.
pub struct NetworkResponse {
    pub status: u16,
    pub reason: String,
    /// Every header of the response, the content headers among them, in the order received.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Opens connections.
pub trait Network: Send + Sync {
    /// Sends `request`. `Err` when no response came (no connection, a timeout, a TLS failure), with the reason for
    /// diagnostics; a response with any status is `Ok`.
    fn send(&self, request: &NetworkRequest) -> Result<NetworkResponse, String>;
}
