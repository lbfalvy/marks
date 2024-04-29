use common::{clone, TokenPair};
use gloo_console::log;
use yew::prelude::*;
use yew_router::prelude::*;

use super::about::About;
use super::not_found::{NotFound, NotFoundTyp};
use crate::auth::{Auth, AuthRoutes};
use crate::board::BoardView;
use crate::layout::{DefaultBoard, LayoutView};
use crate::rtr_client::use_token_pair;

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Routes {
  #[at("/")]
  Home,
  #[at("/board/:id")]
  Board { id: String },
  #[at("/foo/123/:id")]
  FooBarBaz{ id: i64 },
  #[at("/about")]
  About,
  #[not_found]
  #[at("/404")]
  NotFound,
  #[at("/auth")]
  AuthRoot,
  #[at("/auth/*")]
  Auth,
}

#[function_component(App)]
pub fn app() -> Html {
  let token_pair = use_token_pair();
  html! {
    <ContextProvider<Option<TokenPair>> context={token_pair.clone()}>
      <HashRouter>
        <Switch<Routes> render={clone!(token_pair; move |r: Routes| {
          log!(format!("Routed to {r:?}"));match (r, &token_pair) {
          (Routes::NotFound, _) => html!{ <NotFound typ={NotFoundTyp::Route} /> },
          (Routes::FooBarBaz{ id }, _) => html!{ <p>{"FooBarBaz"}{id}</p> },
          (Routes::About, _) => html!{ <About /> },
          (Routes::Board{ id }, None) => html!{
            <main><BoardView id={id.parse::<i64>().unwrap()} /></main>
          },
          (Routes::Board{ id }, Some(tokens@TokenPair{ access_token: access, .. })) => html!{
            <ContextProvider<TokenPair> context={tokens.clone()}>
              <LayoutView access_token={access.clone()} board_id={id.parse::<i64>().unwrap()} />
            </ContextProvider<TokenPair>>
          },
          (Routes::Home, None) => html!{ <Redirect<Routes> to={Routes::About} /> },
          (Routes::Home, Some(tokens)) => html!{
            <DefaultBoard access_token={tokens.access_token.clone()} />
          },
          (Routes::Auth | Routes::AuthRoot, _) => html!{ <Auth /> },
        }})} />
        <footer>
          <Link<Routes> to={Routes::Home}>{"cURL Marks"}</Link<Routes>>
          {if token_pair.is_none() {html!{
            <Link<Routes> to={Routes::AuthRoot}>{"Log in"}</Link<Routes>>
          }} else {html!{
            <Link<AuthRoutes> to={AuthRoutes::ChangePass}>{"Change Pass"}</Link<AuthRoutes>>
          }}}
        </footer>
      </HashRouter>
    </ContextProvider<Option<TokenPair>>>
  }
}
