use dioxus::prelude::*;
use crate::api::ApiClient;
use crate::types::{ApiError, Anime, SolutionData};
use super::win_screen::WinScreen;
use super::anime_card::AnimeCard;

// ─── Recommendation Card ──────────────────────────────────────────────

/// Simple card that delegates click handling up to the Game component.
#[component]
pub fn RecsCard(anime: Anime, clickable: bool, on_click: Callback<Anime>) -> Element {
    let anime_clone = anime.clone();
    rsx! {
        AnimeCard {
            anime: anime_clone.clone(),
            clickable,
            onclick: move |_| {
                if clickable {
                    on_click.call(anime_clone.clone());
                }
            },
        }
    }
}

// ─── Local Storage Keys ─────────────────────────────────────────────

const CARD_SIZE_KEY: &str = "ani_recdle_card_size";
const DAILY_DATE_KEY: &str = "ani_recdle_daily_date";
const DAILY_ATTEMPTS_KEY: &str = "ani_recdle_daily_attempts";
const DAILY_REVEALED_KEY: &str = "ani_recdle_daily_revealed";

fn get_stored_card_size() -> usize {
    #[cfg(feature = "web")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(val)) = storage.get_item(CARD_SIZE_KEY) {
                    if let Ok(size) = val.parse::<usize>() {
                        if size >= 50 && size <= 150 {
                            return size;
                        }
                    }
                }
            }
        }
    }
    100
}

fn set_stored_card_size(size: usize) {
    #[cfg(feature = "web")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(CARD_SIZE_KEY, &size.to_string());
            }
        }
    }
}

// ─── Attempt Tracking ─────────────────────────────────────────────────

fn today_date_str() -> String {
    // Only called inside #[cfg(feature = "web")] blocks
    #[cfg(feature = "web")]
    {
        use js_sys::Date;
        use web_sys::wasm_bindgen::JsValue;
        let ms = Date::now(); // milliseconds since epoch
        let date = Date::new(&JsValue::from_f64(ms));
        let y = date.get_utc_full_year() as usize;
        let m = date.get_utc_month() as usize + 1;
        let d = date.get_utc_date() as usize;
        format!("{:04}-{:02}-{:02}", y, m, d)
    }
    #[cfg(not(feature = "web"))]
    {
        "1970-01-01".to_string()
    }
}

/// Returns (first_attempt, best_attempt, min_steps, revealed_solution)
fn load_attempts() -> (Option<usize>, Option<usize>, Option<usize>, bool) {
    #[cfg(feature = "web")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(stored_date)) = storage.get_item(DAILY_DATE_KEY) {
                    if stored_date == today_date_str() {
                        if let Ok(Some(json_str)) = storage.get_item(DAILY_ATTEMPTS_KEY) {
                            if let Ok(json) =
                                serde_json::from_str::<serde_json::Value>(&json_str)
                            {
                                let first =
                                    json.get("first").and_then(|v| v.as_u64()).map(|x| x as usize);
                                let best =
                                    json.get("best").and_then(|v| v.as_u64()).map(|x| x as usize);
                                let min_steps =
                                    json.get("min_steps").and_then(|v| v.as_u64()).map(|x| x as usize);
                                let revealed = storage.get_item(DAILY_REVEALED_KEY)
                                    .ok()
                                    .flatten()
                                    .is_some();
                                return (first, best, min_steps, revealed);
                            }
                        }
                    }
                }
            }
        }
    }
    (None, None, None, false)
}

fn save_attempts(
    first: Option<usize>,
    best: Option<usize>,
    min_steps: Option<usize>,
    revealed: bool,
) {
    #[cfg(feature = "web")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let json = serde_json::json!({
                    "first": first,
                    "best": best,
                    "min_steps": min_steps,
                });
                let _ = storage.set_item(DAILY_DATE_KEY, &today_date_str());
                let _ = storage.set_item(DAILY_ATTEMPTS_KEY, &json.to_string());
                if revealed {
                    let _ = storage.set_item(DAILY_REVEALED_KEY, "1");
                }
            }
        }
    }
}

// ─── Card Size UI ─────────────────────────────────────────────────────
#[component]
pub fn CardSizeProvider(children: Element) -> Element {
    let mut card_size = use_signal(|| 100usize);
    use_context_provider(|| card_size.clone());

    // Load stored value after hydration to avoid SSR mismatch
    use_effect(move || {
        let stored = get_stored_card_size();
        if stored != 100 {
            card_size.set(stored);
        }
    });

    rsx! {
        {children}
    }
}

#[component]
pub fn CardSizeSlider() -> Element {
    let mut card_size = use_context::<Signal<usize>>();
    rsx! {
        div { class: "card-size-slider",
            label { "Card size: " }
            input {
                r#type: "range",
                min: "50",
                max: "150",
                value: card_size().to_string(),
                oninput: move |e| {
                    if let Ok(val) = e.value().parse::<usize>() {
                        card_size.set(val);
                        set_stored_card_size(val);
                    }
                },
            }
            span { class: "slider-value", "{card_size()}%" }
        }
    }
}

// ─── Game Component ───────────────────────────────────────────────────

#[component]
pub fn Game(token: String, start: Anime, end: Anime, is_daily: bool) -> Element {
    let card_size = use_context::<Signal<usize>>();

    // Core state
    let path = use_signal(|| vec![start.clone()]);
    let mut error = use_signal(|| Option::<String>::None);
    let verifying = use_signal(|| false);
    let won = use_signal(|| false);
    let min_steps = use_signal(|| 0usize);

    // Win / solution flow
    let game_type = use_signal(|| String::new());
    // Daily challenge state: only load from localStorage for daily games
    let (stored_first, stored_best, stored_min_steps, stored_revealed) = if is_daily {
        load_attempts()
    } else {
        (None, None, None, false)
    };
    let first_attempt = use_signal(|| stored_first);
    let best_attempt = use_signal(|| stored_best);
    let solution_data = use_signal(|| Option::<SolutionData>::None);
    let revealing_solution = use_signal(|| false);
    let conflict_error = use_signal(|| false);
    // Skip completed screen when user presses Try Again or Play Again
    let skip_completed = use_signal(|| false);

    // Reactive signal: true once solution is revealed (from localStorage or in-session)
    let already_revealed = use_signal(|| stored_revealed);

    // Track what the user has accomplished (for completed screen messaging)
    let best_optimal = stored_best.is_some()
        && stored_min_steps.is_some()
        && stored_best == stored_min_steps;

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
                    let path_ids_set: std::collections::HashSet<i32> =
                        path_ids.into_iter().collect();
                    response
                        .recs
                        .into_iter()
                        .filter(|a| !path_ids_set.contains(&a.id))
                        .collect()
                }
                Err(ApiError::Conflict(_)) => {
                    // 409 — game conflicts with daily; silently return empty
                    Vec::new()
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    Vec::new()
                }
            }
        }
    });

    // ─── Card click handler ─────────────────────────────────────────

    // ─── Card click handler ─────────────────────────────────────────

    let on_card_click = {
        let mut path_sig = path.clone();
        let verifying_sig = verifying.clone();
        let won_sig = won.clone();
        let revealing_sig = revealing_solution.clone();
        let conflict_sig = conflict_error.clone();
        let end_clone = end.clone();
        let token_clone = token.clone();
        let is_daily_clone = is_daily;
        let min_steps_sig = min_steps.clone();
        let game_type_sig = game_type.clone();
        let first_att_sig = first_attempt.clone();
        let best_att_sig = best_attempt.clone();
        let sol_data_sig = solution_data.clone();
        let error_sig = error.clone();
        let already_revealed_sig = already_revealed.clone();

        Callback::new(move |selected: Anime| {
            if verifying_sig() || won_sig() || revealing_sig() || conflict_sig() {
                return;
            }

            let is_target = selected.id == end_clone.id;
            {
                let mut pw = path_sig.write();
                pw.push(selected);
            }
            let user_steps = path_sig.read().len() - 1;

            if !is_target {
                return;
            }

            // Spawn verify
            let path_ids: Vec<i32> = path_sig.read().iter().map(|a| a.id).collect();
            let token_inner = token_clone.clone();
            let path_for_sol = path_sig.clone();
            let mut verifying_inner = verifying_sig.clone();
            let mut won_inner = won_sig.clone();
            let mut min_steps_inner = min_steps_sig.clone();
            let mut game_type_inner = game_type_sig.clone();
            let mut first_inner = first_att_sig.clone();
            let mut best_inner = best_att_sig.clone();
            let mut sol_inner = sol_data_sig.clone();
            let mut error_inner = error_sig.clone();
            let mut revealing_inner = revealing_sig.clone();
            let mut conflict_inner = conflict_sig.clone();
            let already_revealed_inner = already_revealed_sig.clone();
            let is_daily_inner = is_daily_clone;

            dioxus::prelude::spawn(async move {
                let mut did_solution_fetch = false;
                match ApiClient::new().verify_win(&token_inner, &path_ids).await {
                    Ok(response) if response.is_valid => {
                        min_steps_inner.set(response.min_steps);
                        game_type_inner.set(response.game_type);

                        // If already revealed, skip intermediate screen and fetch solution directly
                        if already_revealed_inner() {
                            revealing_inner.set(true);
                            let ids: Vec<i32> = path_for_sol.read().iter().map(|a| a.id).collect();
                            match ApiClient::new().get_solution(&token_inner, &ids).await {
                                Ok(sol) => sol_inner.set(Some(sol)),
                                Err(ApiError::Conflict(_)) => conflict_inner.set(true),
                                Err(e) => {
                                    error_inner
                                        .set(Some(format!("Failed to fetch solution: {}", e)))
                                }
                            }
                            revealing_inner.set(false);
                            did_solution_fetch = true;
                        } else {
                            // First time winning — show intermediate screen
                            won_inner.set(true);

                            // Update attempts
                            let fa = first_inner.read().clone();
                            let ba = best_inner.read().clone();
                            let new_first = fa.or(Some(user_steps));
                            let new_best = if ba.map_or(true, |b| user_steps < b) {
                                Some(user_steps)
                            } else {
                                ba
                            };
                            first_inner.set(new_first.clone());
                            best_inner.set(new_best.clone());
                            if is_daily_inner {
                                save_attempts(new_first, new_best, Some(response.min_steps), false);
                            }

                            // Auto-reveal if optimal
                            if user_steps == response.min_steps {
                                revealing_inner.set(true);
                                let ids: Vec<i32> = path_for_sol.read().iter().map(|a| a.id).collect();
                                match ApiClient::new().get_solution(&token_inner, &ids).await {
                                    Ok(sol) => {
                                        sol_inner.set(Some(sol));
                                        if is_daily_inner {
                                            let fa = first_inner.read().clone();
                                            let ba = best_inner.read().clone();
                                            save_attempts(fa, ba, Some(response.min_steps), true);
                                        }
                                    }
                                    Err(ApiError::Conflict(_)) => conflict_inner.set(true),
                                    Err(e) => {
                                        error_inner
                                            .set(Some(format!("Failed to fetch solution: {}", e)))
                                    }
                                }
                                revealing_inner.set(false);
                            }
                        }
                    }
                    Ok(_) => {
                        error_inner
                            .set(Some("Invalid path — win not confirmed.".to_string()))
                    }
                    Err(ApiError::Conflict(_)) => {
                        conflict_inner.set(true)
                    }
                    Err(e) => {
                        error_inner.set(Some(format!("Failed to verify win: {}", e)))
                    }
                }
                if !did_solution_fetch {
                    verifying_inner.set(false);
                }
            });
        })
    };

    // ─── Reveal solution handler ────────────────────────────────────

     let on_reveal_solution = {
        let token_inner = token.clone();
        let mut revealing_sig = revealing_solution.clone();
        let path_sig = path.clone();
        let sol_sig = solution_data.clone();
        let conflict_sig = conflict_error.clone();
        let error_sig = error.clone();
        let first_att_sig = first_attempt.clone();
        let best_att_sig = best_attempt.clone();
        let min_steps_sig = min_steps.clone();
        let already_revealed_sig = already_revealed.clone();
        let is_daily_clone = is_daily;

        Callback::new(move |_| {
            revealing_sig.set(true);
            let path_ids: Vec<i32> = path_sig.read().iter().map(|a| a.id).collect();
            let token_for_req = token_inner.clone();
            let mut revealing_inner = revealing_sig.clone();
            let mut sol_inner = sol_sig.clone();
            let mut conflict_inner = conflict_sig.clone();
            let mut error_inner = error_sig.clone();
            let first_inner = first_att_sig.clone();
            let best_inner = best_att_sig.clone();
            let min_steps_inner = min_steps_sig.clone();
            let mut already_revealed_inner = already_revealed_sig.clone();
            let is_daily_inner = is_daily_clone;

            spawn(async move {
                match ApiClient::new().get_solution(&token_for_req, &path_ids).await {
                    Ok(sol) => {
                        sol_inner.set(Some(sol));
                        if is_daily_inner {
                            let fa = first_inner.read().clone();
                            let ba = best_inner.read().clone();
                            let ms = min_steps_inner();
                            save_attempts(fa, ba, Some(ms), true);
                        }
                        already_revealed_inner.set(true);
                    }
                    Err(ApiError::Conflict(_)) => conflict_inner.set(true),
                    Err(e) => {
                        error_inner
                            .set(Some(format!("Failed to fetch solution: {}", e)))
                    }
                }
                revealing_inner.set(false);
            });
        })
    };

    // ─── Try again handler ──────────────────────────────────────────

    // Shared handler: reset game state AND bypass completed screen
    // Used by both "Try Again" (intermediate) and "Play again" (completed)
    let on_play_again = {
        let start_inner = start.clone();
        let mut path_sig = path.clone();
        let mut won_sig = won.clone();
        let mut sol_sig = solution_data.clone();
        let mut error_sig = error.clone();
        let mut skip_sig = skip_completed.clone();

        Callback::new(move |_| {
            path_sig.set(vec![start_inner.clone()]);
            won_sig.set(false);
            sol_sig.set(None);
            error_sig.set(None);
            skip_sig.set(true);
        })
    };

    // Alias for the completed-screen button
    let on_completed_play_again = on_play_again.clone();

      // ─── Render ─────────────────────────────────────────────────────

     // 0. Already completed today — show summary screen
    // Only when NOT in the middle of a win (won == false means fresh page load or after reset)
    // And NOT when skip_completed is set (user just pressed Try Again / Play Again)
    if stored_first.is_some() && !skip_completed() && !won() {
        let best_val = stored_best;
        let first_val = stored_first.unwrap_or(0);
        let show_best = best_val.is_some();

        return rsx! {
            div { class: "game-container completed-today",
                h1 { "✅ You've completed today's daily challenge!" }

                div { class: "win-stats",
                    p { class: "attempt-stat", "First attempt: {first_val} steps" }
                    if show_best {
                        p { class: "attempt-stat",
                            if best_val == Some(first_val) {
                                "Best attempt: {first_val} steps"
                            } else {
                                "Best attempt: {best_val.unwrap()} steps"
                            }
                        }
                    }
                }

                if best_optimal {
                    p { class: "completed-note", "You've already found the best path!" }
                } else if already_revealed() {
                    p { class: "completed-note",
                        "You've already revealed the solution. Play again for fun!"
                    }
                } else {
                    p { class: "completed-note", "Play again to try for a better attempt!" }
                }

                div { class: "win-actions",
                    if best_optimal || already_revealed() {
                        button {
                            class: "btn btn-primary",
                            onclick: on_completed_play_again.clone(),
                            "🎮 Play again for fun"
                        }
                    } else {
                        button {
                            class: "btn btn-primary",
                            onclick: on_completed_play_again,
                            "🔄 Play again for a better attempt"
                        }
                    }
                    a { href: "/", class: "btn btn-secondary", "🏠 Go Home" }
                }
            }
        };
    }

    // 1. Conflict
    if conflict_error() {
        return rsx! {
            div { class: "game-container conflict-banner",
                h1 { "⚠️ Conflict" }
                p { "This game has become today's daily challenge. Start a new game instead!" }
                div { class: "win-actions",
                    a { href: "/", class: "btn btn-primary", "Play Daily Challenge" }
                }
            }
        };
    }

    // 2. Solution revealed
    if let Some(ref sol) = solution_data() {
        return rsx! {
            WinScreen {
                start: start.clone(),
                end: end.clone(),
                user_path: path.read().clone(),
                user_steps: path.read().len() - 1,
                min_steps: sol.min_steps,
                shortest_paths: sol.shortest_paths.clone(),
                game_type: sol.game_type.clone(),
                first_attempt: first_attempt(),
                best_attempt: best_attempt(),
            }
        };
    }

    // 3. Valid win (non-optimal) — intermediate screen
    if won() {
        let user_steps = path.read().len() - 1;
        let rating = match (user_steps, min_steps()) {
            (u, m) if u == m => "⭐⭐⭐",
            (u, m) if u == m + 1 => "⭐⭐",
            (u, m) if u <= m + 2 => "⭐",
            _ => "🔄",
        };

        return rsx! {
            div { class: "game-container win-intermediate",
                h1 { "🎉 You found a path!" }
                div { class: "win-rating", "{rating}" }
                div { class: "win-stats",
                    p { "Your path: {user_steps} steps" }
                    p { "Shortest path: {min_steps()} steps" }
                    if let Some(fa) = first_attempt() {
                        p { class: "attempt-stat", "First attempt: {fa} steps" }
                    }
                    if let Some(ba) = best_attempt() {
                        p { class: "attempt-stat", "Best attempt: {ba} steps" }
                    }
                }

                if revealing_solution() {
                    div { class: "verifying-overlay", "Fetching solution..." }
                }

                div { class: "win-actions",
                    if revealing_solution() {
                        button { class: "btn btn-secondary", disabled: true, "Revealing..." }
                    } else {
                        button {
                            class: "btn btn-secondary",
                            onclick: on_play_again,
                            "🔄 Try Again"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: on_reveal_solution,
                            "👁 Reveal Solution"
                        }
                    }
                }
            }
        };
    }

    // 4. Normal gameplay
    let path_count = path.read().len();

    rsx! {
        div { class: "game-container",
            CardSizeSlider {}

            div { class: "game-header",
                AnimeMiniCard { anime: start.clone(), label: "Start" }
                span { class: "arrow", "⟶" }
                AnimeMiniCard { anime: end.clone(), label: "Target" }
            }

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

            if let Some(ref err) = error() {
                div { class: "error-box", "{err}" }
            }

            if verifying() {
                div { class: "verifying-overlay", "Verifying win..." }
            }

            div {
                class: "recs-grid",
                style: format!(
                    "--card-img-height: {}px; --card-min-width: {}px;",
                    (280_f64 * card_size() as f64 / 100.0) as usize,
                    (200_f64 * card_size() as f64 / 100.0) as usize,
                ),
                match recs() {
                    Some(ref rec_list) => rsx! {
                        for anime in rec_list.iter().cloned() {
                            RecsCard {
                                anime: anime.clone(),
                                clickable: !verifying() && !won(),
                                on_click: on_card_click.clone(),
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

#[cfg(test)]
mod tests {
    #[test]
    fn card_size_calculation() {
        assert_eq!((280_f64 * 50.0 / 100.0) as usize, 140);
        assert_eq!((280_f64 * 100.0 / 100.0) as usize, 280);
        assert_eq!((280_f64 * 150.0 / 100.0) as usize, 420);
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
