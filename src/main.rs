mod comment;
use actix_cors::Cors;
pub use comment::*;

mod database;
pub use database::Database;

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::{
    env,
    sync::{Mutex, MutexGuard},
};
use validator::Validate;

struct AppState {
    databases: HashMap<String, Mutex<Database>>,
}

fn get_db<'a>(
    data: &'a web::Data<AppState>,
    request: &HttpRequest,
) -> Result<MutexGuard<'a, Database>, HttpResponse> {
    // all the .into() are converting from HttpResponseBuilder to HttpResponse
    let origin = match request.head().headers().get("Origin") {
        Some(origin) => match origin.to_str() {
            Ok(origin) => origin,
            Err(_) => return Err(HttpResponse::BadRequest().into()),
        },
        None => return Err(HttpResponse::BadRequest().into()),
    };
    match data.databases.get(origin) {
        Some(database) => Ok(match database.lock() {
            Ok(database) => database,
            Err(_) => return Err(HttpResponse::InternalServerError().into()),
        }),
        None => return Err(HttpResponse::BadRequest().into()),
    }
}

#[get("/{content_id}")]
async fn get_comments(
    data: web::Data<AppState>,
    request: HttpRequest,
    content_id: web::Path<String>,
) -> impl Responder {
    let database = match get_db(&data, &request) {
        Ok(database) => database,
        Err(response) => return response,
    };
    HttpResponse::Ok().json(database.get_comments(&content_id).unwrap())
}

#[derive(Deserialize)]
struct PostCommentsRequest {
    url: String,
    comment: Comment,
}

#[post("/")]
async fn post_comment(
    data: web::Data<AppState>,
    request: HttpRequest,
    bytes: web::Bytes,
) -> impl Responder {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => {
            let PostCommentsRequest { url, comment } = match serde_json::from_str(&text) {
                Ok(req) => req,
                Err(_) => return HttpResponse::BadRequest().into(),
            };
            if comment.validate().is_err() {
                return HttpResponse::BadRequest().into();
            }
            let origin = match request.head().headers().get("Origin") {
                Some(origin) => match origin.to_str() {
                    Ok(origin) => origin,
                    // If the Origin is not valid ASCII, it is a bad request not sent from a browser
                    Err(_) => return HttpResponse::BadRequest().into(),
                },
                // If there is no Origin header, it is a bad request not sent from a browser
                None => return HttpResponse::BadRequest().into(),
            };
            // Check to see if provided URL is in scope.
            // This is to prevent malicious requests that try to get server to fetch external websites.
            // (requires loop because "labels on blocks are unstable")
            // https://github.com/rust-lang/rust/issues/48594
            'outer: loop {
                for site_root in data.databases.keys() {
                    if site_root.starts_with(origin) && url.starts_with(site_root) {
                        break 'outer;
                    }
                }
                return HttpResponse::BadRequest().into();
            }
            match get_page_data(&url).await {
                Ok(page_data_option) => match page_data_option {
                    Some(page_data) => {
                        if page_data.content_id != comment.content_id {
                            return HttpResponse::BadRequest().into();
                        }
                    }
                    None => return HttpResponse::BadRequest().into(),
                },
                Err(_) => return HttpResponse::InternalServerError().into(),
            };
            let database = match get_db(&data, &request) {
                Ok(database) => database,
                Err(response) => return response,
            };
            database.create_comment(&comment).unwrap();
            HttpResponse::Ok().into()
        }
        Err(_) => HttpResponse::BadRequest().into(),
    }
}

// Contains all page details stored in meta tags.
// Currently, only content_id, but this is wrapped in this struct
// to make adding other meta tags, such as locked comments, in the future
struct PageData {
    content_id: String,
}

async fn get_page_data(url: &str) -> Result<Option<PageData>, reqwest::Error> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let content = response.text_with_charset("utf-8").await?;
    let document = Html::parse_document(&content);
    let get_meta = |name: &str| -> Option<String> {
        let selector = Selector::parse(&format!("meta[name=\"{}\"]", name)).unwrap();
        match document.select(&selector).next() {
            Some(element) => match element.value().attr("content") {
                Some(value) => Some(value.to_owned()),
                None => return None,
            },
            None => return None,
        }
    };
    return Ok(Some(PageData {
        content_id: match get_meta("soudan-content-id") {
            Some(id) => id,
            None => return Ok(None),
        },
    }));
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let mut domains = Vec::new();
    let testing = {
        let mut testing = false;
        let mut args = env::args();
        args.next(); // Skip first, will be executable name
        for argument in args {
            if argument == "--testing" || argument == "-t" {
                testing = true;
            } else {
                domains.push(argument);
            }
        }
        testing
    };
    if domains.len() == 0 {
        panic!("At least one domain is required!");
    }
    let mut databases = HashMap::new();
    for domain in domains.iter() {
        databases.insert(
            domain.to_owned(),
            Mutex::new(Database::new(testing).unwrap()),
        );
    }
    let state = web::Data::new(AppState { databases });
    HttpServer::new(move || {
        App::new()
            .service(get_comments)
            .service(post_comment)
            .app_data(state.clone())
            .wrap(if testing {
                Cors::permissive()
            } else {
                Cors::default()
            })
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
