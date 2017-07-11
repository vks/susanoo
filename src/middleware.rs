use std::sync::Arc;
use context::Context;
use result::AsyncResult;
use futures::{future, Future};

/// This traits represents a `Middleware`
pub trait Middleware: 'static + Send + Sync {
    /// Handler function called if the process has not finished yet
    /// (i.e. `ctx.status == Status::Ongoing`).
    fn call(&self, ctx: Context) -> AsyncResult;

    /// Handler function called if the process has done (i.e. `ctx.status == Status::Finished`).
    ///
    /// By default, this function returns a future contains given `ctx` immediately.
    fn after(&self, ctx: Context) -> AsyncResult {
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
pub struct Chain {
    middlewares: Vec<Arc<Middleware>>,
}

impl Chain {
    /// Put a middleware to tail of chains.
    pub fn push<M: Middleware>(&mut self, middleware: M) -> &mut Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Put a middleware to tail of chains, and return moved instance of `Chain`.
    ///
    /// The method is useful for builder style coding.
    pub fn with<M: Middleware>(mut self, middleware: M) -> Self {
        self.push(middleware);
        self
    }
}

impl Middleware for Chain {
    fn call(&self, ctx: Context) -> AsyncResult {
        self.middlewares.iter().fold(
            future::ok(ctx).boxed(),
            |ctx, middleware| {
                let middleware = middleware.clone();
                ctx.and_then(move |ctx| if ctx.res.is_some() {
                    middleware.after(ctx)
                } else {
                    middleware.call(ctx)
                }).boxed()
            },
        )
    }
}

#[macro_export]
macro_rules! chain {
    ($($m:expr),*) => {
        {
            let mut chain = Chain::default();
            $(
                chain.push($m);
            )*
            chain
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chain;
    use context::Context;
    use result::AsyncResult;

    #[test]
    fn chain_macro() {
        fn f1(ctx: Context) -> AsyncResult {
            ctx.next()
        }
        fn f2(ctx: Context) -> AsyncResult {
            ctx.next()
        }
        let _chain: Chain = chain!(f1, f2);
    }
}
