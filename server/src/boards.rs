use std::fmt;

use actix_web::http::header::{EntityTag, IfMatch, IfNoneMatch};
use actix_web::http::StatusCode;
use actix_web::{delete, get, post, web, HttpResponse, Responder, ResponseError};
use common::{BoardDetails, BoardPatch, FreshBoard, NewBoardForm};
use diesel::prelude::*;
use itertools::Itertools;

use crate::auth::AuthdUser;
use crate::db::{Board, DbPool};

pub fn cfg_boards(cfg: &mut web::ServiceConfig) {
  cfg
    .service(del_board)
    .service(manage_board)
    .service(edit_board)
    .service(new_board)
    .service(move_board)
    .service(get_board_layout)
    .service(get_board);
}

#[derive(Clone, Debug)]
pub struct BoardNotFound {
  must_own: bool,
}
impl fmt::Display for BoardNotFound {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.must_own {
      true => write!(f, "Board moved, deleted, or not owned by this user"),
      false => write!(f, "Board moved or deleted"),
    }
  }
}
impl ResponseError for BoardNotFound {
  fn status_code(&self) -> StatusCode { StatusCode::NOT_FOUND }
}

#[delete("/boards/{id}")]
async fn del_board(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
  target_board: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
  let count = web::block(move || {
    use crate::schema::board::dsl::*;
    diesel::delete(board.filter(owner_id.eq(ses_u.id).and(url.eq(*target_board))))
      .execute(&mut pool.get().unwrap())
      .unwrap()
  })
  .await?;
  (0 < count)
    .then(|| HttpResponse::NoContent().finish())
    .ok_or_else(|| BoardNotFound { must_own: true }.into())
}

#[post("/boards/{id}")]
async fn manage_board(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
  target_board: web::Path<i64>,
  patch: web::Json<BoardPatch>,
) -> actix_web::Result<impl Responder> {
  let count = web::block(move || {
    use crate::schema::board::dsl::*;
    diesel::update(board.filter(url.eq(*target_board).and(owner_id.eq(ses_u.id))))
      .set((
        patch.name.as_ref().map(|n| name.eq(n.clone())),
        patch.owner_id.map(|uid| owner_id.eq(uid)),
        patch.public_mut.map(|pmut| public_mut.eq(pmut)),
        version.eq(version + 1),
      ))
      .execute(&mut pool.get().unwrap())
      .unwrap()
  })
  .await?;
  (0 < count)
    .then(|| HttpResponse::NoContent().finish())
    .ok_or_else(|| BoardNotFound { must_own: true }.into())
}

#[derive(Clone, Debug)]
pub struct MalformedEntityTag;
impl fmt::Display for MalformedEntityTag {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "A correct board tag is a positive integer version number")
  }
}
impl ResponseError for MalformedEntityTag {
  fn status_code(&self) -> actix_web::http::StatusCode { StatusCode::BAD_REQUEST }
}
fn parse_etags<'a>(v: impl IntoIterator<Item = &'a EntityTag>) -> actix_web::Result<Vec<i32>> {
  (v.into_iter())
    .map(|x| x.tag().parse::<i32>().map_err(|_| actix_web::Error::from(MalformedEntityTag)))
    .collect()
}

#[post("/boards/{id}/layout")]
async fn edit_board(
  pool: web::Data<DbPool>,
  ses_u: Option<AuthdUser>,
  target_board: web::Path<i64>,
  new_layout: String,
  ifmatch: web::Header<IfMatch>,
) -> actix_web::Result<impl Responder> {
  let tags = match &*ifmatch {
    IfMatch::Items(itv) => Some(parse_etags(&itv[..])?),
    IfMatch::Any => None,
  };
  let count = web::block(move || {
    use crate::schema::board::dsl::*;
    let update = diesel::update(board.filter(url.eq(*target_board))).into_boxed();
    let mut update = match ses_u {
      Some(u) => update.filter(public_mut.eq(true).or(owner_id.eq(u.id))),
      None => update.filter(public_mut.eq(true)),
    };
    if let Some(tags) = tags {
      update = update.filter(version.eq_any(tags));
    }
    update
      .set((layout.eq(new_layout), version.eq(version + 1)))
      .execute(&mut pool.get().unwrap())
      .unwrap()
  })
  .await?;
  (0 < count)
    .then(|| HttpResponse::NoContent().finish())
    .ok_or_else(|| BoardNotFound { must_own: true }.into())
}

#[post("/new_board")]
async fn new_board(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
  form: web::Json<NewBoardForm>,
) -> actix_web::Result<impl Responder> {
  let NewBoardForm { layout, name, public_mut } = form.clone();
  let [id, url]: [i64; 2] = rand::random::<[u32; 2]>().map(i64::from);
  let new_board = Board { id, name, url, public_mut, layout, owner_id: ses_u.id, version: 0 };
  web::block(move || {
    use crate::schema::board::dsl::*;
    diesel::insert_into(board).values(new_board).execute(&mut pool.get().unwrap()).unwrap();
  })
  .await?;
  Ok(HttpResponse::Ok().json(FreshBoard { id, url }))
}

#[post("/boards/{id}/move")]
async fn move_board(
  pool: web::Data<DbPool>,
  ses_u: AuthdUser,
  target_board: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
  let new_url: i64 = rand::random::<u32>().into();
  let count = web::block(move || {
    use crate::schema::board::dsl::*;
    diesel::update(board.filter(url.eq(*target_board).and(owner_id.eq(ses_u.id))))
      .set(url.eq(new_url))
      .execute(&mut pool.get().unwrap())
      .unwrap()
  })
  .await?;
  (0 < count)
    .then(|| HttpResponse::Ok().body(new_url.to_string()))
    .ok_or_else(|| BoardNotFound { must_own: true }.into())
}

#[get("/boards/{id}")]
async fn get_board(
  pool: web::Data<DbPool>,
  target_board: web::Path<i64>,
  ifnmatch: web::Header<IfNoneMatch>,
) -> actix_web::Result<impl Responder> {
  let known = match &*ifnmatch {
    IfNoneMatch::Items(itv) => parse_etags(&itv[..])?,
    IfNoneMatch::Any => Vec::new(),
  };
  let board: Option<Board> = web::block(move || {
    use crate::schema::board::dsl::*;
    (board.filter(url.eq(*target_board)))
      .select(Board::as_select())
      .load(&mut pool.get().unwrap())
      .unwrap()
      .into_iter()
      .exactly_one()
      .ok()
  })
  .await?;
  let board = board.ok_or(BoardNotFound { must_own: false })?;
  if known.iter().any(|v| board.version == *v) {
    return Ok(HttpResponse::NotModified().finish());
  }
  Ok(HttpResponse::Ok().json(BoardDetails {
    id: board.id,
    name: board.name,
    version: board.version,
    owner_id: board.owner_id,
    public_mut: board.public_mut,
  }))
}

#[get("/boards/{id}/layout")]
async fn get_board_layout(
  pool: web::Data<DbPool>,
  target_board: web::Path<i64>,
  ifnmatch: web::Header<IfNoneMatch>,
) -> actix_web::Result<impl Responder> {
  let known = match &*ifnmatch {
    IfNoneMatch::Items(itv) => parse_etags(&itv[..])?,
    IfNoneMatch::Any => Vec::new(),
  };
  let board: Option<Board> = web::block(move || {
    use crate::schema::board::dsl::*;
    (board.filter(url.eq(*target_board)))
      .select(Board::as_select())
      .load(&mut pool.get().unwrap())
      .unwrap()
      .into_iter()
      .exactly_one()
      .ok()
  })
  .await?;
  let board = board.ok_or(BoardNotFound { must_own: false })?;
  if known.iter().any(|v| board.version == *v) {
    return Ok(HttpResponse::NotModified().finish());
  }
  Ok(HttpResponse::Ok().body(board.layout))
}
