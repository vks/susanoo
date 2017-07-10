use hyper::{Request as HyperRequest, Method, HttpVersion, Headers, Body};
use url::Url;


/// HTTP request, reconstructed from `hyper::Request`.
pub struct Request {
    pub method: Method,
    pub http_version: HttpVersion,
    pub url: Url,
    pub headers: Headers,
    body: Option<Body>,
}

impl From<HyperRequest> for Request {
    fn from(req: HyperRequest) -> Self {
        let (method, uri, http_version, headers, body) = req.deconstruct();
        // TODO: treat url::ParseError
        let url = Url::parse(uri.as_ref()).unwrap();
        Request {
            method,
            url,
            http_version,
            headers,
            body: Some(body),
        }
    }
}

impl Request {
    /// Returns the path of request URL.
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// Takes the value of request body with its ownership,
    /// and put `None` to its place as a substitute.
    ///
    /// If the body has already been taken out, the method will return a `None`.
    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }
}
