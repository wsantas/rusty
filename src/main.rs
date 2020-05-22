#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;

mod task;
mod user;
#[cfg(test)] mod tests;

use rocket::{Rocket, Response, Request, response};
use rocket::fairing::AdHoc;
use rocket::request::{Form, FlashMessage};
use rocket::response::{Flash, Redirect, status, Responder};
use rocket_contrib::{templates::Template, serve::StaticFiles, json::Json, json::JsonValue};
use rocket::http::{ContentType, Status};
use diesel::SqliteConnection;

use task::{Task, Todo};
use user::{User, UserForm};
use std::env;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
embed_migrations!();

#[database("sqlite_database")]
pub struct DbConn(SqliteConnection);

#[derive(Debug, Serialize)]
struct Context<'a, 'b>{ msg: Option<(&'a str, &'b str)>, tasks: Vec<Task> }

impl<'a, 'b> Context<'a, 'b> {
    pub fn err(conn: &DbConn, msg: &'a str) -> Context<'static, 'a> {
        Context{msg: Some(("error", msg)), tasks: Task::all(conn)}
    }

    pub fn raw(conn: &DbConn, msg: Option<(&'a str, &'b str)>) -> Context<'a, 'b> {
        Context{msg: msg, tasks: Task::all(conn)}
    }
}

#[derive(Debug)]
struct ApiResponse {
    json: JsonValue,
    status: Status,
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[post("/", data = "<todo_form>")]
fn new(todo_form: Form<Todo>, conn: DbConn) -> Flash<Redirect> {
    let todo = todo_form.into_inner();
    if todo.description.is_empty() {
        Flash::error(Redirect::to("/"), "Description cannot be empty.")
    } else if Task::insert(todo, &conn) {
        Flash::success(Redirect::to("/"), "Todo successfully added.")
    } else {
        Flash::error(Redirect::to("/"), "Whoops! The server failed.")
    }
}

#[put("/<id>")]
fn toggle(id: i32, conn: DbConn) -> Result<Redirect, Template> {
    if Task::toggle_with_id(id, &conn) {
        Ok(Redirect::to("/"))
    } else {
        Err(Template::render("index", &Context::err(&conn, "Couldn't toggle task.")))
    }
}

#[delete("/<id>")]
fn delete(id: i32, conn: DbConn) -> Result<Flash<Redirect>, Template> {
    if Task::delete_with_id(id, &conn) {
        Ok(Flash::success(Redirect::to("/"), "Todo was deleted."))
    } else {
        Err(Template::render("index", &Context::err(&conn, "Couldn't delete task.")))
    }
}

#[get("/")]
fn index(msg: Option<FlashMessage>, conn: DbConn) -> Template {
    Template::render("index", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}

#[get("/")]
fn hello(msg: Option<FlashMessage>, conn: DbConn) -> Template {
    Template::render("hello", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}

#[get("/")]
fn gpl(msg: Option<FlashMessage>, conn: DbConn) -> Template {
    Template::render("gpl", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}

#[post("/", data = "<user_form>", format = "json")]
fn tokensignin(user_form: Json<UserForm>,  conn: DbConn) -> ApiResponse {
    format!("Success: {}", user_form.access_token);
    if User::insertOrUpdate(user_form.into_inner(), &conn) {
        ApiResponse {
            json: json!({"status": "success"}),
            status: Status::Ok,
        }
    } else {
        ApiResponse {
            json: json!({"status": "failed"}),
            status: Status::Ok,
        }
    }
}




#[get("/<email>")]
fn fetch_inbox_top(email: String, msg: Option<FlashMessage>, conn: DbConn) -> Template {

    let users = User::all(&conn);
    let user = users.get(0).unwrap();

    let at = &user.access_token.to_owned();

    Template::render("hello", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}


fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = DbConn::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

fn rocket() -> Rocket {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .mount("/", StaticFiles::from("static/"))
        .mount("/", routes![index])
        .mount("/todo", routes![new, toggle, delete])
        .mount("/fetch_inbox_top", routes![fetch_inbox_top])
        .mount("/gpl", routes![gpl])
        .mount("/tokensignin", routes![tokensignin])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
