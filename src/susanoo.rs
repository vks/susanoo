use std::io;
use std::sync::Arc;

use futures::{future, Future};
use futures::future::BoxFuture;
use hyper::Error as HyperError;
use hyper::{StatusCode, Server, Chunk};
use hyper::server::{Http, Service, NewService, Response};
use hyper::server::Request;
use typemap::{TypeMap, Key};
use unsafe_any::UnsafeAny;

use context::Context;
use middleware::Middleware;
use router::Router;
use response::Success;


pub type States = TypeMap<UnsafeAny + 'static + Send + Sync>;


/// Internal state of server
#[doc(visible)]
pub(crate) struct ServerInner {
    router: Router,
    middlewares: Vec<Arc<Middleware>>,
    states: Arc<States>,
}


/// Root object of the application.
pub struct Susanoo {
    inner: Arc<ServerInner>,
}

impl Susanoo {
    /// Creates an empty instance of the server.
    pub fn new(router: Router) -> Self {
        Susanoo {
            inner: Arc::new(ServerInner {
                router,
                middlewares: Vec::new(),
                states: Arc::new(States::custom()),
            }),
        }
    }

    /// Put a middleware into the server.
    pub fn with_middleware<M: Middleware>(mut self, middleware: M) -> Self {
        Arc::get_mut(&mut self.inner)
            .unwrap()
            .middlewares
            .push(Arc::new(middleware));
        self
    }

    /// Insert a shared state into the server.
    pub fn with_state<T: Key<Value = T> + Send + Sync>(mut self, value: T) -> Self {
        {
            let inner = Arc::get_mut(&mut self.inner).unwrap();
            let mut states = Arc::get_mut(&mut inner.states).unwrap();
            states.insert::<T>(value);
        }
        self
    }

    /// Run server.
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
        // apply router
        let method = req.method().clone();
        let path = req.path().to_owned();
        match self.inner.router.recognize(&method, &path) {
            Ok((middleware, cap)) => {
                let ctx = future::ok(Context::new(req, cap, self.inner.states.clone()).into())
                    .boxed();

                // apply middlewares
                let ctx = self.inner
                    .middlewares
                    .iter()
                    .chain(vec![middleware].iter())
                    .fold(ctx, |ctx, middleware| {
                        let middleware = middleware.clone();
                        ctx.and_then(move |ctx| match ctx {
                            Success::Continue(ctx) => middleware.call(ctx),
                            Success::Finished(res) => future::ok(res.into()).boxed(),
                        }).boxed()
                    });

                // convert to Hyper response
                ctx.then(|resp| match resp {
                    Ok(Success::Finished(resp)) => Ok(resp),
                    Ok(Success::Continue(_ctx)) => Ok(Response::new().with_status(
                        StatusCode::NotFound,
                    )),
                    Err(failure) => Ok(
                        failure.response.unwrap_or(
                            Response::new()
                                .with_status(StatusCode::InternalServerError)
                                .with_body(format!("Internal Server Error: {:?}", failure.err)),
                        ),
                    ),
                }).boxed()
            }
            Err(code) => future::ok(Response::new().with_status(code)).boxed(),
        }
    }
}
