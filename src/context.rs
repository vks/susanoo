use hyper::{Request as HyperRequest, Method, Uri, HttpVersion, Headers, Body};
use typemap::SendMap;

pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub http_version: HttpVersion,
    pub headers: Headers,
    pub body: Option<Body>,
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
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }
}


/// An object which contains request data, parameters extracted by the router,
/// global/per-request shared variables.
pub struct Context {
    pub req: Request,
    pub ext: SendMap,
}

impl Context {
    pub fn from_hyper(req: HyperRequest) -> Self {
        Context {
            req: req.into(),
            ext: SendMap::custom(),
        }
    }
}
