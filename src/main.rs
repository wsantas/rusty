#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;

mod task;
mod user;
mod nlp;
#[cfg(test)] mod tests;

use rocket::{Rocket, Response, Request, response, Config};
use rocket::fairing::AdHoc;
use rocket::request::{Form, FlashMessage};
use rocket::response::{Flash, Redirect, Responder};
use rocket_contrib::{templates::Template, serve::StaticFiles, json::Json, json::JsonValue};
use rocket::http::{ContentType, Status};
use diesel::SqliteConnection;

use task::{Task, Todo};
use user::{User, UserForm};
extern crate imap;
extern crate native_tls;
extern crate rusoto_core;
extern crate json;
extern crate rusoto_comprehend;

use json::object;

use native_tls::TlsConnector;
use nlp::{EmailSentimentForm, detect_key_phrases};
use rocket::config::Environment;

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
fn gpl(msg: Option<FlashMessage>, conn: DbConn) -> Template {
    Template::render("gpl", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}

#[get("/")]
fn email_sentiment_form(msg: Option<FlashMessage>, conn: DbConn) -> Template {
    Template::render("email_sentiment_form", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}

#[post("/", data = "<user_form>", format = "json")]
fn tokensignin(user_form: Json<UserForm>,  conn: DbConn) -> ApiResponse {
    format!("Success: {}", user_form.access_token);
    if User::insert_or_update(user_form.into_inner(), &conn) {
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

struct GmailOAuth2 {
    user: String,
    access_token: String,
}

impl imap::Authenticator for GmailOAuth2 {
    type Response = String;
    #[allow(unused_variables)]
    fn process(&self, data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}
type ID = usize;
#[derive(Serialize, Deserialize)]
struct Message {
    id: Option<ID>,
    contents: String
}

struct SentimentScoreCalc {
    positive: f32,
    negative: f32,
    neutral: f32,
    mixed: f32
}


#[post("/<email>", data = "<email_sentiment_form>", format = "json")]
fn fetch_message_analysis(email: String, email_sentiment_form: Json<EmailSentimentForm>, conn: DbConn) -> Json<Message> {
    let form = email_sentiment_form.into_inner();
    let users = User::all(&conn);
    let user = users.get(0).unwrap();
    let at = &user.access_token;

    let gmail_auth = GmailOAuth2 {
        user: String::from(&email),
        access_token: String::from(at)
    };
    let domain = "imap.gmail.com";
    let port = 993;
    let socket_addr = (domain, port);
    let ssl_connector = TlsConnector::builder().build().unwrap();
    let client = imap::connect(socket_addr, domain, &ssl_connector).unwrap();

    let mut imap_session = match client.authenticate("XOAUTH2", &gmail_auth) {
        Ok(c) => c,
        Err((e, _unauth_client)) => panic!("{}", e)
    };

    match imap_session.select("INBOX") {
        Ok(mailbox) => println!("{}", mailbox),
        Err(e) => println!("Error selecting INBOX: {}", e),
    };

    let messages = imap_session.fetch(form.message_id.clone(), "RFC822");
    let message = messages.iter().next().unwrap();
    // extract the message's body
    let body = message.get(0).unwrap().body().expect("message did not have a body!");
    let mut body = std::str::from_utf8(body)
        .expect("message was not valid utf-8")
        .to_string();
    let full_body = body.clone();
    imap_session.logout().unwrap();

    let mut sentiment_score_results = Vec::new();
    let mut sentiment_score_options = Vec::new();
    if body.len() < 5000 {
        sentiment_score_options.push(Some(nlp::check_sentiment(body.clone()).sentiment_score.unwrap()));
    } else {
        while body.len() > 5000 {
            sentiment_score_options.push(Some(nlp::check_sentiment(body[..5000].parse().unwrap()).sentiment_score.unwrap()));
            body = body[5000..].parse().unwrap();
        }
        sentiment_score_options.push(Some(nlp::check_sentiment(body.parse().unwrap()).sentiment_score.unwrap()));
    }

    for x in &sentiment_score_options {
        if x.is_some() {
            let sentiment = x.as_ref().unwrap();
            println!("Positive Score: {}", sentiment.positive.unwrap());
            println!("Negative Score: {}", sentiment.negative.unwrap());
            println!("Mixed Score: {}", sentiment.mixed.unwrap());
            println!("Neutral Score: {}", sentiment.neutral.unwrap());

            let data = SentimentScoreCalc {
                positive: sentiment.positive.unwrap(),
                negative: sentiment.negative.unwrap(),
                mixed: sentiment.mixed.unwrap(),
                neutral: sentiment.neutral.unwrap()
            };
            sentiment_score_results.push(data)
        }
    }

    // let key_phrases_result = detect_key_phrases(full_body.clone());
    // let mut key_phrases = Vec::new();
    // for key_phrase in key_phrases_result {
    //     key_phrases.push(Some(key_phrase));
    // }

    if sentiment_score_results.len() > 0 {
        let mut positive_score = 0.0;
        let mut negative_score = 0.0;
        let mut mixed_score = 0.0;
        let mut neutral_score = 0.0;
        for x in &sentiment_score_results {
            positive_score += x.positive;
            negative_score += x.negative;
            mixed_score += x.mixed;
            neutral_score += x.neutral;
        }


        let data = object! {
            sentiment_pos: positive_score,
            sentiment_neg: negative_score,
            sentiment_mix: mixed_score,
            sentiment_neu: neutral_score,
            // key_phrases: key_phrases,
            body: full_body
        };

        Json(Message {
            id: Some(form.message_id.parse().unwrap()),
            contents: data.to_string()
        })
    }else {
        Json(Message {
            id: Some(form.message_id.parse().unwrap()),
            contents: "".parse().unwrap()
        })
    }
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
    let config = Config::build(Environment::Staging)
        .address("127.0.0.1")
        .port(8000)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .mount("/", StaticFiles::from("static/"))
        .mount("/", routes![index])
        .mount("/todo", routes![new, toggle, delete])
        .mount("/fetch_message_analysis", routes![fetch_message_analysis])
        .mount("/gpl", routes![gpl])
        .mount("/email_sentiment_form", routes![email_sentiment_form])
        .mount("/tokensignin", routes![tokensignin])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
