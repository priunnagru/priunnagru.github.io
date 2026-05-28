use dioxus::prelude::*;
use crate::types::GameResponse;
use crate::components::home::Home;
use crate::components::game::{Game, CardSizeProvider};
use crate::components::custom_game::CustomGame;

mod types;
mod api;
mod components;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(Layout)]
        #[route("/")]
        Home {},
        #[route("/game")]
        GamePage {},
        #[route("/game/custom")]
        CustomGame {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Layout() -> Element {
    use_context_provider(|| Signal::new(Option::<GameResponse>::None));
    
    rsx! {
        body {
            header { class: "app-header",
                a { href: "/", class: "app-logo", "ani-rec-dle" }
                nav { class: "app-nav",
                    a { href: "/", "Say Hi" }
                }
            }
            main { class: "app-main", Outlet::<Route> {} }
            footer { class: "app-footer",
                p { "© 2026 ani-rec-dle. All rights reserved. JK this is not copyrighted lol" }
            }
        }
    }
}

// Bridge component that retrieves game state from context and renders the game
#[component]
fn GamePage() -> Element {
    let game_state = use_context::<Signal<Option<GameResponse>>>();
    let game = game_state.read().clone();
    
    match game {
        Some(game) => {
            rsx! {
                CardSizeProvider {
                    Game {
                        token: game.token.clone(),
                        start: game.start.clone(),
                        end: game.end.clone(),
                        is_daily: game.is_daily,
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "game-not-found",
                    h1 { "No game found" }
                    p { "Please start a game from the home page or create a custom game." }
                    a { href: "/", class: "btn btn-primary", "Go Home" }
                }
            }
        }
    }
}
