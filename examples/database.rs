#[macro_use]
extern crate susanoo;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use susanoo::{Context, Susanoo, AsyncResult, Router, Middleware, Chain};
use susanoo::contrib::hyper::{Get, StatusCode, Response};
use susanoo::contrib::futures::{future, Future};
use susanoo::contrib::typemap::Key;

use std::ops::Deref;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection as SqliteConnection;


// DB connection pool.
struct DBPool(Pool<SqliteConnectionManager>);

impl Deref for DBPool {
    type Target = Pool<SqliteConnectionManager>;
    fn deref(&self) -> &Pool<SqliteConnectionManager> {
        &self.0
    }
}

impl Key for DBPool {
    type Value = Self;
}


struct DBMiddleware(Pool<SqliteConnectionManager>);

impl DBMiddleware {
    fn new(path: &str) -> Self {
        let manager = SqliteConnectionManager::new(path);
        let pool = r2d2::Pool::new(Default::default(), manager).unwrap();
        DBMiddleware(pool)
    }
}

impl Middleware for DBMiddleware {
    fn call(&self, mut ctx: Context) -> AsyncResult {
        ctx.ext.insert::<DBPool>(DBPool(self.0.clone()));
        ctx.next()
    }
}



// Model
#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

impl Person {
    fn new(id: i32, name: &str) -> Self {
        Person {
            id,
            name: name.to_owned(),
            data: None,
        }
    }

    fn from_row(row: &rusqlite::Row) -> Self {
        Person {
            id: row.get(0),
            name: row.get(1),
            data: row.get(2),
        }
    }

    fn insert(&self, conn: &SqliteConnection) -> rusqlite::Result<i32> {
        conn.execute(
            "INSERT INTO persons (name, data) VALUES (?1, ?2)",
            &[&self.name, &self.data],
        )
    }

    fn create_table(conn: &SqliteConnection) -> rusqlite::Result<()> {
        conn.execute(
            r#"CREATE TABLE persons (
                id    INTEGER   PRIMARY KEY
              , name  TEXT      NOT NULL
              , data  BLOB
            )"#,
            &[],
        ).map(|_| ())
    }

    fn select(conn: &SqliteConnection) -> rusqlite::Result<Vec<Person>> {
        let mut stmt = conn.prepare("SELECT id,name,data FROM persons")?;
        let people = stmt.query_map(&[], Person::from_row)?
            .collect::<Result<_, _>>()?;
        Ok(people)
    }
}


fn index(mut ctx: Context) -> AsyncResult {
    let people = {
        let db = ctx.ext.get::<DBPool>().unwrap();
        let conn = try_f!(db.get());
        try_f!(Person::select(&*conn))
    };
    ctx.finish(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("people: {:?}", people)),
    )
}


fn init_db(path: &str) {
    let _ = std::fs::remove_file(path);
    let conn = SqliteConnection::open(path).unwrap();

    Person::create_table(&conn).unwrap();

    let me = Person::new(0, "Bob");
    me.insert(&conn).unwrap();
}


fn main() {
    init_db("app.sqlite");

    let db = DBMiddleware::new("app.sqlite");
    let router = Router::default().with_route(Get, "/", index);
    let susanoo = Susanoo::new(chain!(db, router));

    let server = susanoo.into_server("0.0.0.0:4000").unwrap();
    server.run().unwrap();
}
