use std::sync::Arc;
use hyper::server::Request;
use typemap::TypeMap;
use susanoo::States;
use unsafe_any::UnsafeAny;

/// An object which contains request data, parameters extracted by the router,
/// global/per-request shared variables.
pub struct Context {
    pub req: Request,
    pub map: TypeMap<UnsafeAny + Send>,
    pub states: Arc<States>,
}

impl Context {
    pub fn new(req: Request, states: Arc<States>) -> Self {
        Context {
            req,
            map: TypeMap::custom(),
            states,
        }
    }
}
