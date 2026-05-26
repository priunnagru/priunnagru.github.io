use dioxus::prelude::*;
use crate::api::ApiClient;
use crate::types::GameResponse;
use crate::Route;

#[component]
pub fn CustomGame() -> Element {
    let navigator = use_navigator();
    let mut game_state = use_context::<Signal<Option<GameResponse>>>();
    let error = use_signal(|| Option::<String>::None);
    let loading = use_signal(|| false);
    let result = use_signal(|| Option::<Result<GameResponse, String>>::None);

    let mut start_id = use_signal(|| String::new());
    let mut end_id = use_signal(|| String::new());

    rsx! {
        div { class: "custom-game-container",
            h1 { "🎮 Custom Game" }
            p { "Enter two anime IDs (from AniList) to find a path between them" }

            div { class: "input-group",
                label { r#for: "start-id", "Start Anime ID" }
                input {
                    id: "start-id",
                    r#type: "number",
                    value: start_id(),
                    placeholder: "e.g., 1 (Cowboy Bebop)",
                    oninput: move |e| *start_id.write() = e.value(),
                }
            }

            div { class: "input-group",
                label { r#for: "end-id", "Target Anime ID" }
                input {
                    id: "end-id",
                    r#type: "number",
                    value: end_id(),
                    placeholder: "e.g., 5114 (LoGH)",
                    oninput: move |e| *end_id.write() = e.value(),
                }
            }

            if let Some(ref err) = error() {
                div { class: "error-box", "{err}" }
            }

            if let Some(Err(ref e)) = result() {
                div { class: "error-box", "{e}" }
            }

            button {
                class: "btn btn-primary",
                disabled: loading() || start_id().is_empty() || end_id().is_empty(),
                onclick: move |_| {
                    let start: i32 = start_id().parse().unwrap_or(0);
                    let end: i32 = end_id().parse().unwrap_or(0);
                    if start > 0 && end > 0 && start != end {
                        let mut game_state = game_state.clone();
                        let mut result_sig = result.clone();
                        let mut loading_sig = loading.clone();
                        let navigator = navigator.clone();
                        spawn(async move {
                            loading_sig.set(true);
                            let client = ApiClient::new();
                            match client.get_custom_game(start, end).await {
                                Ok(response) => {
                                    result_sig.set(None);
                                    game_state.set(Some(response));
                                    loading_sig.set(false);
                                    navigator.push(Route::GamePage {});
                                }
                                Err(e) => {
                                    result_sig
                                        .set(
                                            Some(
                                                Err(
                                                    format!(
                                                        "Could not create game: {}. No path exists between these anime.",
                                                        e,
                                                    ),
                                                ),
                                            ),
                                        );
                                    loading_sig.set(false);
                                }
                            }
                        });
                    }
                },
                if loading() {
                    "Starting..."
                } else {
                    "Start Game"
                }
            }

            div { class: "popular-anime",
                h3 { "Popular Anime IDs" }
                div { class: "anime-id-list",
                    div { class: "anime-id-item", "1 - Cowboy Bebop" }
                    div { class: "anime-id-item", "5114 - Fullmetal Alchemist: Brotherhood" }
                    div { class: "anime-id-item", "16498 - Attack on Titan" }
                    div { class: "anime-id-item", "11061 - Hunter x Hunter (2011)" }
                    div { class: "anime-id-item", "20 - Naruto" }
                    div { class: "anime-id-item", "21 - One Piece" }
                }
            }

            a { href: "/", class: "btn btn-back", "← Back to Home" }
        }
    }
}
