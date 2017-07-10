use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::error::Error as StdError;
use std::sync::Arc;
use futures::{future, Future};
use hyper::server::Response;
use hyper::{Method, StatusCode};
use regex::Regex;
use typemap::Key;

use context::Context;
use middleware::Middleware;
use result::{AsyncResult, Failure};


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
    pattern: Regex,
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
                pattern,
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
            if let Some(caps) = get_owned_captures(&route.pattern, path) {
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
                ctx.insert_ext(cap);
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


/// Captured value extracted by the router.
#[derive(Debug, Clone)]
pub struct OwnedCaptures {
    matches: Vec<(Option<String>, String)>,
    names: HashMap<String, usize>,
}

impl OwnedCaptures {
    pub fn get(&self, i: usize) -> Option<&str> {
        self.matches.get(i).map(|&(_, ref s)| s.as_str())
    }

    pub fn name(&self, name: &str) -> Option<&str> {
        self.names.get(name).map(|&i| {
            self.matches[i].1.as_str()
        })
    }
}

impl Key for OwnedCaptures {
    type Value = Self;
}

fn get_owned_captures(re: &Regex, s: &str) -> Option<OwnedCaptures> {
    re.captures(s).map(|caps| {
        let mut matches = Vec::with_capacity(caps.len());
        let mut names = HashMap::new();

        for (i, name) in re.capture_names().enumerate() {
            match name {
                Some(name) => {
                    let m = caps.name(name).unwrap();
                    matches.push((Some(name.to_owned()), m.as_str().to_owned()));
                    names.insert(name.to_owned(), i);
                }
                None => {
                    let m = caps.get(i).unwrap();
                    matches.push((None, m.as_str().to_owned()));
                }
            }
        }
        OwnedCaptures { matches, names }
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
