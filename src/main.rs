mod comment;
pub use comment::*;

mod database;
pub use database::Database;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

struct AppState {
    db: Mutex<Database>
}

#[get("/")]
async fn get_comments(data: web::Data<AppState>) -> impl Responder {
    let db = &data.db.lock().unwrap();
    HttpResponse::Ok().json(&db.get_send_comments().unwrap())
}

#[post("/")]
async fn post_comment(data: web::Data<AppState>, bytes: web::Bytes) -> impl Responder {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => {
            let db = match data.db.lock() {
                Ok(db) => db,
                Err(_) => return HttpResponse::InternalServerError(),
            };
            let comment: CommentReceive = match serde_json::from_str(&text) {
                Ok(comment) => comment,
                Err(_) => return HttpResponse::BadRequest(),
            };
            db.create_comment(&comment.to_master()).unwrap();
            HttpResponse::Ok()
        },
        Err(_) => HttpResponse::BadRequest().into()
    }
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let db = Database::new().unwrap();
    let state = web::Data::new(AppState { db: Mutex::new(db) });
    HttpServer::new(move || {
        App::new()
            .service(get_comments)
            .service(post_comment)
            .app_data(state.clone())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
