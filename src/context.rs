use hyper::{Request as HyperRequest, Response, Method, Uri, HttpVersion, Headers, Body};
use typemap::{SendMap, Key};
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
    pub status: Status,
    ext: SendMap,
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

    pub fn get_ext<T: Key<Value = T> + Send + Clone>(&self) -> Option<T> {
        self.ext.get::<T>().map(|v| v.clone())
    }

    pub fn get_ext_ref<T: Key<Value = T> + Send>(&self) -> Option<&T> {
        self.ext.get::<T>()
    }

    pub fn get_ext_mut<T: Key<Value = T> + Send>(&mut self) -> Option<&mut T> {
        self.ext.get_mut::<T>()
    }

    pub fn insert_ext<T: Key<Value = T> + Send>(&mut self, val: T) -> Option<T> {
        self.ext.insert::<T>(val)
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
