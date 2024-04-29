use actix_web::{get, post, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::auth::AuthdUser;
use crate::db::DbPool;

pub fn cfg_views(cfg: &mut web::ServiceConfig) {
  cfg.service(get_layout).service(post_layout).service(own_boards);
}

#[get("/layout")]
pub async fn get_layout(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
) -> actix_web::Result<impl Responder> {
  let layout: String = web::block(move || {
    use crate::schema::user::dsl::*;
    user.find(ses_u.id).select(layout).first(&mut pool.get().unwrap()).unwrap()
  })
  .await?;
  Ok(HttpResponse::Ok().body(layout))
}

#[post("/layout")]
pub async fn post_layout(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
  body: String,
) -> actix_web::Result<impl Responder> {
  web::block(move || {
    use crate::schema::user::dsl::*;
    diesel::update(user.find(ses_u.id))
      .set(layout.eq(body))
      .execute(&mut pool.get().unwrap())
      .unwrap()
  })
  .await?;
  Ok(HttpResponse::NoContent().finish())
}

#[get("/own_boards")]
pub async fn own_boards(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
) -> actix_web::Result<impl Responder> {
  let boards: Vec<i64> = web::block(move || {
    use crate::schema::board::dsl::*;
    board.filter(owner_id.eq(ses_u.id)).select(id).load(&mut pool.get().unwrap()).unwrap()
  })
  .await?;
  Ok(HttpResponse::Ok().json(boards))
}
