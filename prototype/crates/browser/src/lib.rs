mod grid;
mod motion;
mod navigation;
mod overlays;

use std::cell::RefCell;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Document, Element, Event, EventTarget, HtmlElement, Window};

fn window() -> Window {
    web_sys::window().expect("browser window")
}
fn document() -> Document {
    window().document().expect("browser document")
}
fn query(selector: &str) -> Option<Element> {
    document().query_selector(selector).ok().flatten()
}
fn elements(root: &Element, selector: &str) -> Vec<Element> {
    let Ok(list) = root.query_selector_all(selector) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into().ok())
        .collect()
}
fn all(selector: &str) -> Vec<Element> {
    document()
        .document_element()
        .map(|el| elements(&el, selector))
        .unwrap_or_default()
}
fn attr(element: &Element, key: &str) -> String {
    element.get_attribute(key).unwrap_or_default()
}
fn html(element: &Element) -> &HtmlElement {
    element.unchecked_ref()
}
fn style(element: &Element, key: &str, value: &str) {
    let _ = html(element).style().set_property(key, value);
}
fn set(element: &Element, key: &str, value: &str) {
    let _ = element.set_attribute(key, value);
}
fn toggle(element: &Element, class: &str, on: bool) {
    let _ = element.class_list().toggle_with_force(class, on);
}
fn focus(element: &Element) {
    let _ = html(element).focus();
}
fn on(target: &EventTarget, name: &str, listener: impl FnMut(Event) + 'static) {
    let listener = Closure::<dyn FnMut(Event)>::new(listener);
    if target
        .add_event_listener_with_callback(name, listener.as_ref().unchecked_ref())
        .is_ok()
    {
        listener.forget();
    }
}
struct PageListener {
    target: EventTarget,
    name: String,
    listener: Closure<dyn FnMut(Event)>,
}

thread_local! {
    static PAGE_LISTENERS: RefCell<Vec<PageListener>> = const { RefCell::new(Vec::new()) };
}

fn on_page(target: &EventTarget, name: &str, listener: impl FnMut(Event) + 'static) {
    let listener = Closure::<dyn FnMut(Event)>::new(listener);
    if target
        .add_event_listener_with_callback(name, listener.as_ref().unchecked_ref())
        .is_ok()
    {
        PAGE_LISTENERS.with_borrow_mut(|listeners| {
            listeners.push(PageListener {
                target: target.clone(),
                name: name.to_owned(),
                listener,
            })
        });
    }
}

fn clear_page_listeners() {
    PAGE_LISTENERS.with_borrow_mut(|listeners| {
        for entry in listeners.drain(..) {
            let _ = entry.target.remove_event_listener_with_callback(
                &entry.name,
                entry.listener.as_ref().unchecked_ref(),
            );
        }
    });
}
fn element(tag: &str, class: &str) -> Element {
    let el = document().create_element(tag).expect("valid HTML element");
    if !class.is_empty() {
        el.set_class_name(class);
    }
    el
}
fn mobile() -> bool {
    window()
        .match_media("(max-width: 760px)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}
fn world() -> String {
    query("html")
        .and_then(|e| e.get_attribute("data-world"))
        .unwrap_or_else(|| "their-world".into())
}

#[wasm_bindgen]
pub fn start(catalog: &str) -> Result<(), JsValue> {
    navigation::install_catalog(catalog)?;
    if let Some(root) = document().document_element() {
        let pointer_root = root.clone();
        on(document().as_ref(), "pointerdown", move |_| {
            set(&pointer_root, "data-pointer-input", "");
        });
        on(document().as_ref(), "keydown", move |_| {
            let _ = root.remove_attribute("data-pointer-input");
        });
    }
    navigation::init();
    overlays::init();
    grid::init();
    if query("#manifold[open]").is_none() {
        motion::enter_page();
    }
    if let Some(body) = query("body") {
        set(&body, "data-rust-ready", "");
    }
    Ok(())
}
