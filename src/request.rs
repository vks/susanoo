use hyper::{Request as HyperRequest, Method, Uri, HttpVersion, Headers, Body};


/// HTTP request, reconstructed from `hyper::Request`.
pub struct Request {
    pub method: Method,
    pub http_version: HttpVersion,
    pub uri: Uri,
    pub headers: Headers,
    body: Option<Body>,
}

impl From<HyperRequest> for Request {
    fn from(req: HyperRequest) -> Self {
        let (method, uri, http_version, headers, body) = req.deconstruct();
        Request {
            method,
            uri,
            http_version,
            headers,
            body: Some(body),
        }
    }
}

impl Request {
    /// Returns the path of request URL.
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Takes the value of request body with its ownership,
    /// and put `None` to its place as a substitute.
    ///
    /// If the body has already been taken out, the method will return a `None`.
    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }
}
