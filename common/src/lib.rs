use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserDataForm {
  pub name: String,
  pub pass: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChangePassForm {
  pub name: String,
  pub pass: String,
  pub new_pass: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
  pub access_token: String,
  pub refresh_token: String,
}

pub fn epoch_secs(st: SystemTime) -> u64 { st.duration_since(UNIX_EPOCH).unwrap().as_secs() }
pub fn from_epoch_secs(secs: u64) -> SystemTime { UNIX_EPOCH + Duration::from_secs(secs) }

#[macro_export]
macro_rules! clone {
  ($($n:ident),+; $body:expr) => (
    {
      $( let $n = $n.clone(); )+
      $body
    }
  );
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardDetails {
  pub id: i64,
  pub name: String,
  pub version: i32,
  pub owner_id: i64,
  pub public_mut: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardPatch {
  pub name: Option<String>,
  pub public_mut: Option<bool>,
  pub owner_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBoardForm {
  pub name: String,
  pub public_mut: bool,
  pub layout: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreshBoard {
  pub id: i64,
  pub url: i64,
}
