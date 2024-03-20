use yew::prelude::*;

#[derive(PartialEq)]
pub enum NotFoundTyp {
  Route,
}
impl NotFoundTyp {
  pub fn name(&self) -> &str {
    match self {
      Self::Route => "Route",
    }
  }

  pub fn description(&self) -> &str {
    match self {
      Self::Route => "The route couldn't be resolved with the app's routing scheme",
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
