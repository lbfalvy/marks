use std::cell::OnceCell;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::time::{Duration, SystemTime};
use std::{env, fmt};

use actix_web::http::header::AUTHORIZATION;
use actix_web::{http, FromRequest, ResponseError};
use common::{epoch_secs, from_epoch_secs};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;

thread_local! {
  static KEY: OnceCell<Hmac<Sha256>> = OnceCell::new();
}

fn key() -> Hmac<Sha256> {
  KEY.with(|c| {
    Hmac::clone(
      c.get_or_init(|| Hmac::new_from_slice(env::var("JWT_SECRET").unwrap().as_bytes()).unwrap()),
    )
  })
}

#[derive(Debug, Clone)]
pub struct BearerToken {
  pub token: String,
  pub claims: HashMap<String, String>,
  pub exp: SystemTime,
  pub iat: SystemTime,
}
impl BearerToken {
  pub fn expired(&self) -> bool { self.exp < SystemTime::now() }
}

#[derive(Debug)]
pub enum TokenError {
  NoAuth,
  BadAuth,
  BadToken(jwt::Error),
  BadStdField,
  Expired,
}
impl fmt::Display for TokenError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NoAuth => write!(f, "Unauthenticated"),
      Self::BadAuth => write!(f, "Unrecognized authentication scheme, use Bearer"),
      Self::BadToken(e) => write!(f, "Bad token: {e}"),
      Self::BadStdField => write!(f, "A standard field was missing or had an unexpected value"),
      Self::Expired => write!(f, "Access token expired"),
    }
  }
}
impl ResponseError for TokenError {
  fn status_code(&self) -> http::StatusCode {
    match self {
      Self::Expired => http::StatusCode::from_u16(441).unwrap(), // custom expired error
      Self::NoAuth => http::StatusCode::UNAUTHORIZED,
      Self::BadAuth | Self::BadToken(_) | Self::BadStdField => http::StatusCode::BAD_REQUEST,
    }
  }
}

impl FromRequest for BearerToken {
  type Error = TokenError;
  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
    ready((|| {
      let auth = req.headers().get(AUTHORIZATION).ok_or(TokenError::NoAuth)?;
      let token = auth
        .to_str()
        .ok()
        .and_then(|s| s.trim().strip_prefix("Bearer "))
        .map(|s| s.trim())
        .ok_or(TokenError::BadAuth)?;
      let token = parse_token(token)?;
      (!token.expired()).then_some(token).ok_or(TokenError::Expired)
    })())
  }
}

pub fn parse_token(t: &str) -> Result<BearerToken, TokenError> {
  let claims: HashMap<String, String> = t.verify_with_key(&key()).map_err(TokenError::BadToken)?;
  let exp: u64 = claims.get("exp").and_then(|s| s.parse().ok()).ok_or(TokenError::BadStdField)?;
  let iat: u64 = claims.get("iat").and_then(|s| s.parse().ok()).ok_or(TokenError::BadStdField)?;
  Ok(BearerToken { token: t.into(), claims, iat: from_epoch_secs(iat), exp: from_epoch_secs(exp) })
}

pub fn make_token(iat: SystemTime, lt: Duration, mut claims: HashMap<String, String>) -> String {
  claims.insert("exp".to_string(), epoch_secs(iat + lt).to_string());
  claims.insert("iat".to_string(), epoch_secs(iat).to_string());
  claims.sign_with_key(&key()).expect("Creating JWT should not fail")
}
