/// A simple wrapper around a successful HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Supported HTTP methods for requests.
#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

/// Sends an HTTP request and returns the raw response.
pub async fn request<T: serde::Serialize>(
    method: Method,
    url: &str,
    body: Option<&T>,
) -> Result<Response, String> {
    todo!()
}

/// Sends an HTTP request and deserializes the response as JSON.
pub async fn request_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    method: Method,
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    todo!()
}

/// Performs a GET request and returns the raw response.
pub async fn get(url: &str) -> Result<Response, String> {
    todo!()
}

/// Performs a GET request and deserializes the JSON response.
pub async fn get_json<R: serde::de::DeserializeOwned>(url: &str) -> Result<R, String> {
    todo!()
}

/// Performs a POST request with an optional JSON body and returns the raw response.
pub async fn post<T: serde::Serialize>(url: &str, body: Option<&T>) -> Result<Response, String> {
    todo!()
}

/// Performs a POST request with an optional JSON body and deserializes the JSON response.
pub async fn post_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    todo!()
}
