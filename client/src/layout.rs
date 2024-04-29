use common::{clone, FreshBoard, NewBoardForm, TokenPair};
use gloo_console::log;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use yew::suspense::use_future;
use yew::{function_component, hook, html, use_context, use_effect_with, Html, Properties};
use yew_hooks::use_async;
use yew_router::prelude::*;

use crate::app::Routes;
use crate::board::{BoardLayout, BoardView};
use crate::rtr_client::{authenticated, tok_claims};
use crate::util::{use_local_storage_unf, UseLocalStorageUnfHandle};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewLayout {
  V1(ViewLayoutV1),
}

impl Default for ViewLayout {
  fn default() -> Self { Self::V1(ViewLayoutV1 { top: Vec::new(), groups: Vec::new() }) }
}

fn get_user_id(tok: &str) -> i64 { tok_claims(tok).get("user_id").unwrap().parse().unwrap() }

#[hook]
fn use_layout_ls() -> UseLocalStorageUnfHandle<(i64, ViewLayout)> {
  use_local_storage_unf::<(i64, ViewLayout)>("current layout".to_string())
}

#[hook]
fn use_layout(tok: &String) -> Option<ViewLayout> {
  use std::time::Duration;

  use common::clone;
  use yew_hooks::use_async;

  use crate::rtr_client::authenticated;

  let layout = use_layout_ls();
  let current_user: i64 = get_user_id(&tok);
  let load_layout = use_async::<_, (), !>(clone!(tok, layout; async move {
    loop {
      match authenticated(Request::get, Some(&tok), "layout").send().await {
        Err(e) => {
          log!(format!("{e}"));
          yew::platform::time::sleep(Duration::from_secs(4)).await;
          continue;
        },
        Ok(rep) if !rep.ok() => panic!("Unexpected error while getting layout: {rep:?}"),
        Ok(rep) => {
          let text = rep.text().await.unwrap();
          let data = match &*text {
            "" => ViewLayout::default(),
            _ => serde_json::from_str(&text).unwrap()
          };
          layout.set((get_user_id(&tok), data));
          break Ok(())
        }
      }
    }
  }));
  use_effect_with(current_user, move |_| load_layout.run());
  layout.as_ref().filter(|(id, _)| *id == current_user).map(|(_, lo)| lo.clone())
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewLayoutV1 {
  pub top: Vec<i64>,
  pub groups: Vec<(String, Vec<i64>)>,
}

#[derive(PartialEq, Clone, Properties)]
pub struct DBProps {
  pub access_token: String,
}

#[function_component(DefaultBoard)]
pub fn default_board(props: &DBProps) -> Html {
  let layout_ls = use_layout_ls();
  let layout = use_layout(&props.access_token);
  let current_user: i64 = get_user_id(&props.access_token);
  let add_home_board = use_async(clone!(props, layout, layout_ls; async move {
    let mut layout = layout.unwrap();
    let board_layout = serde_json::to_string(&BoardLayout::default()).unwrap();
    let rep = authenticated(Request::post, Some(&props.access_token), "new_board")
      .json(&NewBoardForm { layout: board_layout, name: "Home".to_string(), public_mut: true })
      .unwrap()
      .send()
      .await
      .unwrap();
    if !rep.ok() {
      panic!("{}: {}", rep.status(), rep.text().await.unwrap())
    }
    let nums = rep.json::<FreshBoard>().await.unwrap();
    match &mut layout {
      ViewLayout::V1(v1) => v1.top.push(nums.url),
    }
    let rep = authenticated(Request::post, Some(&props.access_token), "layout")
      .json(&layout).unwrap().send().await.unwrap();
    if !rep.ok() {
      panic!("Failed to add fresh board to layout; the details are {nums:?}")
    }
    layout_ls.set((current_user, layout));
    Ok::<_, !>(())
  }));
  match layout {
    None => html! { <p>{"Loading layout..."}</p> },
    Some(ViewLayout::V1(layout)) => {
      log!(format!("Routing to default board for {layout:?}"));
      match layout.top.first().or_else(|| layout.groups.iter().find_map(|g| g.1.first())) {
        Some(fst) => html! { <Redirect<Routes> to={Routes::Board{ id: fst.to_string() }} /> },
        None if add_home_board.loading => html!{ <p>{"Creating first board..."}</p> },
        None if add_home_board.data.is_some() => html!{ <p>{"Failed to commit first board!"}</p> },
        None => {
          add_home_board.run();
          html!{ <p>{"Preparing to create first board..."}</p> }
        }
      }
    },
  }
}

#[derive(PartialEq, Clone, Properties)]
pub struct LVProps {
  pub access_token: String,
  pub board_id: i64,
}

#[function_component(LayoutView)]
pub fn layout_view(props: &LVProps) -> Html {
  let tp = use_context::<TokenPair>();
  let layout = use_layout(&tp.unwrap().access_token);
  match layout.as_ref() {
    None => html! { <BoardView id={props.board_id} /> },
    Some(ViewLayout::V1(v1)) => html! {
      <LayoutViewV1 layout={v1.clone()} board_id={props.board_id} />
    },
  }
}

#[derive(PartialEq, Clone, Properties)]
struct LVV1Props {
  pub layout: ViewLayoutV1,
  pub board_id: i64,
}

#[function_component(LayoutViewV1)]
fn layout_view_v1(props: &LVV1Props) -> Html {
  html! {
    <>
      <div>{"hello from the default layout"}</div>
      <BoardView id={props.board_id} />
    </>
  }
}
