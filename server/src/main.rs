#![feature(trivial_bounds)]
#![feature(ready_into_inner)]

mod auth;
mod bearer_token;
mod db;
mod schema;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use auth::{change_pass, login, refresh, register};
use db::create_pool;
use dotenvy::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();
  HttpServer::new(move || {
    App::new()
      .wrap(Logger::default())
      .app_data(web::Data::new(create_pool()))
      .service(hello)
      .service(login)
      .service(register)
      .service(refresh)
      .service(change_pass)
      .wrap(Cors::permissive())
  })
  .bind(("0.0.0.0", 8081))?
  .run()
  .await
}

#[get("/hello")]
async fn hello() -> impl Responder {
  eprintln!("Received request");
  HttpResponse::Ok().body("Hello, World!")
}
