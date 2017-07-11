extern crate susanoo;

use susanoo::{Context, Susanoo, AsyncResult, Router, OwnedCaptures};
use susanoo::contrib::hyper::{Get, Post, StatusCode, Response};
use susanoo::contrib::futures::{Future, Stream};


fn index(mut ctx: Context) -> AsyncResult {
    let res = Response::new()
        .with_status(StatusCode::Ok)
        .with_body("Hello, world");
    ctx.finish(res)
}

fn index_post(mut ctx: Context) -> AsyncResult {
    let body = ctx.req.take_body().unwrap();
    body.collect()
        .map_err(Into::into)
        .and_then(move |chunks| {
            let mut body = Vec::new();
            for chunk in chunks {
                body.extend_from_slice(&chunk);
            }
            let res = Response::new()
                .with_status(StatusCode::Ok)
                .with_body(format!("Posted: {}", String::from_utf8_lossy(&body)));
            ctx.finish(res)
        })
        .boxed()
}

fn show_captures(mut ctx: Context) -> AsyncResult {
    let cap = ctx.ext.remove::<OwnedCaptures>().unwrap();
    let res = Response::new()
        .with_status(StatusCode::Ok)
        .with_body(format!(
            "Captures: {:?}, {:?}, {:?} ({:?})",
            &cap[1],
            &cap["hoge"],
            &cap[3],
            cap
        ));
    ctx.finish(res)
}

fn main() {
    let router = Router::default()
        .with_route(Get, "/", index)
        .with_route(Post, "/", index_post)
        .with_route(Post, "/post", index_post)
        .with_route(Get, r"/echo/([^/]+)/(?P<hoge>[^/]+)/([^/]+)", show_captures);
    let susanoo = Susanoo::new(router);
    let server = susanoo.into_server("0.0.0.0:4000").unwrap();
    server.run().unwrap();
}
