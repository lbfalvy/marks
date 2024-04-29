use std::collections::HashMap;
use std::fmt;
use std::future::{ready, Ready};
use std::time::{Duration, SystemTime};

use actix_web::http::StatusCode;
use actix_web::{post, web, FromRequest, HttpResponse, Responder, ResponseError};
use common::{clone, epoch_secs, from_epoch_secs, ChangePassForm, TokenPair, UserDataForm};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::{RunQueryDsl, SqliteConnection};
use itertools::{partition, Itertools};

use crate::bearer_token::{make_token, BearerToken, TokenError};
use crate::db::{DbPool, Session, User};
use crate::schema::{session, user};

pub fn cfg_auth(cfg: &mut web::ServiceConfig) {
  cfg.service(register).service(login).service(refresh).service(change_pass);
}

pub struct AuthdUser {
  pub id: i64,
  pub name: String,
  pub claims: HashMap<String, String>,
}

#[derive(Debug)]
pub enum AuthError {
  Token(TokenError),
  NotAccess,
}
impl fmt::Display for AuthError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Token(e) => write!(f, "{e}"),
      Self::NotAccess => write!(f, "The token provided is not an access token"),
    }
  }
}
impl ResponseError for AuthError {
  fn status_code(&self) -> StatusCode { StatusCode::UNAUTHORIZED }
}

impl FromRequest for AuthdUser {
  type Error = AuthError;
  type Future = Ready<Result<Self, Self::Error>>;
  fn from_request(
    req: &actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
  ) -> Self::Future {
    ready((|| {
      let mut token =
        BearerToken::from_request(req, payload).into_inner().map_err(AuthError::Token)?;
      if !token.claims.get("ty").is_some_and(|s| *s == "access") {
        return Err(AuthError::NotAccess);
      }
      Ok(AuthdUser {
        id: token.claims.remove("user_id").unwrap().parse().unwrap(),
        name: token.claims.remove("name").unwrap(),
        claims: token.claims,
      })
    })())
  }
}

fn generate_token_pair(
  user_id: String,
  name: String,
  now: SystemTime,
  start: SystemTime,
) -> TokenPair {
  TokenPair {
    refresh_token: make_token(
      now,
      Duration::from_secs(60 * 60 * 24 * 7),
      HashMap::from([
        ("ty".to_string(), "refresh".to_string()),
        ("start".to_string(), epoch_secs(start).to_string()),
        ("user_id".to_string(), user_id.to_string()),
        ("name".to_string(), name.clone()),
      ]),
    ),
    access_token: make_token(
      now,
      Duration::from_secs(60 * 6),
      HashMap::from([
        ("ty".to_string(), "access".to_string()),
        ("user_id".to_string(), user_id.to_string()),
        ("name".to_string(), name.clone()),
      ]),
    ),
  }
}

fn start_session(conn: &mut SqliteConnection, user: &User) -> (Session, TokenPair) {
  let now = SystemTime::now();
  let tpair = generate_token_pair(user.id.to_string(), user.name.to_string(), now, now);
  let ses = Session {
    user_id: user.id.clone(),
    start: epoch_secs(now) as i64,
    token: tpair.refresh_token.clone(),
    refresh: epoch_secs(now) as i64,
  };
  diesel::insert_into(session::table).values(&ses).execute(conn).unwrap();
  (ses, tpair)
}

#[derive(Debug)]
pub enum RegisterError {
  NameTaken,
}
impl fmt::Display for RegisterError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NameTaken => write!(f, "Username already registered"),
    }
  }
}
impl ResponseError for RegisterError {
  fn status_code(&self) -> StatusCode { StatusCode::CONFLICT }
}

#[post("/auth/register")]
pub async fn register(
  pool: web::Data<DbPool>,
  form: web::Json<UserDataForm>,
) -> actix_web::Result<impl Responder> {
  let user = User {
    id: rand::random::<i64>().abs(),
    name: form.name.clone(),
    pass_hash: pwhash::bcrypt::hash(&form.pass).unwrap(),
  };
  web::block(move || {
    let mut conn = pool.get().unwrap();
    match diesel::insert_into(user::table).values(&user).execute(&mut conn) {
      Ok(_) => Ok(start_session(&mut conn, &user).1),
      Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) =>
        Err(RegisterError::NameTaken),
      Err(e) => panic!("Unexpected database error {e}"),
    }
  })
  .await?
  .map_err(actix_web::Error::from)
  .map(|tpair| HttpResponse::Ok().json(tpair))
}

#[derive(Debug)]
pub enum LoginError {
  NoUser,
  BadPass,
}
impl fmt::Display for LoginError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NoUser => write!(f, "User not found"),
      Self::BadPass => write!(f, "The password didn't match"),
    }
  }
}
impl ResponseError for LoginError {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::NoUser => StatusCode::BAD_REQUEST,
      Self::BadPass => StatusCode::CONFLICT,
    }
  }
}

async fn login_logic(
  pool: web::Data<DbPool>,
  form: UserDataForm,
) -> actix_web::Result<(User, Session, TokenPair)> {
  let user @ User { id: userid, .. } = web::block(clone!(pool, form; move || {
    use crate::schema::user::dsl::*;
    user.filter(name.eq(form.name))
      .select(User::as_select())
      .load(&mut pool.get().unwrap()).unwrap()
      .into_iter().at_most_one().unwrap()
  }))
  .await?
  .ok_or_else(|| actix_web::Error::from(LoginError::NoUser))?;
  if !pwhash::bcrypt::verify(&*form.pass, &*user.pass_hash) {
    return Err(actix_web::Error::from(LoginError::BadPass));
  }
  let mut sessions = web::block(clone!(pool; move || {
    use crate::schema::session::dsl::*;
    session.filter(user_id.eq(userid))
      .select(Session::as_select())
      .load(&mut pool.get().unwrap()).unwrap()
  }))
  .await?;
  let now = epoch_secs(SystemTime::now()) as i64;
  let split_point = partition(&mut sessions, |s| now < s.refresh);
  let dropping = sessions.drain(split_point..).map(|s| s.token).collect_vec();
  web::block(clone!(pool; move || {
    use crate::schema::session::dsl::*;
    diesel::delete(session.filter(token.eq_any(dropping)))
      .execute(&mut pool.get().unwrap()).unwrap();
  }))
  .await?;
  let (ses, tpair) =
    web::block(clone!(user; move || start_session(&mut *pool.get().unwrap(), &user))).await?;
  Ok((user, ses, tpair))
}

#[post("/auth/login")]
async fn login(
  pool: web::Data<DbPool>,
  form: web::Json<UserDataForm>,
) -> actix_web::Result<impl Responder> {
  let (_, _, token_pair) = login_logic(pool, form.0).await?;
  Ok(HttpResponse::Ok().json(token_pair))
}

#[derive(Clone, Debug)]
enum RefreshError {
  NotRefresh,
  TokenReuse,
  ForceEnd,
}
impl fmt::Display for RefreshError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NotRefresh => write!(f, "Refresh must be called with a refresh token"),
      Self::TokenReuse => write!(f, "This token has already been refreshed"),
      Self::ForceEnd => write!(f, "The session was closed externally"),
    }
  }
}
impl ResponseError for RefreshError {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::NotRefresh => StatusCode::BAD_REQUEST,
      Self::TokenReuse => StatusCode::CONFLICT,
      Self::ForceEnd => StatusCode::CONFLICT,
    }
  }
}

#[post("/auth/refresh")]
async fn refresh(
  pool: web::Data<DbPool>,
  bearer: BearerToken,
) -> actix_web::Result<impl Responder> {
  if !bearer.claims.get("ty").is_some_and(|t| &*t == "refresh") {
    return Err(actix_web::Error::from(RefreshError::NotRefresh));
  }
  let uid: i64 = bearer.claims.get("user_id").unwrap().parse().unwrap();
  let start_ts = bearer.claims.get("start").unwrap().parse::<u64>().unwrap();
  let ses = web::block(clone!(pool; move || {
    use crate::schema::session::dsl::*;
    session.filter(user_id.eq(uid).and(start.eq(start_ts as i64)))
      .select(Session::as_select())
      .load(&mut pool.get().unwrap()).unwrap()
      .into_iter().exactly_one().ok()
  }))
  .await?
  .ok_or_else(|| actix_web::Error::from(RefreshError::ForceEnd))?;
  if ses.token != bearer.token {
    return Err(actix_web::Error::from(RefreshError::TokenReuse));
  }
  let tpair = generate_token_pair(
    uid.to_string(),
    bearer.claims.get("name").unwrap().to_string(),
    SystemTime::now(),
    from_epoch_secs(start_ts),
  );
  let refresh_token = tpair.refresh_token.clone();
  web::block(move || {
    use crate::schema::session::dsl::*;
    diesel::update(session.find(ses.token))
      .set(token.eq(refresh_token))
      .execute(&mut pool.get().unwrap())
      .unwrap();
  })
  .await?;
  Ok(HttpResponse::Ok().json(tpair))
}

#[post("/auth/change_pass")]
async fn change_pass(
  pool: web::Data<DbPool>,
  form: web::Json<ChangePassForm>,
) -> actix_web::Result<impl Responder> {
  let (User { id: uid, .. }, _, tpair) =
    login_logic(pool.clone(), UserDataForm { name: form.name.clone(), pass: form.pass.clone() })
      .await?;
  let new_hash = pwhash::bcrypt::hash(&form.new_pass).unwrap();
  web::block(move || {
    use crate::schema::user::dsl::*;
    diesel::update(user.filter(id.eq(uid)))
      .set(pass_hash.eq(new_hash))
      .execute(&mut pool.get().unwrap())
      .unwrap();
  })
  .await?;
  Ok(HttpResponse::Ok().json(tpair))
}
