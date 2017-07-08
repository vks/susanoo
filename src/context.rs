use std::sync::Arc;
use hyper::server::Request;
use typemap::TypeMap;
use susanoo::States;
use unsafe_any::UnsafeAny;

/// Captured value extracted by the router.
pub type Captures = Vec<(Option<String>, String)>;

/// An object which contains request data, parameters extracted by the router,
/// global/per-request shared variables.
pub struct Context {
    pub req: Request,
    pub cap: Option<Captures>,
    pub map: TypeMap<UnsafeAny + Send>,
    pub states: Arc<States>,
}

impl Context {
    pub fn new(req: Request, states: Arc<States>) -> Self {
        Context {
            req,
            cap: None,
            map: TypeMap::custom(),
            states,
        }
    }
}
