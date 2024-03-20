use common::{clone, TokenPair};
use yew::prelude::*;
use yew_router::prelude::*;

use super::about::About;
use super::not_found::{NotFound, NotFoundTyp};
use crate::auth::{Auth, AuthRoutes};
use crate::rtr_client::use_token_pair;

#[derive(Clone, Routable, PartialEq)]
pub enum Routes {
  #[at("/")]
  Home,
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
    <HashRouter>
      <Switch<Routes> render={clone!(token_pair; move |r: Routes| match r {
        Routes::NotFound => html!{ <NotFound typ={NotFoundTyp::Route} /> },
        Routes::About => html!{ <About /> },
        Routes::Home => match &token_pair {
          None => html!{ <Redirect<Routes> to={Routes::About} /> },
          Some(tokens) => html!{
            <ContextProvider<TokenPair> context={tokens.clone()} >
              <AppView />
            </ContextProvider<TokenPair>>
          }
        },
        Routes::Auth | Routes::AuthRoot => html!{ <Auth /> }
      })} />
      <footer>
        <Link<Routes> to={Routes::Home}>{"cURL Marks"}</Link<Routes>>
        {if token_pair.is_none() {html!{
          <Link<Routes> to={Routes::AuthRoot}>{"Log in"}</Link<Routes>>
        }} else {html!{
          <Link<AuthRoutes> to={AuthRoutes::ChangePass}>{"Change Pass"}</Link<AuthRoutes>>
        }}}
      </footer>
    </HashRouter>
  }
}

#[function_component(AppView)]
pub fn app_view() -> Html {
  let _token = yew::use_context::<TokenPair>().unwrap().access_token;
  html! { <main> {"Hello!"}</main> }
}
