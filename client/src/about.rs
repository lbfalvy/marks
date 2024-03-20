pub use yew::prelude::*;

#[function_component(About)]
pub fn about() -> Html {
  html! {
    <main>
      <h1>{"Welcome to Marks!"}</h1>
      <p>{"Marks is a lightweight bookmark manager created by me (lbfalvy) for my personal use.
      It is inspired by "}<a href="https://booky.io">{"Booky"}</a>{" which was my previous
      choice and basically a perfect bookmark manager. Marks is in essence a variant of booky
      which I can set as my browser startpage. It loads as fast as possible, it incorporates
      a web search bar and some other utilities, and I can theme it to match Firefox. I use it
      in conjunction with
      "}<a href="https://addons.mozilla.org/firefox/addon/tree-style-tab/">{"Tree Style Tab"}</a>{"
      and a "}<a href="https://github.com/lbfalvy/userchrome">{"custom usercss"}</a>{", but it is
      intended to be usable on its own."}</p>
    </main>
  }
}
