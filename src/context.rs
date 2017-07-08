use hyper::{Request as HyperRequest, Response, Method, Uri, HttpVersion, Headers, Body};
use typemap::SendMap;
use futures::{future, Future};
use result::AsyncResult;


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


pub enum Status {
    Finished,
    Ongoing,
}


/// An object which contains request data, parameters extracted by the router,
/// global/per-request shared variables.
pub struct Context {
    pub req: Request,
    pub res: Response,
    pub ext: SendMap,
    pub status: Status,
}

impl Context {
    pub fn from_hyper(req: HyperRequest) -> Self {
        Context {
            req: req.into(),
            res: Response::new(),
            ext: SendMap::custom(),
            status: Status::Ongoing,
        }
    }

    pub fn next_middleware(mut self) -> AsyncResult {
        self.status = Status::Ongoing;
        future::ok(self).boxed()
    }

    pub fn finish(mut self) -> AsyncResult {
        self.status = Status::Finished;
        future::ok(self).boxed()
    }
}
