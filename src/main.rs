mod comment;
use actix_cors::Cors;
pub use comment::*;

mod database;
pub use database::Database;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::{env, sync::Mutex};
use validator::Validate;

struct AppState {
    db: Mutex<Database>,
}

#[get("/")]
async fn get_comments(data: web::Data<AppState>) -> impl Responder {
    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().into(),
    };
    HttpResponse::Ok().json(&db.get_comments().unwrap())
}

#[post("/")]
async fn post_comment(data: web::Data<AppState>, bytes: web::Bytes) -> impl Responder {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => {
            let db = match data.db.lock() {
                Ok(db) => db,
                Err(_) => return HttpResponse::InternalServerError(),
            };
            let comment: Comment = match serde_json::from_str(&text) {
                Ok(comment) => comment,
                Err(_) => return HttpResponse::BadRequest(),
            };
            if comment.validate().is_err() {
                return HttpResponse::BadRequest();
            }
            db.create_comment(&comment).unwrap();
            HttpResponse::Ok()
        }
        Err(_) => HttpResponse::BadRequest().into(),
    }
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let testing = {
        let mut testing = false;
        for argument in env::args() {
            if argument == "--testing" || argument == "-t" {
                testing = true;
                break;
            }
        }
        testing
    };
    let db = Database::new(testing).unwrap();
    let state = web::Data::new(AppState { db: Mutex::new(db) });
    HttpServer::new(move || {
        App::new()
            .service(get_comments)
            .service(post_comment)
            .app_data(state.clone())
            .wrap(if testing { Cors::permissive() } else { Cors::default() })
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
