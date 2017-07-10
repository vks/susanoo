use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::error::Error as StdError;
use std::ops::Deref;
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
                pattern: RegexPattern::new(pattern),
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



struct RegexPattern {
    pattern: Regex,
    names: Arc<HashMap<String, usize>>,
}

impl RegexPattern {
    fn new(pattern: Regex) -> Self {
        let names = pattern
            .capture_names()
            .enumerate()
            .filter_map(|(i, name)| name.map(|name| (name.to_owned(), i)))
            .collect();
        RegexPattern {
            pattern,
            names: Arc::new(names),
        }
    }

    fn owned_captures(&self, text: &str) -> Option<OwnedCaptures> {
        self.pattern.captures(text).map(|caps| {
            let matches = caps.iter()
                .map(|cap| cap.map(|m| (m.start(), m.end())))
                .collect();

            OwnedCaptures {
                text: text.to_owned(),
                matches,
                names: self.names.clone(),
            }
        })
    }
}

impl Deref for RegexPattern {
    type Target = Regex;
    fn deref(&self) -> &Self::Target {
        &self.pattern
    }
}

/// Captured value extracted by the router.
#[derive(Debug, Clone)]
pub struct OwnedCaptures {
    text: String,
    matches: Vec<Option<(usize, usize)>>,
    names: Arc<HashMap<String, usize>>,
}

impl OwnedCaptures {
    pub fn get(&self, i: usize) -> Option<&str> {
        self.matches.get(i).and_then(|m| {
            m.map(|(start, end)| &self.text[start..end])
        })
    }

    pub fn name(&self, name: &str) -> Option<&str> {
        self.names.get(name).and_then(|&i| self.get(i))
    }
}

impl Key for OwnedCaptures {
    type Value = Self;
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
