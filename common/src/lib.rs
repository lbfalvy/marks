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
