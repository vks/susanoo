extern crate susanoo;

use susanoo::{Context, Susanoo, AsyncResult, Router};
use susanoo::contrib::hyper::{Get, StatusCode, Response};

fn hello(ctx: Context) -> AsyncResult {
    let res = Response::new()
        .with_status(StatusCode::Ok)
        .with_body("Hello, world");
    ctx.finish(res)
}

fn main() {
    let router = Router::default().with_route(Get, "/", hello);
    let susanoo = Susanoo::new(router);
    let server = susanoo.into_server("0.0.0.0:4000").unwrap();
    server.run().unwrap();
}
