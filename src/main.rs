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
use sanitize_html::{sanitize_str, rules::predefined::DEFAULT, errors::SanitizeError};

struct AppState {
    databases: HashMap<String, Mutex<Database>>,
}

fn get_db<'a>(
    data: &'a web::Data<AppState>,
    request: &HttpRequest,
) -> Result<MutexGuard<'a, Database>, HttpResponse> {
    let origin = match request.head().headers().get("Origin") {
        Some(origin) => match origin.to_str() {
            Ok(origin) => origin,
            Err(_) => return Err(HttpResponse::BadRequest().reason("bad origin").finish()),
        },
        None => return Err(HttpResponse::BadRequest().reason("bad origin").finish()),
    };
    match data.databases.get(origin) {
        Some(database) => Ok(match database.lock() {
            Ok(database) => database,
            Err(_) => return Err(HttpResponse::InternalServerError().reason("database error").finish()),
        }),
        None => return Err(HttpResponse::BadRequest().reason("bad origin").finish()),
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
            let PostCommentsRequest { url, comment } = match serde_json::from_str::<PostCommentsRequest>(&text) {
                Ok(mut req) => {
                    let mut sanitize_req = || -> Result<(), SanitizeError> {
                        req.comment.text = sanitize_str(&DEFAULT, &req.comment.text)?
                            .replace("&gt;", ">"); // required for markdown quotes
                        if let Some(ref mut author) = req.comment.author {
                            *author = sanitize_str(&DEFAULT, &author)?;
                        }
                        Ok(())
                    };
                    if let Err(_) = sanitize_req() {
                        return HttpResponse::InternalServerError().reason("failed to sanitize request").finish();
                    }
                    req
                }
                Err(_) => return HttpResponse::BadRequest().reason("invalid request body").finish(),
            };
            if comment.validate().is_err() {
                return HttpResponse::BadRequest().reason("invalid comment field(s)").finish();
            }
            let origin = match request.head().headers().get("Origin") {
                Some(origin) => match origin.to_str() {
                    Ok(origin) => origin,
                    // If the Origin is not valid ASCII, it is a bad request not sent from a browser
                    Err(_) => return HttpResponse::BadRequest().reason("bad origin").finish(),
                },
                // If there is no Origin header, it is a bad request not sent from a browser
                None => return HttpResponse::BadRequest().reason("bad origin").finish(),
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
                return HttpResponse::BadRequest().reason("url out of scope").finish();
            }
            match get_page_data(&url).await {
                Ok(page_data_option) => match page_data_option {
                    Some(page_data) => {
                        if page_data.content_id != comment.content_id {
                            return HttpResponse::BadRequest().reason("content ids don't match").finish();
                        }
                    }
                    None => return HttpResponse::BadRequest().reason("url invalid").finish(), // e.g. 404
                },
                Err(_) => return HttpResponse::InternalServerError().reason("failed to get page data").finish(),
            };
            let database = match get_db(&data, &request) {
                Ok(database) => database,
                Err(response) => return response,
            };
            if let Some(parent) = comment.parent {
                'outer2: loop {
                    match database.get_comments(&comment.content_id) {
                        Ok(comments) => for other_comment in comments.iter() {
                            if other_comment.id.unwrap() == parent {
                                if other_comment.parent.is_none() {
                                    break 'outer2;
                                }
                                break;
                            }
                        },
                        Err(_) => return HttpResponse::InternalServerError().reason("failed to get comments").finish(),
                    }
                    return HttpResponse::BadRequest().reason("invalid comment parent").finish();
                }
            }
            if let Err(_) = database.create_comment(&comment) {
                return HttpResponse::InternalServerError().reason("failed to create comment").finish();
            }
            HttpResponse::Ok().into()
        }
        Err(_) => HttpResponse::BadRequest().reason("failed to parse request body").finish(),
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
            Mutex::new(Database::new(testing, domain).unwrap()),
        );
    }
    let state = web::Data::new(AppState { databases });
    HttpServer::new(move || {
        App::new()
            .service(get_comments)
            .service(post_comment)
            .app_data(state.clone())
            // Issue with CORS on POST requests,
            // keeping permissive for now
            .wrap(Cors::permissive() /* if testing {
                Cors::permissive()
            } else {
                let mut cors = Cors::default()
                   .allowed_methods(vec!["GET", "POST"]);
                for domain in domains.iter() {
                    cors = cors.allowed_origin(domain);
                }
                cors
            } */)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
