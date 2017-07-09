#[macro_use]
extern crate susanoo;
#[macro_use]
extern crate hyper;

use susanoo::{Susanoo, Context, AsyncResult, Router, Middleware, Chain};
use susanoo::contrib::hyper::{Get, StatusCode};
use susanoo::contrib::hyper::header::{Authorization, Basic};
use susanoo::contrib::typemap::Key;

header! {
    (WWWAuthenticate, "WWW-Authenticate") => [String]
}


#[derive(Clone)]
struct User {
    username: String,
    password: String,
}

impl User {
    fn verify(&self, username: &str, password: &str) -> bool {
        &self.username == username && &self.password == password
    }
}

impl Key for User {
    type Value = Self;
}


struct UserList(Vec<User>);

impl std::ops::Deref for UserList {
    type Target = Vec<User>;
    fn deref(&self) -> &Vec<User> {
        &self.0
    }
}

impl Key for UserList {
    type Value = Self;
}

impl Middleware for UserList {
    fn call(&self, mut ctx: Context) -> AsyncResult {
        ctx.insert_ext(UserList(self.0.clone()));
        ctx.next_middleware()
    }
}



fn check_auth(mut ctx: Context) -> AsyncResult {
    let auth: Option<Authorization<_>> = ctx.req
        .headers
        .get::<Authorization<Basic>>()
        .map(|a| a.clone());

    let (username, password) = match auth {
        Some(Authorization(Basic {
                               ref username,
                               password: Some(ref password),
                           })) => (username, password),
        _ => {
            ctx.res.set_status(StatusCode::Unauthorized);
            ctx.res.headers_mut().set(WWWAuthenticate(
                "Basic realm=\"main\""
                    .to_owned(),
            ));
            return ctx.finish();
        }
    };

    let found: Option<User> = ctx.get_ext_ref::<UserList>()
        .unwrap()
        .iter()
        .find(|&user| user.verify(username, password))
        .map(|u| u.clone());
    match found {
        Some(user) => {
            ctx.insert_ext(user);
        }
        None => {
            ctx.res.set_status(StatusCode::Unauthorized);
            ctx.res.headers_mut().set(WWWAuthenticate(
                "Basic realm=\"main\""
                    .to_owned(),
            ));
            return ctx.finish();
        }
    }

    ctx.next_middleware()
}



fn index(mut ctx: Context) -> AsyncResult {
    let user = ctx.get_ext::<User>().unwrap();
    ctx.res.set_status(StatusCode::Ok);
    ctx.res.set_body(format!(
        "<h1>Welcome, {}!</h1>",
        user.username
    ));
    ctx.finish()
}

fn public(mut ctx: Context) -> AsyncResult {
    ctx.res.set_status(StatusCode::Ok);
    ctx.res.set_body("<h1>Public page</h1>");
    ctx.finish()
}

fn main() {
    let users = vec![
        User {
            username: "alice".to_owned(),
            password: "wonderland".to_owned(),
        },
    ];

    let index = chain!(check_auth, index);

    let router = Router::default()
        .with_route(Get, "/", index)
        .with_route(Get, "/public", public);

    let susanoo = Susanoo::new(chain!(UserList(users), router));

    let server = susanoo.into_server("0.0.0.0:4000").unwrap();
    server.run().unwrap();
}
