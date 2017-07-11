use hyper::{Request as HyperRequest, Response};
use typemap::SendMap;
use futures::{future, Future};
use result::AsyncResult;
use request::Request;


/// A context during handling.
///
/// It contains an HTTP request, HTTP response to return,
/// and a typemap in order to share variables between middlewares.
pub struct Context {
    pub req: Request,
    pub ext: SendMap,
    pub res: Option<Response>,
}

impl Context {
    pub fn from_hyper(req: HyperRequest) -> Self {
        Context {
            req: req.into(),
            ext: SendMap::custom(),
            res: None,
        }
    }

    pub fn next(self) -> AsyncResult {
        future::ok(self).boxed()
    }

    pub fn finish(mut self, res: Response) -> AsyncResult {
        self.res = Some(res);
        self.next()
    }
}
