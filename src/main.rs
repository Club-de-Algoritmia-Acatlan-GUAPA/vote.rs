#![feature(proc_macro_hygiene, decl_macro, never_type, once_cell)]

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

use std::{
    collections::{HashMap, HashSet},
    lazy::SyncLazy,
    sync::RwLock,
};

use diesel::PgConnection;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, Form, FromRequest, Request};
use rocket::Rocket;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::{json::Json, templates::Template};

use schema::{Ballot, Item, NewUser, Vote};

static AUTH: SyncLazy<RwLock<HashSet<String>>> = SyncLazy::new(|| RwLock::new(HashSet::new()));

#[database("postgresql_database")]
pub struct DbConn(PgConnection);

#[derive(Debug, Serialize)]
struct Context {
    winner: Option<Item>,
    second: Option<Item>,
    third: Option<Item>,
    items: Vec<(Item, Option<i32>)>,
    not_user: bool,
}

impl Context {
    pub fn new(conn: &DbConn) -> Context {
        Context {
            winner: Vote::run_election(conn),
            second: None,
            third: None,
            items: Vec::new(), // not used if not logged in
            not_user: false,
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
            not_user: false,
        }
    }

    pub fn error() -> Context {
        let winner = None;
        let second = None;
        let third = None;
        Context {
            winner,
            second,
            third,
            items: Vec::new(),
            not_user: true,
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
    let users = AUTH.read().unwrap();

    if !users.contains(&user.username) {
        drop(users);
        actualiza_usuarios();
        Template::render("index", Context::error())
    } else {
        drop(users);
        let u = user.login(&conn).unwrap();
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

// el bot de discord de guapa envía los usuarios
// al llegar uno nuevo (?)
#[post("/mem", format = "application/json", data = "<users>")]
fn auth_users(users: Json<HashSet<String>>) {
    let mut auth = AUTH.write().unwrap();
    auth.extend(users.into_inner());
}

fn rocket() -> (Rocket, Option<DbConn>) {
    let rocket = rocket::ignite()
        .mount("/", StaticFiles::from("./static"))
        .attach(DbConn::fairing())
        .mount(
            "/",
            routes![index, index_head, login, votes, vote, auth_users],
        )
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

fn actualiza_usuarios() {
    let client = reqwest::blocking::Client::new();
    let mut map = HashMap::new();

    map.insert("content", "!auth");
    let _ = client.post(env!("DISCORD_HOOK")).json(&map).send();
}
