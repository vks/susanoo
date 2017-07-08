extern crate susanoo;

use susanoo::{Context, Susanoo, Response, AsyncResult, Router, Captures};
use susanoo::contrib::hyper::{Get, Post, StatusCode};
use susanoo::contrib::futures::{future, Future, Stream};


fn index(_ctx: Context) -> AsyncResult {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world")
            .into(),
    ).boxed()
}

fn index_post(ctx: Context) -> AsyncResult {
    ctx.req
        .body()
        .collect()
        .and_then(|chunks| {
            let mut body = Vec::new();
            for chunk in chunks {
                body.extend_from_slice(&chunk);
            }
            future::ok(
                Response::new()
                    .with_status(StatusCode::Ok)
                    .with_body(format!("Posted: {}", String::from_utf8_lossy(&body)))
                    .into(),
            )
        })
        .map_err(Into::into)
        .boxed()
}

fn show_captures(ctx: Context) -> AsyncResult {
    let cap = ctx.map.get::<Captures>().unwrap();
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", cap))
            .into(),
    ).boxed()
}

fn main() {
    let router = Router::default()
        .with_route(Get, "/", index)
        .with_route(Post, "/", index_post)
        .with_route(Post, "/post", index_post)
        .with_route(Get, r"/echo/([^/]+)/(?P<hoge>[^/]+)/([^/]+)", show_captures);
    let susanoo = Susanoo::new().with(router);
    let server = susanoo.into_server("0.0.0.0:4000").unwrap();
    server.run().unwrap();
}
