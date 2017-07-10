use hyper::{Request as HyperRequest, Response};
use typemap::{SendMap, Key};
use futures::{future, Future};
use result::AsyncResult;
use request::Request;


pub enum Status {
    Finished,
    Ongoing,
}


/// A context during handling.
///
/// It contains an HTTP request, HTTP response to return,
/// and a typemap in order to share variables between middlewares.
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
