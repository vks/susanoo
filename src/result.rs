use std::error::Error as StdError;
use hyper::Response;
use futures::future::BoxFuture;
use context::Context;

/// Asynchronous result type
pub type AsyncResult = BoxFuture<Context, Failure>;


/// Error type
pub struct Failure {
    pub err: Box<StdError + Send + 'static>,
    pub response: Option<Response>,
}

impl<E: StdError + 'static + Send> From<E> for Failure {
    fn from(err: E) -> Self {
        Failure {
            err: Box::new(err),
            response: None,
        }
    }
}

impl Failure {
    pub fn with_response(mut self, response: Response) -> Self {
        self.response = Some(response);
        self
    }
}


#[macro_export]
macro_rules! try_f {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => return future::err(err.into()).boxed(),
    });
}
