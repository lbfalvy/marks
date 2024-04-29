#![feature(trivial_bounds)]
#![feature(ready_into_inner)]

mod auth;
mod bearer_token;
mod boards;
mod db;
mod schema;
mod views;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use auth::cfg_auth;
use boards::cfg_boards;
use db::create_pool;
use dotenvy::dotenv;
use views::cfg_views;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();
  HttpServer::new(move || {
    App::new()
      .wrap(Logger::default())
      .app_data(web::Data::new(create_pool()))
      .configure(cfg_auth)
      .configure(cfg_views)
      .configure(cfg_boards)
      .service(hello)
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
