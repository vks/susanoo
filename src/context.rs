use hyper::server::Request;
use typemap::SendMap;

/// An object which contains request data, parameters extracted by the router,
/// global/per-request shared variables.
pub struct Context {
    pub req: Request,
    pub map: SendMap,
}

impl Context {
    pub fn new(req: Request) -> Self {
        Context {
            req,
            map: SendMap::custom(),
        }
    }
}
