#![feature(async_closure)]
#![feature(never_type)]

mod about;
mod app;
mod auth;
mod board;
mod layout;
mod misc_yew;
mod not_found;
mod rtr_client;
mod util;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use app::App;
use gloo_console::log;
use gloo_net::http::Request;
use misc_yew::now;
use rtr_client::run_rtr;

use crate::rtr_client::authenticated;
use crate::util::retry;

static mut BOOT: Option<SystemTime> = None;
/// Get the time when the `main` function was called
pub fn boot() -> u32 {
  // SAFETY: BOOT is never mutated after its initialization in `main()`
  let boot = unsafe { BOOT.as_ref() };
  boot.unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

pub fn api(sub: &str) -> String { format!("http://localhost:8081/{sub}") }

pub fn spawn_rtr() {
  log!("Starting RTR...");
  wasm_bindgen_futures::spawn_local(run_rtr(
    Duration::from_secs(5 * 60),
    Duration::from_secs(10),
    |refresh| async move {
      let rep = retry(Duration::from_secs(5), || async {
        authenticated(Request::post, Some(&refresh), "auth/refresh")
      })
      .await;
      if [409, 441].contains(&rep.status()) {
        None // session expired or invalidated due to token reuse
      } else if rep.ok() {
        Some(rep.json().await.unwrap()) // new token pair
      } else {
        panic!("{rep:?}") // unrecognized error condition
      }
    },
  ))
}

fn main() {
  console_error_panic_hook::set_once();
  // SAFETY: this is the only thread at this moment
  unsafe { BOOT = Some(now()) }
  spawn_rtr();
  yew::Renderer::<App>::new().render();
}
