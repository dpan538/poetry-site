use crate::{
    all, attr, clear_page_listeners, document, grid, html, mobile, on, overlays, query, set, style,
    window,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
use web_sys::{
    Document, Element, KeyboardEvent, MouseEvent, PopStateEvent, ScrollRestoration, Storage,
};

const HISTORY: &str = "three-worlds-history";
const LAST: &str = "three-worlds-entry-last";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Visit {
    id: String,
    world: String,
    emotion_score: f64,
    timestamp: f64,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    world: String,
}

#[derive(Deserialize)]
struct Candidate {
    id: String,
    world: String,
    score: f64,
    types: Vec<String>,
    affinity: f64,
    scene: Option<String>,
}

#[derive(Deserialize)]
struct Reading {
    current: String,
    world: String,
    works: Vec<Candidate>,
}

pub fn storage() -> Option<Storage> {
    window().local_storage().ok().flatten()
}

fn history() -> Vec<Visit> {
    storage()
        .and_then(|s| s.get_item(HISTORY).ok().flatten())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn parse<T: serde::de::DeserializeOwned>(selector: &str) -> Option<T> {
    let data = query(selector)?.text_content()?;
    serde_json::from_str(&data).ok()
}

fn entry_path(entries: Vec<Entry>) -> Option<String> {
    let recent = history();
    let last = storage().and_then(|s| s.get_item(LAST).ok().flatten());
    let mut pool: Vec<_> = entries
        .iter()
        .filter(|e| Some(&e.id) != last.as_ref() && !recent.iter().take(4).any(|v| v.id == e.id))
        .collect();
    if pool.is_empty() {
        pool = entries
            .iter()
            .filter(|e| Some(&e.id) != last.as_ref())
            .collect();
    }
    if pool.is_empty() {
        pool = entries.iter().collect();
    }
    let entry = pool.get((js_sys::Math::random() * pool.len() as f64) as usize)?;
    if let Some(store) = storage() {
        let _ = store.set_item(LAST, &entry.id);
    }
    Some(format!("/{}/{}", entry.world, entry.id))
}

fn enter(action: TurnAction) {
    if let Some(path) = parse("#entry-data").and_then(entry_path) {
        turn_to(path, action, 0);
    }
}

fn set_links(role: &str, path: Option<&str>) {
    for e in all(&format!("[data-poem-nav='{role}']")) {
        if let Some(path) = path {
            set(&e, "href", path);
            style(&e, "display", "");
            let _ = e.remove_attribute("aria-disabled");
        } else {
            set(&e, "href", "#");
            set(&e, "aria-disabled", "true");
            style(&e, "display", "none");
        }
    }
}

fn content(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let union = a.iter().chain(b).collect::<BTreeSet<_>>().len();
    a.iter().filter(|t| b.contains(t)).count() as f64 / union as f64
}

fn read_navigation(data: Reading) {
    let Some(current) = data
        .works
        .iter()
        .find(|w| w.id == data.current && w.world == data.world)
    else {
        return;
    };
    let mut visits = history();
    visits.retain(|v| v.id != current.id || v.world != current.world);
    visits.insert(
        0,
        Visit {
            id: current.id.clone(),
            world: current.world.clone(),
            emotion_score: current.score,
            timestamp: js_sys::Date::now(),
        },
    );
    visits.truncate(20);
    if let (Some(store), Ok(raw)) = (storage(), serde_json::to_string(&visits)) {
        let _ = store.set_item(HISTORY, &raw);
    }
    let prev = visits.get(1);
    let prior = visits.get(2);
    let direction = match (prev, prior) {
        (Some(a), Some(b))
            if b.emotion_score < a.emotion_score && a.emotion_score < current.score =>
        {
            -1
        }
        (Some(a), Some(b))
            if b.emotion_score > a.emotion_score && a.emotion_score > current.score =>
        {
            1
        }
        (Some(a), Some(_)) if a.emotion_score < current.score => 1,
        (Some(_), Some(_)) => -1,
        _ => 0,
    };
    let recent_scenes: BTreeSet<_> = visits
        .iter()
        .take(3)
        .filter_map(|visit| {
            data.works
                .iter()
                .find(|work| work.id == visit.id && work.world == visit.world)
                .and_then(|work| work.scene.as_deref())
        })
        .collect();
    let mut candidates: Vec<_> = data
        .works
        .iter()
        .filter(|c| {
            !visits
                .iter()
                .take(3)
                .any(|v| v.id == c.id && v.world == c.world)
        })
        .filter(|work| {
            !work
                .scene
                .as_deref()
                .is_some_and(|scene| recent_scenes.contains(scene))
        })
        .map(|c| {
            let arc = match direction {
                1 => {
                    if c.score > current.score {
                        1.0
                    } else {
                        0.2
                    }
                }
                -1 => {
                    if c.score < current.score {
                        1.0
                    } else {
                        0.2
                    }
                }
                _ => 1.0 - (c.score - current.score).abs(),
            };
            let world = if c.world == current.world { 0.3 } else { 1.0 };
            let penalty = if prev.is_some_and(|p| p.world == c.world) {
                0.6
            } else {
                1.0
            };
            (
                c,
                (0.35 * c.affinity
                    + 0.25 * arc
                    + 0.25 * content(&current.types, &c.types)
                    + 0.1 * world
                    + 0.05)
                    * penalty,
            )
        })
        .collect();
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
    let next = candidates
        .get((js_sys::Math::random() * candidates.len().min(5) as f64) as usize)
        .map(|(c, _)| format!("/{}/{}", c.world, c.id));
    let previous = prev
        .filter(|v| {
            data.works
                .iter()
                .any(|w| w.world == v.world && w.id == v.id)
        })
        .map(|v| format!("/{}/{}", v.world, v.id));
    set_links("next", next.as_deref());
    set_links("prev", previous.as_deref());
}

fn reading_keyboard() {
    on(document().as_ref(), "keydown", move |event| {
        let Some(e) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if event.default_prevented() || query("#manifold[open], .index-overlay.is-open, .mobile-sheet.is-open, .mobile-index-sheet.is-open").is_some() { return; }
        if e.alt_key() || e.ctrl_key() || e.meta_key() || e.shift_key() {
            return;
        }
        if event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
            .is_some_and(|el| {
                matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                    || el.is_content_editable()
            })
        {
            return;
        }
        let selector = match e.key().as_str() {
            "ArrowLeft" => "#nav-prev",
            "ArrowRight" => "#nav-next",
            _ => return,
        };
        if let Some(link) = query(selector).filter(|l| attr(l, "href") != "#") {
            html(&link).click();
        }
    });
}

fn reading_turns() {
    on(document().as_ref(), "click", move |event| {
        let Some(click) = event.dyn_ref::<MouseEvent>() else {
            return;
        };
        if event.default_prevented()
            || click.button() != 0
            || click.alt_key()
            || click.ctrl_key()
            || click.meta_key()
            || click.shift_key()
        {
            return;
        }
        if mobile()
            && event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|element| {
                    element
                        .closest("#mobile-enter-button")
                        .ok()
                        .flatten()
                        .is_some()
                })
        {
            event.prevent_default();
            enter(TurnAction::Push);
            return;
        }
        let Some(link) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .and_then(|element| {
                element
                    .closest("[data-poem-nav], .mobile-index-link, .site-nav a")
                    .ok()
                    .flatten()
            })
        else {
            return;
        };
        let path = attr(&link, "href");
        if !reader_path(&path) {
            return;
        }
        event.prevent_default();
        let direction = match attr(&link, "data-poem-nav").as_str() {
            "prev" => -1,
            "next" => 1,
            _ => 0,
        };
        if DISPLAYED.with_borrow(|shown| shown == &path) {
            if let Some(close) = query(
                ".mobile-sheet.is-open [data-mobile-close], .mobile-index-sheet.is-open [data-mobile-site-index-close]",
            ) {
                html(&close).click();
            }
        }
        turn_to(path, TurnAction::Push, direction);
    });

    on(window().as_ref(), "popstate", |event| {
        let Ok(path) = window().location().pathname() else {
            return;
        };
        if !reader_path(&path) {
            return;
        }
        let scroll = event
            .dyn_ref::<PopStateEvent>()
            .and_then(|e| e.state().as_f64())
            .unwrap_or_default();
        turn_to(path, TurnAction::Pop(scroll), 0);
    });
}

#[wasm_bindgen(module = "/reading_turn.js")]
extern "C" {
    #[wasm_bindgen(js_name = readWork)]
    fn read_work(path: &str, done: &js_sys::Function);
    #[wasm_bindgen(js_name = showWork)]
    fn show_work(update: &js_sys::Function, direction: i8);
    #[wasm_bindgen(js_name = stopWorkTurn)]
    fn stop_work_turn();
    #[wasm_bindgen(js_name = applyReaderStyles)]
    fn apply_reader_styles(page: &Document);
}

thread_local! {
    static TURN: Cell<u32> = const { Cell::new(0) };
    static DISPLAYED: RefCell<String> = const { RefCell::new(String::new()) };
}

fn work_path(path: &str) -> bool {
    let Some((world, slug)) = path.strip_prefix('/').and_then(|path| path.split_once('/')) else {
        return false;
    };
    matches!(world, "their-world" | "bless-you" | "still-life")
        && !slug.is_empty()
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn reader_path(path: &str) -> bool {
    matches!(path, "/" | "/about") || work_path(path)
}

#[derive(Clone, Copy)]
enum TurnAction {
    Push,
    Pop(f64),
    Replace,
}

fn turn_to(path: String, action: TurnAction, direction: i8) {
    let id = TURN.with(|turn| {
        let id = turn.get().wrapping_add(1);
        turn.set(id);
        id
    });
    if DISPLAYED.with_borrow(|shown| shown == &path) {
        return;
    }
    let animate =
        direction != 0 && work_path(&path) && DISPLAYED.with_borrow(|shown| work_path(shown));
    let target = path.clone();
    let done = Closure::once_into_js(move |page: JsValue| {
        if !TURN.with(|turn| turn.get() == id) {
            return;
        }
        let Some(next) = page.dyn_into::<Document>().ok().filter(|page| {
            page.query_selector(".site-body-stack")
                .ok()
                .flatten()
                .is_some()
        }) else {
            match action {
                TurnAction::Push => {
                    let _ = window().location().assign(&target);
                }
                TurnAction::Pop(_) => {
                    let _ = window().location().reload();
                }
                TurnAction::Replace => {
                    let _ = window().location().replace(&target);
                }
            }
            return;
        };
        if target == "/" && !mobile() {
            let entry = next
                .get_element_by_id("entry-data")
                .and_then(|element| element.text_content())
                .and_then(|data| serde_json::from_str(&data).ok())
                .and_then(entry_path);
            if let Some(path) = entry {
                turn_to(path, action, 0);
            }
            return;
        }
        if animate {
            let update = Closure::once_into_js(move || {
                if TURN.with(|turn| turn.get() == id) {
                    replace_reading(next, &target, action);
                }
            });
            show_work(update.unchecked_ref(), direction);
        } else {
            stop_work_turn();
            replace_reading(next, &target, action);
        }
    });
    read_work(&path, done.unchecked_ref());
}

fn replace_reading(next: Document, path: &str, action: TurnAction) {
    let (Some(current), Some(root), Some(body), Some(incoming)) = (
        query(".site-body-stack"),
        query("html"),
        query("body"),
        next.query_selector(".site-body-stack").ok().flatten(),
    ) else {
        return;
    };
    let Ok(new_shell) = document().import_node_with_deep(&incoming, true) else {
        return;
    };
    let Ok(history) = window().history() else {
        return;
    };
    let scroll = match action {
        TurnAction::Push => {
            let offset = window().scroll_y().unwrap_or_default();
            let _ = history.replace_state_with_url(&offset.into(), "", None);
            if history
                .push_state_with_url(&0.into(), "", Some(path))
                .is_err()
            {
                let _ = window().location().assign(path);
                return;
            }
            0.0
        }
        TurnAction::Pop(scroll) => scroll,
        TurnAction::Replace => {
            if history
                .replace_state_with_url(&0.into(), "", Some(path))
                .is_err()
            {
                let _ = window().location().replace(path);
                return;
            }
            0.0
        }
    };
    apply_reader_styles(&next);
    overlays::dispose();
    clear_page_listeners();
    let _ = current.replace_with_with_node_1(&new_shell);
    if let Some(incoming) = next.document_element() {
        for name in ["data-world", "data-reading-type"] {
            if let Some(value) = incoming.get_attribute(name) {
                set(&root, name, &value);
            } else {
                let _ = root.remove_attribute(name);
            }
        }
    }
    if let Some(incoming) = next.body() {
        body.set_class_name(&incoming.class_name());
        if incoming.has_attribute("data-entry-page") {
            set(&body, "data-entry-page", "");
        } else {
            let _ = body.remove_attribute("data-entry-page");
        }
    }
    for link in all(".site-nav a") {
        let href = attr(&link, "href");
        if href == path && path != "/" {
            set(&link, "aria-current", "page");
        } else {
            let _ = link.remove_attribute("aria-current");
        }
    }
    document().set_title(&next.title());
    for selector in [
        "link[rel='canonical']",
        "meta[property='og:url']",
        "meta[property='og:title']",
        "meta[name='description']",
    ] {
        if let (Some(current), Some(incoming)) = (
            query(selector),
            next.query_selector(selector).ok().flatten(),
        ) {
            let attribute = if selector.starts_with("link") {
                "href"
            } else {
                "content"
            };
            set(&current, attribute, &attr(&incoming, attribute));
        }
    }
    if let Some(head) = document().query_selector("head").ok().flatten() {
        for id in ["reading-data", "entry-data"] {
            if let Some(current) = document().get_element_by_id(id) {
                current.remove();
            }
            if let Some(incoming) = next.get_element_by_id(id) {
                if let Ok(node) = document().import_node_with_deep(&incoming, true) {
                    let _ = head.append_child(&node);
                }
            }
        }
    }
    DISPLAYED.with_borrow_mut(|shown| *shown = path.to_owned());
    if let Some(data) = parse::<Reading>("#reading-data") {
        read_navigation(data);
    }
    overlays::init();
    grid::init();
    style(&root, "scroll-behavior", "auto");
    window().scroll_to_with_x_and_y(0.0, scroll);
    style(&root, "scroll-behavior", "");
}

pub fn init() {
    DISPLAYED.with_borrow_mut(|shown| *shown = window().location().pathname().unwrap_or_default());
    if let Ok(history) = window().history() {
        let _ = history.set_scroll_restoration(ScrollRestoration::Manual);
    }
    if query("#entry-data").is_some() && !mobile() {
        enter(TurnAction::Replace);
    }
    if let Ok(Some(view)) = window().match_media("(max-width: 760px)") {
        on(view.as_ref(), "change", |_| {
            if !mobile() && DISPLAYED.with_borrow(|shown| shown == "/") {
                enter(TurnAction::Replace);
            }
        });
    }
    if let Some(data) = parse::<Reading>("#reading-data") {
        read_navigation(data);
    }
    reading_keyboard();
    reading_turns();
}
