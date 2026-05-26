use dioxus::prelude::*;
use crate::api::ApiClient;
use crate::types::GameResponse;
use crate::Route;

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let mut game_state = use_context::<Signal<Option<GameResponse>>>();

    let daily_game = use_resource(|| {
        let client = ApiClient::new();
        async move { client.get_daily_game().await }
    });

    rsx! {
        div { class: "home-container",
            h1 { class: "game-title", "ani-rec-dle" }
            p { class: "game-subtitle",
                "Find the shortest path between two anime through recommendations"
            }

            div { class: "home-sections",
                div { class: "section-card daily-section",
                    h2 { "🎯 Daily Challenge" }
                    match daily_game() {
                        Some(Ok(ref game)) => rsx! {
                            div { class: "daily-info",
                                div { class: "anime-pair",
                                    AnimeMiniCard { anime: game.start.clone(), label: "Start" }
                                    span { class: "arrow", "⟶" }
                                    AnimeMiniCard { anime: game.end.clone(), label: "Target" }
                                }
                                button {
                                    class: "btn btn-primary",
                                    onclick: move |_| {
                                        if let Some(Ok(ref game)) = daily_game() {
                                            game_state.set(Some(game.clone()));
                                            {
                                                navigator.push(Route::GamePage {});
                                            }
                                        }
                                    },
                                    "Start Daily Game"
                                }
                            }
                        },
                        Some(Err(ref e)) => rsx! {
                            p { class: "error", "Failed to load daily: {e}" }
                        },
                        None => rsx! {
                            p { "Loading daily challenge..." }
                        },
                    }
                }

                // Custom Game Section
                div { class: "section-card custom-section",
                    h2 { "🎮 Custom Game" }
                    p { "Pick any two anime and find the path between them" }
                    a { href: "/game/custom", class: "btn btn-secondary", "Create Custom Game" }
                }

                // How To Play Section
                div { class: "section-card help-section",
                    h2 { "📖 How to Play" }
                    ol {
                        li { "You are given a start and target anime" }
                        li { "Each anime has recommendations (similar anime)" }
                        li { "Click recommendations to build a path from start to target" }
                        li { "Find the shortest path possible!" }
                    }
                }
            }
        }
    }
}

#[component]
fn AnimeMiniCard(anime: crate::types::Anime, label: &'static str) -> Element {
    let title = anime.title_english.as_deref().unwrap_or(&anime.title_romaji);
    
    rsx! {
        div { class: "anime-mini-card",
            img {
                class: "anime-mini-image",
                src: &anime.image_url,
                alt: title,
            }
            div { class: "anime-mini-info",
                span { class: "anime-mini-label", "{label}" }
                span { class: "anime-mini-title", "{title}" }
            }
        }
    }
}
