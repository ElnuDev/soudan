mod comment;
pub use comment::Comment;

mod database;
pub use database::Database;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::sync::{LockResult, Mutex};

#[get("/comments")]
async fn get_comments(data: web::Data<AppState>) -> impl Responder {
    let db = &data.db.lock().unwrap();
    /*db.create_comment(&Comment {
        author: Some("Elnu".to_string()),
        text: "Hello world".to_string()
    }).unwrap();*/
    HttpResponse::Ok().body(format!("{}", db.get_comments().unwrap().get(0).unwrap_or(&Comment { author: None, text: "No comments yet!".to_string() }).text))
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let state = web::Data::new(AppState { db: Mutex::new(Database::new().unwrap()) });
    HttpServer::new(move || {
        App::new()
            .service(get_comments)
            .app_data(state.clone())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
