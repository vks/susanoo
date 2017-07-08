use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::error::Error as StdError;
use std::sync::Arc;
use hyper::{Method, StatusCode};
use hyper::server::Response;
use regex::Regex;
use typemap::Key;

use futures::{future, Future};
use response::{AsyncResult, Failure};
use context::Context;
use middleware::Middleware;


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


/// Captured value extracted by the router.
#[derive(Debug)]
pub struct Captures(Vec<(Option<String>, String)>);

impl Key for Captures {
    type Value = Self;
}


struct Route {
    pattern: Regex,
    middleware: Arc<Middleware>,
}


#[derive(Default)]
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
}

impl Router {
    pub fn with_route<S, M>(mut self, method: Method, pattern: S, middleware: M) -> Self
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
                pattern,
                middleware: Arc::new(middleware),
            });
        self
    }

    pub(crate) fn recognize(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<(Arc<Middleware>, Captures), NoRoute> {
        let routes = self.routes.get(method).ok_or(NoRoute)?;
        for route in routes {
            if let Some(caps) = get_owned_captures(&route.pattern, path) {
                return Ok((route.middleware.clone(), caps));
            }
        }
        Err(NoRoute)
    }
}

impl Middleware for Router {
    fn call(&self, mut ctx: Context) -> AsyncResult {
        match self.recognize(&ctx.req.method, &ctx.req.path()) {
            Ok((middleware, cap)) => {
                ctx.ext.insert::<Captures>(cap);
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


fn get_owned_captures(re: &Regex, path: &str) -> Option<Captures> {
    re.captures(path).map(|caps| {
        let mut res = Vec::with_capacity(caps.len());
        for (i, name) in re.capture_names().enumerate() {
            let val = match name {
                Some(name) => caps.name(name).unwrap(),
                None => caps.get(i).unwrap(),
            };
            res.push((name.map(|s| s.to_owned()), val.as_str().to_owned()));
        }
        Captures(res)
    })
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
