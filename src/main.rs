mod comment;
pub use comment::Comment;

mod database;
pub use database::Database;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

struct AppState {
    db: Mutex<Database>
}

#[get("/comments")]
async fn get_comments(data: web::Data<AppState>) -> impl Responder {
    let db = &data.db.lock().unwrap();
    HttpResponse::Ok().json(&db.get_comments().unwrap())
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let db = Database::new().unwrap();
    db.create_comment(&Comment {
        author: None,
        text: "This is anonymous test comment!".to_string(),
    }).unwrap();
    let state = web::Data::new(AppState { db: Mutex::new(db) });
    HttpServer::new(move || {
        App::new()
            .service(get_comments)
            .app_data(state.clone())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
