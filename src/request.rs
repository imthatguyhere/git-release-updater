/// A simple wrapper around a successful HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Same as Response but with raw bytes (for binary downloads).
#[derive(Debug, Clone)]
pub struct BytesResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Supported HTTP methods for requests.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

/// Builds a reqwest client with a user-agent header set.
fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("git-release-updater/0.1.0")
        .build()
        .expect("failed to build HTTP client")
}

/// Sends an HTTP GET and returns raw bytes.
pub async fn get_bytes(url: &str) -> Result<BytesResponse, String> {
    let client = build_client();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status().as_u16();
    let body = resp
        .bytes()
        .await
        .map_err(|e| format!("failed to read response body: {e}"))?
        .to_vec();

    Ok(BytesResponse { status, body })
}

/// Sends an HTTP request and returns the raw response.
pub async fn request<T: serde::Serialize>(
    method: Method,
    url: &str,
    body: Option<&T>,
) -> Result<Response, String> {
    let client = build_client();

    let req = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
        Method::Put => client.put(url),
        Method::Delete => client.delete(url),
    };

    let req = if let Some(b) = body {
        req.json(b)
    } else {
        req
    };

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status().as_u16();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("failed to read response body: {e}"))?;

    Ok(Response { status, body })
}

/// Sends an HTTP request and deserializes the response as JSON.
pub async fn request_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    method: Method,
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    let resp = request(method, url, body).await?;

    if !(200..300).contains(&resp.status) {
        return Err(format!(
            "HTTP {} from {url}: {}",
            resp.status,
            resp.body.lines().next().unwrap_or("")
        ));
    }

    serde_json::from_str::<R>(&resp.body)
        .map_err(|e| format!("failed to parse JSON from {url}: {e}"))
}

/// Performs a GET request and returns the raw response.
#[allow(dead_code)]
pub async fn get(url: &str) -> Result<Response, String> {
    request::<()>(Method::Get, url, None).await
}

/// Performs a GET request and deserializes the JSON response.
#[allow(dead_code)]
pub async fn get_json<R: serde::de::DeserializeOwned>(url: &str) -> Result<R, String> {
    request_json::<(), R>(Method::Get, url, None).await
}

/// Performs a POST request with an optional JSON body and returns the raw response.
#[allow(dead_code)]
pub async fn post<T: serde::Serialize>(url: &str, body: Option<&T>) -> Result<Response, String> {
    request(Method::Post, url, body).await
}

/// Performs a POST request with an optional JSON body and deserializes the JSON response.
#[allow(dead_code)]
pub async fn post_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    url: &str,
    body: Option<&T>,
) -> Result<R, String> {
    request_json(Method::Post, url, body).await
}

//=-- ---------------------------------------------------------------------------
//=-- Inline tests (private fn coverage only; public API tested via tests/)
//=-- ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_variants() {
        let _get = Method::Get;
        let _post = Method::Post;
        let _put = Method::Put;
        let _delete = Method::Delete;
    }

    #[test]
    fn test_response_construction() {
        let resp = Response {
            status: 200,
            body: "ok".into(),
        };
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "ok");
    }

    #[test]
    fn test_bytes_response_construction() {
        let resp = BytesResponse {
            status: 200,
            body: vec![0, 1, 2],
        };
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, vec![0, 1, 2]);
    }
}
