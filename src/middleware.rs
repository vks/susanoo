use std::sync::Arc;
use context::{Context, Status};
use result::AsyncResult;
use futures::{future, Future};

pub trait Middleware: 'static + Send + Sync {
    fn call(&self, ctx: Context) -> AsyncResult;
    fn call_ongoing(&self, ctx: Context) -> AsyncResult {
        future::ok(ctx).boxed()
    }
}

impl<F> Middleware for F
where
    F: 'static + Send + Sync + Fn(Context) -> AsyncResult,
{
    fn call(&self, ctx: Context) -> AsyncResult {
        (*self)(ctx)
    }
}


/// The chain of middlewares.
#[derive(Default)]
pub struct MiddlewareStack {
    middlewares: Vec<Arc<Middleware>>,
}

impl MiddlewareStack {
    pub fn push<M: Middleware>(&mut self, middleware: M) -> &mut Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    pub fn with<M: Middleware>(mut self, middleware: M) -> Self {
        self.push(middleware);
        self
    }

    pub fn iter(&self) -> ::std::slice::Iter<Arc<Middleware>> {
        self.middlewares.iter()
    }
}

impl Middleware for MiddlewareStack {
    fn call(&self, ctx: Context) -> AsyncResult {
        self.middlewares.iter().fold(
            future::ok(ctx).boxed(),
            |ctx, middleware| {
                let middleware = middleware.clone();
                ctx.and_then(move |ctx| match ctx.status {
                    Status::Ongoing => middleware.call(ctx),
                    Status::Finished => middleware.call_ongoing(ctx),
                }).boxed()
            },
        )
    }
}
