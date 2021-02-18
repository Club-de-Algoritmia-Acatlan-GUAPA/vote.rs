#![feature(proc_macro_hygiene, decl_macro)]
#![feature(never_type)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_contrib;
extern crate itertools;
extern crate rcir;

mod schema;

use std::collections::HashSet;

use diesel::PgConnection;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, Form, FromRequest, Request};
use rocket::Rocket;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::{json::Json, templates::Template};

use schema::{Ballot, Item, NewUser, Vote};

#[database("postgresql_database")]
pub struct DbConn(PgConnection);

#[derive(Debug, Serialize)]
struct Context {
    winner: Option<Item>,
    second: Option<Item>,
    third: Option<Item>,
    items: Vec<(Item, Option<i32>)>,
}

impl Context {
    pub fn new(conn: &DbConn) -> Context {
        Context {
            winner: Vote::run_election(conn),
            second: None,
            third: None,
            items: Vec::new(), // not used if not logged in
        }
    }

    pub fn for_user(user: Auth, conn: &DbConn) -> Context {
        let winner = Vote::run_election(conn);
        let second = Vote::run_second_election(conn, &winner);
        let third = Vote::run_third_election(conn, &winner, &second);
        Context {
            winner,
            second,
            third,
            items: Item::for_user(user.0, conn),
        }
    }
}

#[derive(Debug)]
struct Auth(i32);

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Auth, !> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Auth)
            .or_forward(())
    }
}

#[post("/login", data = "<input>")]
fn login(mut cookies: Cookies, input: Form<NewUser>, conn: DbConn) -> Template {
    let user = input.into_inner();
    if user.username.is_empty() {
        index(conn)
    } else {
        let u = user.login(&conn);
        cookies.add_private(Cookie::new("user_id", u.id.to_string()));
        votes(Auth(u.id), conn)
    }
}

#[post("/vote", data = "<ballot>")]
fn vote(ballot: Json<Ballot>, user: Auth, conn: DbConn) -> &'static str {
    Vote::save_ballot(user.0, ballot.into_inner(), &conn);
    "voted"
}

#[get("/")]
fn votes(user: Auth, conn: DbConn) -> Template {
    Template::render("vote", Context::for_user(user, &conn))
}

#[get("/", rank = 2)]
fn index(conn: DbConn) -> Template {
    Template::render("index", Context::new(&conn))
}

#[head("/")]
fn index_head(conn: DbConn) -> Template {
    index(conn)
}

#[post("/mem", format = "application/json", data = "<user>")]
fn auth_users(user: Json<HashSet<String>>) {
    //TODO, do the real auth, RwLock, etc
    eprintln!("{:?}", user);
}

fn rocket() -> (Rocket, Option<DbConn>) {
    let rocket = rocket::ignite()
        .attach(DbConn::fairing())
        .mount(
            "/",
            routes![index, index_head, login, votes, vote, auth_users],
        )
        .mount("/", StaticFiles::from("./templates"))
        .attach(Template::fairing());

    let conn = match cfg!(test) {
        true => DbConn::get_one(&rocket),
        false => None,
    };

    (rocket, conn)
}

fn main() {
    rocket().0.launch();
}
