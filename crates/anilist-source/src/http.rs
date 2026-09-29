use crate::SourceFuture;

/// Executes a request without knowing which anime source produced it.
pub trait HttpClient: Send + Sync {
    fn execute(&self, request: HttpRequest) -> SourceFuture<'_, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub body: Option<String>,
}

#[cfg(feature = "http")]
impl HttpClient for reqwest::Client {
    fn execute(&self, request: HttpRequest) -> SourceFuture<'_, String> {
        Box::pin(async move {
            use crate::error::SourceError;
            let method = match request.method {
                HttpMethod::Get => reqwest::Method::GET,
                HttpMethod::Post => reqwest::Method::POST,
            };
            let mut builder = self.request(method, request.url);
            for header in request.headers {
                builder = builder.header(header.name, header.value);
            }
            if let Some(body) = request.body {
                builder = builder.body(body);
            }
            let response = builder.send().await.map_err(|_| SourceError::Unavailable)?;
            if !response.status().is_success() {
                return Err(SourceError::Unavailable);
            }
            response.text().await.map_err(|_| SourceError::Unavailable)
        })
    }
}
