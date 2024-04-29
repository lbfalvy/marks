#![allow(trivial_bounds)] // diesel generated code

use std::env;

use diesel::prelude::Insertable;
use diesel::{Queryable, Selectable, SqliteConnection};

use crate::schema;

pub type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

pub fn create_pool() -> DbPool {
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  let manager = diesel::r2d2::ConnectionManager::<SqliteConnection>::new(database_url);
  (DbPool::builder().build(manager)).expect("database URL should be valid path to SQLite DB file")
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::user)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
  pub id: i64,
  pub name: String,
  pub pass_hash: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::session)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Session {
  pub token: String,
  pub user_id: i64,
  pub start: i64,
  pub refresh: i64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::board)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Board {
  pub id: i64,
  pub name: String,
  pub url: i64,
  pub version: i32,
  pub owner_id: i64,
  pub public_mut: bool,
  pub layout: String,
}
