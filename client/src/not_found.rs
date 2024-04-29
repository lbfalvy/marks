use yew::prelude::*;

#[derive(PartialEq)]
pub enum NotFoundTyp {
  Route,
  Board,
}
impl NotFoundTyp {
  pub fn name(&self) -> &str {
    match self {
      Self::Route => "Route",
      Self::Board => "Board",
    }
  }

  pub fn description(&self) -> &str {
    match self {
      Self::Route => "The route couldn't be resolved with the app's routing scheme",
      Self::Board => "This board doesn't exist; it might have been deleted or moved",
    }
  }
}

#[derive(PartialEq, Properties)]
pub struct NotFoundProps {
  pub typ: NotFoundTyp,
}

#[function_component(NotFound)]
pub fn not_found(props: &NotFoundProps) -> Html {
  html! {
    <main>
      <h1>{"404: "}{props.typ.name()}{" not found"}</h1>
      <p>{props.typ.description()}</p>
    </main>
  }
}
