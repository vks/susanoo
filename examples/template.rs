extern crate susanoo;
extern crate tera;

use susanoo::{Susanoo, Router, Context, AsyncResult, Middleware};
use susanoo::contrib::hyper::{Get, StatusCode};
use susanoo::contrib::hyper::header::ContentType;
use susanoo::contrib::typemap::Key;
use std::sync::Arc;
use tera::{Tera, Context as TeraContext};

#[derive(Clone)]
struct TeraMiddleware(Arc<Tera>);

impl TeraMiddleware {
    fn new() -> Self {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "index.html",
            r#"
            <html>
            <head>
                <title>Susanoo example</title>
            </head>
            <body>
                <h1>Hi, {{ name }}!</h1>
                <p>
                    {{ text }}
                </p>
            </body>
            </html>
            "#,
        ).unwrap();
        TeraMiddleware(Arc::new(tera))
    }
}

impl Key for TeraMiddleware {
    type Value = Self;
}

impl Middleware for TeraMiddleware {
    fn call(&self, mut ctx: Context) -> AsyncResult {
        ctx.insert_ext::<TeraMiddleware>(self.clone());
        ctx.next_middleware()
    }
}


trait TeraMiddlewareExt {
    fn render(&mut self, name: &str, ctx: &TeraContext);
}

impl TeraMiddlewareExt for Context {
    fn render(&mut self, name: &str, ctx: &TeraContext) {
        let body = {
            let tera = self.get_ext_ref::<TeraMiddleware>().unwrap();
            tera.0.render(name, ctx).unwrap()
        };
        self.res.headers_mut().set(ContentType::html());
        self.res.set_body(body);
    }
}


fn index(mut ctx: Context) -> AsyncResult {
    let mut tera_ctx = TeraContext::default();
    tera_ctx.add("name", &"Alice".to_owned());
    tera_ctx.add("text", &"Welcome to the wonderland".to_owned());

    ctx.res.set_status(StatusCode::Ok);
    ctx.render("index.html", &tera_ctx);
    ctx.finish()
}

fn main() {
    let tera = TeraMiddleware::new();
    let router = Router::default().with_route(Get, "/", index);
    let susanoo = Susanoo::new().with(tera).with(router);
    let server = susanoo.into_server("0.0.0.0:4000").unwrap();

    server.run().unwrap();
}
