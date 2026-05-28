use dioxus::prelude::*;
use crate::types::Anime;

#[component]
pub fn AnimeCard(anime: Anime, clickable: bool, onclick: Callback<()>) -> Element {
    let card_size = use_context::<Signal<usize>>();
    let title = anime.title_english.as_deref().unwrap_or(&anime.title_romaji).to_string();
    let title_clone = title.clone();
    let title_romaji = anime.title_romaji.clone();

    let img_height = (280_f64 * card_size() as f64 / 100.0) as usize;

    rsx! {
        div {
            class: format!("anime-card{}", if clickable { " clickable" } else { "" }),
            "data-tooltip": &title_clone,
            onclick: move |_| {
                if clickable {
                    onclick.call(())
                }
            },
            img {
                class: "anime-image",
                style: format!("height: {}px;", img_height),
                src: &anime.image_url,
                alt: &title_clone,
                loading: "lazy",
            }
            div { class: "anime-info",
                h3 { class: "anime-title", {title} }
                p { class: "anime-romaji", {title_romaji} }
            }
        }
    }
}
