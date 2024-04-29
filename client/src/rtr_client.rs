use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

use common::{clone, from_epoch_secs, TokenPair};
use gloo_console::log;
use gloo_net::http::RequestBuilder;
use gloo_storage::errors::StorageError;
use gloo_storage::{LocalStorage, Storage};
use jwt::FromBase64;
use serde::{Deserialize, Serialize};
use web_sys::js_sys::Reflect;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::BroadcastChannel;
use yew::hook;

use crate::misc_yew::{now, u32rand};
use crate::{api, boot};

#[derive(Serialize, Deserialize)]
enum RtrMsg {
  AgeReq(u32, u32),
  AgeRep(u32, u32),
}

pub fn tok_claims(tok: &str) -> HashMap<String, String> {
  let claim_str = tok.split('.').nth(1).unwrap();
  HashMap::<String, String>::from_base64(claim_str).unwrap()
}

pub fn get_token_pair() -> Option<TokenPair> {
  LocalStorage::get("tokens").unwrap_or_else(|e| match e {
    StorageError::KeyNotFound(_) => None,
    e => panic!("{e}"),
  })
}
pub fn set_token_pair(tp: Option<TokenPair>) { LocalStorage::set("tokens", tp).unwrap() }

pub async fn run_rtr<Fut: Future<Output = Option<TokenPair>>>(
  refresh_margin: Duration,
  leader_poll_delay: Duration,
  mut refresh: impl FnMut(String) -> Fut,
) {
  let bc = Rc::new(BroadcastChannel::new("rust-link").unwrap());
  let am_eldest_cache = RefCell::new(false);
  let boot_poll = Rc::new(RefCell::<Option<(u32, HashSet<u32>)>>::new(None));
  let cb: Closure<dyn Fn(JsValue) -> Result<JsValue, JsValue>> = Closure::new({
    let (bc, age_poll) = (bc.clone(), boot_poll.clone());
    move |ev: JsValue| {
      let msg = Reflect::get(&ev, &"data".to_string().into())?;
      match (serde_wasm_bindgen::from_value(msg).unwrap(), &mut *age_poll.borrow_mut()) {
        (RtrMsg::AgeRep(id, boot), Some((own_id, boots))) if *own_id == id => {
          boots.insert(boot);
        },
        (RtrMsg::AgeReq(id, _), _) => {
          let rep = serde_wasm_bindgen::to_value(&RtrMsg::AgeRep(id, boot())).unwrap();
          bc.post_message(&rep)?;
        },
        _ => (),
      }
      Ok(JsValue::undefined())
    }
  });
  bc.set_onmessage(Some(cb.as_ref().unchecked_ref()));
  let poll_eldest = || async {
    if *am_eldest_cache.borrow() {
      return true;
    }
    let id = u32rand();
    let boot = boot();
    let req = serde_wasm_bindgen::to_value(&RtrMsg::AgeReq(id, boot)).unwrap();
    let prev = boot_poll.borrow_mut().replace((id, HashSet::new()));
    assert!(prev.is_none(), "Only one leader poll may be running at a time");
    bc.post_message(&req).unwrap();
    // Chrome has spartan resource limits for unfocused tabs
    yew::platform::time::sleep(Duration::from_secs(2)).await;
    let boots = boot_poll.borrow_mut().take().expect("Initialized above").1;
    let am_eldest = boots.iter().all(|x| boot < *x);
    *am_eldest_cache.borrow_mut() |= am_eldest;
    am_eldest
  };
  loop {
    match get_token_pair() {
      None => {
        log!("Not logged in");
        yew::platform::time::sleep(Duration::from_secs(5)).await
      },
      Some(TokenPair { access_token, refresh_token }) => {
        let exp = from_epoch_secs(tok_claims(&access_token).get("exp").unwrap().parse().unwrap());
        if let Ok(defer) = exp.duration_since(now() + refresh_margin) {
          log!("Waiting", JsValue::from(defer.as_secs() as u32), "seconds until renewal");
          yew::platform::time::sleep(defer).await
        }
        log!("Time to renew");
        if !poll_eldest().await {
          log!("Am not leader");
          yew::platform::time::sleep(leader_poll_delay).await;
          continue;
        }
        log!("Am leader");
        // we are leader and refresh is due
        set_token_pair(refresh(refresh_token).await)
      },
    };
  }
}

#[hook]
pub fn use_token_pair() -> Option<TokenPair> {
  let token_pair = yew::use_state_eq(get_token_pair);
  yew::use_effect_with(
    (),
    clone!(token_pair; move |()| {
      let flag = Rc::new(RefCell::new(true));
      wasm_bindgen_futures::spawn_local(clone!(flag; async move {
        while *flag.borrow_mut() {
          let tp = get_token_pair();
          let delay = Duration::from_secs(if tp.is_some() { 5 } else { 1 });
          token_pair.set(tp);
          yew::platform::time::sleep(delay).await;
        }
      }));
      move || *flag.borrow_mut() = false
    }),
  );
  token_pair.deref().clone()
}

pub fn authenticated(
  f: impl FnOnce(&str) -> RequestBuilder,
  token: Option<&str>,
  sub: &str,
) -> RequestBuilder {
  let base = f(&api(sub));
  if let Some(token) = token {
    base.header("Authorization", &format!("Bearer {token}"))
  } else {
    base
  }
}
