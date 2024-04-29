use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::future::Future;
use std::time::Duration;

use common::clone;
use gloo_console::log;
use gloo_net::http::{RequestBuilder, Response};
use gloo_storage::{LocalStorage, Storage as _};
use gloo_utils::errors::JsError;
use gloo_utils::window;
use web_sys::wasm_bindgen::UnwrapThrowExt;
use web_sys::StorageEvent;
use yew::{hook, use_memo, use_mut_ref};
use serde::{Deserialize, Serialize};
use yew_hooks::{use_event_with_window, use_mut_latest, UseMutLatestHandle};

pub async fn retry<Fut: Future<Output = RequestBuilder>>(
  timeout: Duration,
  mut req: impl FnMut() -> Fut,
) -> Response {
  loop {
    match req().await.send().await {
      // network error or some such
      Err(gloo_net::Error::JsError(JsError { name, message, .. })) => {
        log!(name, ": ", message);
        yew::platform::time::sleep(timeout).await;
      },
      Err(e) => panic!("{e}"),
      Ok(rep) => break rep,
    }
  }
}

pub struct UseLocalStorageUnfHandle<T> {
  inner: Option<T>,
  latest: Rc<RefCell<Option<T>>>,
  key: UseMutLatestHandle<String>,
}
impl<T> UseLocalStorageUnfHandle<T> {
  /// Set a `value` for the specified key.
  pub fn set(&self, value: T) where T: Serialize + Clone {
    let cur_key = self.key.current();
    let cur_key = cur_key.as_ref().borrow();
    if LocalStorage::set(&*cur_key, value.clone()).is_ok() {
      let steve = StorageEvent::new("storage").unwrap_throw();
      steve.init_storage_event_with_can_bubble_and_cancelable_and_key("storage", false, false, Some(&*cur_key));
      window().dispatch_event(&steve).unwrap_throw();
    }
  }
  pub fn delete(&self) {
    LocalStorage::delete(&*self.key.current().as_ref().borrow())
  }
  /// Obtain a reference to the latest value. If the handle is copied out of the original component
  /// which then rerenders, the enclosed value might get stale, but this function locks the internal
  /// backing store of the hook so its value is guaranteed to always be fresh
  /// 
  /// This is a lock guard that conflicts a Yew hook. Yielding while holding it may cause a crash.
  pub fn latest(&self) -> impl Deref<Target = Option<T>> + '_ {
    self.latest.as_ref().try_borrow().unwrap()
  }
}
impl<T: Clone> Clone for UseLocalStorageUnfHandle<T> {
  fn clone(&self) -> Self {
      Self { inner: self.inner.clone(), latest: self.latest.clone(), key: self.key.clone() }
  }
}
impl<T> Deref for UseLocalStorageUnfHandle<T> {
  type Target = Option<T>;
  fn deref(&self) -> &Self::Target {&self.inner }
}

#[hook]
pub fn use_local_storage_unf<T: for<'de> Deserialize<'de> + Clone + 'static>(
  key: String
) -> UseLocalStorageUnfHandle<T> {
  use gloo_storage::{LocalStorage, Storage as _};
  use web_sys::StorageEvent;
  use yew_hooks::use_update;

  let latest = use_mut_ref(|| LocalStorage::get::<Option<T>>(&key).unwrap_or_default());
  let update = use_update();
  let _: Rc<()> = use_memo(key.clone(), |k| {
    let val = LocalStorage::get::<Option<T>>(&k).unwrap_or_default();
    *latest.as_ref().try_borrow_mut().unwrap() = val
  });
  let key = use_mut_latest(key);
  use_event_with_window("storage", clone!(key, latest, update; move |e: StorageEvent| {
    let cur_key = key.current();
    let key_g = cur_key.as_ref().borrow();
    if let Some(k) = e.key().filter(|k| *k == *key_g) {
      let val = LocalStorage::get::<Option<T>>(&k).unwrap_or_default();
      *latest.try_borrow_mut().unwrap() = val;
      update();
    }
  }));
  let wtf = latest.as_ref().try_borrow().unwrap();
  UseLocalStorageUnfHandle { inner: (*wtf).clone(), latest: latest.clone(), key }
}