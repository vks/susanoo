use std::error::Error as StdError;
use hyper::{Response, StatusCode};
use futures::future::BoxFuture;
use context::Context;

/// Asynchronous result type
pub type AsyncResult = BoxFuture<Context, Failure>;


/// Error type
pub struct Failure {
    pub err: Box<StdError + Send + 'static>,
    pub response: Response,
}

impl<E: StdError + 'static + Send> From<E> for Failure {
    fn from(err: E) -> Self {
        let body = format!("Internal Server Error: {:?}", err);
        Failure {
            err: Box::new(err),
            response: Response::new()
                .with_status(StatusCode::InternalServerError)
                .with_body(body),
        }
    }
}

impl Failure {
    pub fn with_response(mut self, response: Response) -> Self {
        self.response = response;
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
