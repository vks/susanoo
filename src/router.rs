use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::error::Error as StdError;
use std::sync::Arc;
use futures::{future, Future};
use hyper::server::Response;
use hyper::{Method, StatusCode};
use regex::Regex;

use context::Context;
use middleware::Middleware;
use result::{AsyncResult, Failure};
use regex_pattern::{RegexPattern, OwnedCaptures};


#[derive(Debug)]
pub struct NoRoute;

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no route")
    }
}

impl StdError for NoRoute {
    fn description(&self) -> &str {
        "no route"
    }
}


struct Route {
    pattern: RegexPattern,
    middleware: Arc<Middleware>,
}


#[derive(Default)]
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
}

impl Router {
    /// Add a new route matching both method and given regexp pattern.
    pub fn add_route<S, M>(&mut self, method: Method, pattern: S, middleware: M) -> &mut Self
    where
        S: AsRef<str>,
        M: Middleware,
    {
        let pattern = normalize_pattern(pattern.as_ref());
        let pattern = Regex::new(&pattern).unwrap();
        self.routes
            .entry(method)
            .or_insert(Vec::new())
            .push(Route {
                pattern: pattern.into(),
                middleware: Arc::new(middleware),
            });
        self
    }

    /// Add a new route and return itself as return value.
    ///
    /// This method is useful for builder-style pattern.
    pub fn with_route<S, M>(mut self, method: Method, pattern: S, middleware: M) -> Self
    where
        S: AsRef<str>,
        M: Middleware,
    {
        self.add_route(method, pattern, middleware);
        self
    }

    pub(crate) fn recognize(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<(&Middleware, OwnedCaptures), NoRoute> {
        let routes = self.routes.get(method).ok_or(NoRoute)?;
        for route in routes {
            if let Some(caps) = route.pattern.owned_captures(path) {
                return Ok((&*route.middleware, caps));
            }
        }
        Err(NoRoute)
    }
}

impl Middleware for Router {
    fn call(&self, mut ctx: Context) -> AsyncResult {
        match self.recognize(&ctx.req.method, &ctx.req.path()) {
            Ok((middleware, cap)) => {
                ctx.ext.insert::<OwnedCaptures>(cap);
                middleware.call(ctx)
            }
            Err(err) => {
                future::err(Failure::from(err).with_response(
                    Response::new().with_status(
                        StatusCode::NotFound,
                    ),
                )).boxed()
            }
        }
    }
}



fn normalize_pattern(pattern: &str) -> Cow<str> {
    let pattern = pattern
        .trim()
        .trim_left_matches("^")
        .trim_right_matches("$")
        .trim_right_matches("/");
    match pattern {
        "" => "^/$".into(),
        s => format!("^{}/?$", s).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_pattern;

    #[test]
    fn normalize_cases() {
        assert_eq!(normalize_pattern("/"), "^/$");
        assert_eq!(normalize_pattern("/path/to"), "^/path/to/?$");
        assert_eq!(normalize_pattern("/path/to/"), "^/path/to/?$");
    }
}
