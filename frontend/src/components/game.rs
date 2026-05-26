use dioxus::prelude::*;
use crate::api::ApiClient;
use crate::types::Anime;
use super::win_screen::WinScreen;
use super::anime_card::AnimeCard;

/// Renders a single recommendation card.
/// When clicked, appends the anime to the player's path.
/// If the selected anime matches the target (`end`), triggers win verification
/// by calling `/game/win` so the server returns `min_steps` and shortest paths.
#[component]
pub fn RecsCard(
    anime: Anime,
    end: Anime,
    token: String,
    path: Signal<Vec<Anime>>,
    verifying: Signal<bool>,
    min_steps: Signal<usize>,
    shortest_paths: Signal<Vec<Vec<Anime>>>,
    won: Signal<bool>,
    error: Signal<Option<String>>,
) -> Element {
    let anime_clone = anime.clone();
    let end_clone = end.clone();
    let mut path_clone = path.clone();
    let verifying_clone = verifying.clone();
    let min_steps_clone = min_steps.clone();
    let shortest_paths_clone = shortest_paths.clone();
    let won_clone = won.clone();
    let error_clone = error.clone();
    // Clone token here so onclick closure doesn't capture the owned String
    let token_for_click = token.clone();
    rsx! {
        AnimeCard {
            anime: anime_clone.clone(),
            clickable: true,
            onclick: move |_| {
                // Don't allow clicks while verifying
                if verifying_clone() {
                    return;
                }
                let mut path_write = path_clone.write();
                path_write.push(anime_clone.clone());
                let is_target = anime_clone.id == end_clone.id;
                drop(path_write);
                if is_target {
                    let mut min_steps_sig = min_steps_clone.clone();
                    let mut shortest_paths_sig = shortest_paths_clone.clone();
                    let mut won_sig = won_clone.clone();
                    let mut error_sig = error_clone.clone();
                    let mut verifying_sig = verifying_clone.clone();
                    let path_sig = path_clone.clone();
                    let token_inner = token_for_click.clone();
                    verifying_sig.set(true);
                    spawn(async move {
                        let path_ids: Vec<i32> = path_sig.read().iter().map(|a| a.id).collect();
                        match ApiClient::new().verify_win(&token_inner, &path_ids).await {
                            Ok(response) if response.is_valid => {
                                min_steps_sig.set(response.min_steps);
                                shortest_paths_sig.set(response.shortest_paths);
                                won_sig.set(true);
                            }
                            Err(e) => {
                                error_sig.set(Some(format!("Failed to verify win: {}", e)));
                            }
                            _ => {
                                error_sig
                                    .set(Some("Invalid path - win not confirmed".to_string()));
                            }
                        }
                        verifying_sig.set(false);
                    });
                }
            },
        }
    }
}

#[component]
pub fn Game(token: String, start: Anime, end: Anime) -> Element {
    let path = use_signal(|| vec![start.clone()]);
    let mut error = use_signal(|| Option::<String>::None);
    let verifying = use_signal(|| false);
    let won = use_signal(|| false);
    let min_steps = use_signal(|| 0);
    let shortest_paths = use_signal(|| Vec::<Vec<Anime>>::new());

    // Clone token before it's moved into use_resource
    let token_recs = token.clone();

    // Load recommendations based on current path
    let recs = use_resource(move || {
        let token = token_recs.clone();
        let path = path();
        let path_ids: Vec<i32> = path.iter().map(|a| a.id).collect();
        async move {
            let client = ApiClient::new();
            match client.get_recs(&token, &path_ids).await {
                Ok(response) => {
                    let path_ids_set: std::collections::HashSet<i32> = path_ids.into_iter().collect();
                    response.recs.into_iter()
                        .filter(|a| !path_ids_set.contains(&a.id))
                        .collect()
                }
                Err(e) => {
                    error.set(Some(e));
                    Vec::new()
                }
            }
        }
    });

    // Show win screen if won
    if won() {
        return rsx! {
            WinScreen {
                start: start.clone(),
                end: end.clone(),
                user_path: path.read().clone(),
                user_steps: path.read().len() - 1,
                min_steps: min_steps(),
                shortest_paths: shortest_paths.read().clone(),
            }
        };
    }

    let path_count = path.read().len();

    rsx! {
        div { class: "game-container",
            // Header with start -> end
            div { class: "game-header",
                AnimeMiniCard { anime: start.clone(), label: "Start" }
                span { class: "arrow", "⟶" }
                AnimeMiniCard { anime: end.clone(), label: "Target" }
            }

            // Path display
            div { class: "path-display",
                for (i , anime) in path.read().iter().enumerate() {
                    div { class: "path-node",
                        span { class: "path-number", "{i + 1}" }
                        span { class: "path-name",
                            {anime.title_english.as_deref().unwrap_or(&anime.title_romaji).to_string()}
                        }
                    }
                    if i < path_count - 1 {
                        span { class: "path-arrow", "→" }
                    }
                }
            }

            // Error display
            if let Some(ref err) = error() {
                div { class: "error-box", "{err}" }
            }

            // Verifying overlay
            if verifying() {
                div { class: "verifying-overlay", "Verifying win..." }
            }

            // Recommendations grid
            div { class: "recs-grid",
                match recs() {
                    Some(ref rec_list) => rsx! {
                        for anime in rec_list.iter().cloned() {
                            RecsCard {
                                anime: anime.clone(),
                                end: end.clone(),
                                token: token.clone(),
                                path: path.clone(),
                                verifying: verifying.clone(),
                                min_steps: min_steps.clone(),
                                shortest_paths: shortest_paths.clone(),
                                won: won.clone(),
                                error: error.clone(),
                            }
                        }
                    },
                    None => rsx! {
                        div { class: "loading", "Loading recommendations..." }
                    },
                }
            }
        }
    }
}

#[component]
pub fn AnimeMiniCard(anime: Anime, label: &'static str) -> Element {
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
