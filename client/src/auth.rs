use std::time::Duration;

use common::{clone, ChangePassForm, TokenPair, UserDataForm};
use gloo_net::http::{Request, Response};
use web_sys::wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::app::Routes;
use crate::not_found::{NotFound, NotFoundTyp};
use crate::rtr_client::{get_token_pair, set_token_pair, tok_claims};

#[derive(Clone, Routable, PartialEq)]
pub enum AuthRoutes {
  #[at("/auth")]
  Auth,
  #[at("/auth/change_pass")]
  ChangePass,
  #[not_found]
  #[at("/404")]
  NotFound,
}

#[function_component(Auth)]
pub fn auth() -> Html {
  eprintln!("Hello from Auth!");
  html! {
    <Switch<AuthRoutes> render={|r| match r {
      AuthRoutes::NotFound => html!{ <NotFound typ={NotFoundTyp::Route} /> },
      AuthRoutes::Auth => html!{ <Authenticate /> },
      AuthRoutes::ChangePass => html!{ <ChangePass /> },
    }} />
  }
}

fn inev2val(e: InputEvent) -> String {
  e.target_dyn_into::<HtmlInputElement>().unwrap_throw().value()
}

async fn do_log_in(navi: &Navigator, tp: TokenPair) {
  set_token_pair(Some(tp));
  yew::platform::time::sleep(Duration::from_secs(1)).await;
  navi.push(&Routes::Home);
}

async fn recv_token_pair(navi: &Navigator, rep: Response) -> Result<(), String> {
  match rep.ok() {
    true => Ok(do_log_in(&navi, rep.json::<TokenPair>().await.unwrap()).await),
    false => Err(rep.text().await.unwrap()),
  }
}

#[function_component(Authenticate)]
fn authenticate() -> Html {
  eprintln!("Hello world!");
  let err = use_state_eq(|| None);
  let name = use_state_eq(String::new);
  let pass = use_state_eq(String::new);
  let navi = use_navigator().unwrap();
  let submit = clone!(name, pass, err, navi; move |ep| {
    clone!(name, pass, err, navi; wasm_bindgen_futures::spawn_local(async move {
      let input_form = UserDataForm { name: name.to_string(), pass: pass.to_string() };
      let rep = Request::post(&api(ep))
        .json(&input_form)
        .unwrap()
        .send()
        .await
        .unwrap();
      recv_token_pair(&navi, rep).await.unwrap_or_else(|e| err.set(Some(e)))
    }))
  });
  html! {
    <main>
      {if let Some(err) = err.as_ref() { html!{ <div>{err}</div> } } else { html!{} }}
      <label>
        <div>{"Username"}</div>
        <input type="text" value={name.to_string()}
          oninput={clone!(name; move |v| name.set(inev2val(v)))} />
      </label>
      <label>
        <div>{"Password"}</div>
        <input type="password" value={pass.to_string()}
          oninput={clone!(pass; move |v| pass.set(inev2val(v)))} />
      </label>
      <div>
        <button onclick={clone!(submit; move |_| submit("auth/login"))}>{"Login"}</button>
        <button onclick={clone!(submit; move |_| submit("auth/register"))}>{"Register"}</button>
      </div>
    </main>
  }
}

#[function_component(ChangePass)]
fn change_pass() -> Html {
  let err = use_state_eq(|| None);
  let name = use_state_eq(|| {
    get_token_pair()
      .map_or_else(String::new, |tp| tok_claims(&tp.access_token).get("name").unwrap().clone())
  });
  let pass = use_state_eq(String::new);
  let new_pass = use_state_eq(String::new);
  let navi = use_navigator().unwrap();
  html! {
    <main>
      {if let Some(err) = err.as_ref() { html!{ <div>{err}</div> } } else { html!{} }}
      <label>
        <div>{"Username"}</div>
        <input type="text" value={name.to_string()}
          oninput={clone!(name; move |v| name.set(inev2val(v)))} />
      </label>
      <label>
        <div>{"Old password"}</div>
        <input type="password" value={pass.to_string()}
          oninput={clone!(pass; move |v| pass.set(inev2val(v)))} />
      </label>
      <label>
        <div>{"New password"}</div>
        <input type="password" value={new_pass.to_string()}
          oninput={clone!(new_pass; move |v| new_pass.set(inev2val(v)))} />
      </label>
      <button onclick={clone!(name, pass, new_pass, err, navi; move |_| {
        clone!(name, pass, new_pass, err, navi; wasm_bindgen_futures::spawn_local(async move {
          let rep = Request::post(&api("auth/change_pass"))
            .json(&ChangePassForm {
              name: name.to_string(),
              pass: pass.to_string(),
              new_pass: new_pass.to_string()
            })
            .unwrap()
            .send()
            .await
            .unwrap();
          recv_token_pair(&navi, rep).await.unwrap_or_else(|e| err.set(Some(e)))
        }))
      })}>{"Change password"}</button>
    </main>
  }
}
