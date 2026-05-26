use dioxus::prelude::*;
use crate::types::Anime;
use crate::Route;

#[component]
pub fn WinScreen(start: Anime, end: Anime, user_path: Vec<Anime>, user_steps: usize, min_steps: usize, shortest_paths: Vec<Vec<Anime>>) -> Element {
    let navigator = use_navigator();
    let rating = match (user_steps, min_steps) {
        (u, m) if u == m => "⭐⭐⭐",
        (u, m) if u == m + 1 => "⭐⭐",
        (u, m) if u <= m + 2 => "⭐",
        _ => "🔄",
    };

    rsx! {
        div { class: "win-screen",
            h1 { "You found the path!" }
            div { class: "win-rating", "{rating}" }
            div { class: "win-stats",
                p { "Your path: {user_steps} steps" }
                p { "Shortest path: {min_steps} steps" }
            }
            div { class: "win-anime-pair",
                AnimeMiniCard { anime: start.clone(), label: "Start" }
                span { class: "arrow", "⟶" }
                AnimeMiniCard { anime: end.clone(), label: "Target" }
            }
            div { class: "paths-section",
                div { class: "user-path-item",
                    p { class: "path-label", "Your Solution ({user_steps} steps)" }
                    div { class: "path-anime-list",
                        for (j , anime) in user_path.iter().enumerate() {
                            span { class: "path-anime-name",
                                {anime.title_english.as_deref().unwrap_or(&anime.title_romaji)}
                            }
                            if j < user_path.len() - 1 {
                                span { class: "path-arrow", "→" }
                            }
                        }
                    }
                }
                if !shortest_paths.is_empty() {
                    h2 { "🏆 Shortest Path Solutions" }
                    for (i , solution) in shortest_paths.iter().enumerate() {
                        div { class: "shortest-path-item",
                            p { class: "path-label", "Solution {i + 1} ({min_steps} steps)" }
                            div { class: "path-anime-list",
                                for (j , anime) in solution.iter().enumerate() {
                                    span { class: "path-anime-name",
                                        {anime.title_english.as_deref().unwrap_or(&anime.title_romaji)}
                                    }
                                    if j < solution.len() - 1 {
                                        span { class: "path-arrow", "→" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "win-actions",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        navigator.push(Route::Home {});
                    },
                    "New Game"
                }
            }
        }
    }
}

#[component]
fn AnimeMiniCard(anime: Anime, label: &'static str) -> Element {
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
