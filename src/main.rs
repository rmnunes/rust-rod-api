use actix_web::{error::ErrorNotFound, web, App, Error, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

#[derive(Serialize)]
struct CreateUserResponse {
    id: u32,
    name: String,
}

type UserDb = Arc<Mutex<HashMap<u32, User>>>;

#[actix_web::get("/ping")]
async fn greet() -> impl Responder {
    format!("pong")
}

#[actix_web::get("/{id}")]
async fn get_id(id: web::Path<u32>, db: web::Data<UserDb>) -> Result<impl Responder, Error> {
    let user_id = id.into_inner();
    let db = db.lock().unwrap();

    match db.get(&user_id) {
        Some(user_data) => Ok(HttpResponse::Ok().json(user_data)),
        None => Err(ErrorNotFound("User not found")),
    }
}

#[actix_web::post("/")]
async fn create_user(user_data: web::Json<User>, db: web::Data<UserDb>) -> impl Responder {
    let mut db = db.lock().unwrap();
    let new_id = db.keys().max().unwrap_or(&0) + 1;
    let name = user_data.name.clone();
    db.insert(new_id, user_data.into_inner());

    HttpResponse::Created().json(CreateUserResponse { id: new_id, name })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {port}");

    let user_db: UserDb = Arc::new(Mutex::new(HashMap::<u32, User>::new()));

    HttpServer::new(move || {
        let app_data = web::Data::new(user_db.clone());
        App::new()
            .app_data(app_data)
            .service(greet)
            .service(get_id)
            .service(create_user)
    })
    .bind(("localhost", port))?
    .workers(2)
    .run()
    .await
}