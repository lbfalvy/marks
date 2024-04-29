use crate::not_found::NotFound;
use std::future::ready;

use common::BoardDetails;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use yew::{function_component, hook, html, use_effect_with, use_state, Html, Properties};

use crate::api;
use crate::not_found::NotFoundTyp;
use crate::util::{retry, use_local_storage_unf};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardLayout {
  V1(BoardLayoutV1),
}
impl Default for BoardLayout {
  fn default() -> Self {
      Self::V1(BoardLayoutV1{ sections: Vec::new() })
  }
}

#[derive(Debug, Clone, Copy, Deserialize, Hash, PartialEq, Eq, Serialize)]
pub struct BoardNotFound;

#[hook]
pub fn use_board_layout(id: i64) -> Option<Result<BoardLayout, BoardNotFound>> {
  use std::time::Duration;

  use common::clone;
  use yew_hooks::use_async;

  let board_not_found = use_state(|| false);
  let board_layout = use_local_storage_unf::<BoardLayout>(format!("layout of board {id}"));
  let board_meta = use_local_storage_unf::<BoardDetails>(format!("meatdata of board {id}"));
  let load_layout = use_async::<_, (), !>(clone!(board_layout, board_not_found; async move {
    let has_layout = board_layout.is_some();
    let rep = retry(Duration::from_secs(4), clone!(board_meta; move || {
      let mut req = Request::get(&api(&format!("board/{id}/layout")));
      if let (Some(meta), true) = (board_meta.as_ref(), has_layout) {
        req = req.header("If-None-Match", &format!("\"{}\"", meta.version));
      }
      ready(req)
    })).await;
    if rep.status() == 304 {
      // not modified, use cached
    } else if rep.ok() {
      board_layout.set(rep.json().await.unwrap());
      // if there was a new layout, we know that there's a new version so we load that too.
      // This persists the unchanged shortcut on future layout fetches, but isn't required for
      // functionality so it's not a problem if a rerender slips between the two
      let layout_req = || ready(Request::get(&api(&format!("board/{id}/layout"))));
      board_meta.set(retry(Duration::from_secs(4), layout_req).await.json().await.unwrap());
    } else if rep.status() == 404 {
      board_not_found.set(true);
      board_layout.delete();
      board_meta.delete();
    }
    Ok(())
  }));
  use_effect_with(id, move |_| load_layout.run());
  if *board_not_found { Some(Err(BoardNotFound)) } else { board_layout.as_ref().cloned().map(Ok) }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardLayoutV1 {
  sections: Vec<(String, Vec<Section>)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
  title: String,
  items: Vec<SectionItem>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionItem {
  url: String,
  name: String,
}

#[derive(PartialEq, Clone, Properties)]
pub struct BVProps {
  pub id: i64,
}

#[function_component(BoardView)]
pub fn board_view(props: &BVProps) -> Html {
  let board = use_board_layout(props.id);
  match board {
    None => html!{ "Loading board..."},
    Some(Err(BoardNotFound)) => html!{ <NotFound typ={NotFoundTyp::Board} /> },
    Some(Ok(BoardLayout::V1(v1))) => html!{ <BoardViewV1 layout={v1.clone()} /> }
  }
}

#[derive(PartialEq, Clone, Properties)]
pub struct BVV1Props {
  pub layout: BoardLayoutV1,
}

#[function_component(BoardViewV1)]
pub fn board_view_v1(props: &BVV1Props) -> Html {
  html! {
    <div>{"hello"}</div>
  }
}
