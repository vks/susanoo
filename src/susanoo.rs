use std::io;
use std::sync::Arc;

use futures::Future;
use futures::future::BoxFuture;
use hyper::Error as HyperError;
use hyper::{StatusCode, Server, Chunk};
use hyper::server::{Http, Service, NewService, Response};
use hyper::server::Request;

use context::Context;
use middleware::Middleware;


/// Internal state of server
pub(crate) struct ServerInner {
    middleware: Arc<Middleware>,
}


/// Root object of the application.
pub struct Susanoo {
    inner: Arc<ServerInner>,
}

impl Susanoo {
    /// Creates an empty instance of the server.
    pub fn new<M: Middleware>(middleware: M) -> Self {
        Susanoo { inner: Arc::new(ServerInner { middleware: Arc::new(middleware) }) }
    }

    /// Create server.
    pub fn into_server(self, addr: &str) -> Result<Server<Self, ::hyper::Body>, HyperError> {
        let addr = addr.parse().unwrap();
        Http::<Chunk>::new().bind(&addr, self)
    }
}

impl NewService for Susanoo {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Instance = SusanooService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(SusanooService { inner: self.inner.clone() })
    }
}


/// An asynchronous task executed by hyper.
pub struct SusanooService {
    inner: Arc<ServerInner>,
}

impl Service for SusanooService {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Future = BoxFuture<Response, HyperError>;

    fn call(&self, req: Request) -> Self::Future {
        let ctx = Context::from_hyper(req);

        self.inner
            .middleware
            .call(ctx)
            .then(|result| match result {
                Ok(ctx) => {
                    match ctx.res {
                        Some(res) => Ok(res),
                        None => Ok(Response::new().with_status(StatusCode::NotFound)),
                    }
                }
                Err(failure) => Ok(failure.response),
            })
            .boxed()
    }
}
