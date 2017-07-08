extern crate susanoo;

use susanoo::{Context, Susanoo, AsyncResult, Router, Captures};
use susanoo::contrib::hyper::{Get, Post, StatusCode};
use susanoo::contrib::futures::{Future, Stream};


fn index(mut ctx: Context) -> AsyncResult {
    ctx.res.set_status(StatusCode::Ok);
    ctx.res.set_body("Hello, world");
    ctx.finish()
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
            ctx.res.set_status(StatusCode::Ok);
            ctx.res.set_body(format!(
                "Posted: {}",
                String::from_utf8_lossy(&body)
            ));
            ctx.finish()
        })
        .boxed()
}

fn show_captures(mut ctx: Context) -> AsyncResult {
    {
        let cap = ctx.ext.get::<Captures>().unwrap();
        ctx.res.set_status(StatusCode::Ok);
        ctx.res.set_body(format!("Captures: {:?}", cap));
    }
    ctx.finish()
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
