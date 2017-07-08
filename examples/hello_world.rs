extern crate susanoo;

use susanoo::{Context, Susanoo, AsyncResult, Router};
use susanoo::contrib::hyper::{Get, StatusCode};

fn hello(mut ctx: Context) -> AsyncResult {
    ctx.res.set_status(StatusCode::Ok);
    ctx.res.set_body("Hello, world");
    ctx.finish()
}

fn main() {
    let router = Router::default().with_route(Get, "/", hello);
    let susanoo = Susanoo::new().with(router);
    let server = susanoo.into_server("0.0.0.0:4000").unwrap();
    server.run().unwrap();
}
