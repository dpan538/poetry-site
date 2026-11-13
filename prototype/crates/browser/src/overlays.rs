use crate::{
    all, attr, document, elements, focus, html, motion, on_page as on, query, set, style, toggle,
    window, world,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, closure::Closure, prelude::wasm_bindgen};
use web_sys::{Animation, Element, Event, HtmlDialogElement, KeyboardEvent};

fn layer(overlay: Element, open: Vec<Element>, close: &str, body_class: &'static str) {
    set(&overlay, "inert", "");
    let body: Element = document().body().expect("document body").into();
    let last = Rc::new(RefCell::new(None::<Element>));
    let close_button = elements(&overlay, close).into_iter().next();
    let exit = Rc::new({
        let overlay = overlay.clone();
        let body = body.clone();
        let last = last.clone();
        move || {
            if !overlay.class_list().contains("is-open") {
                return;
            }
            toggle(&overlay, "is-open", false);
            toggle(&body, body_class, false);
            if let Some(el) = last.borrow_mut().take() {
                if el.has_attribute("aria-expanded") {
                    set(&el, "aria-expanded", "false");
                }
                focus(&el);
            }
            set(&overlay, "inert", "");
            set(&overlay, "aria-hidden", "true");
        }
    });
    for button in open {
        let (overlay, body, last, close_button, trigger) = (
            overlay.clone(),
            body.clone(),
            last.clone(),
            close_button.clone(),
            button.clone(),
        );
        on(button.as_ref(), "click", move |_| {
            crate::navigation::populate_index(&overlay);
            *last.borrow_mut() = Some(trigger.clone());
            let _ = overlay.remove_attribute("inert");
            toggle(&overlay, "is-open", true);
            set(&overlay, "aria-hidden", "false");
            toggle(&body, body_class, true);
            if trigger.has_attribute("aria-expanded") {
                set(&trigger, "aria-expanded", "true");
            }
            if let Some(button) = &close_button {
                focus(button);
            }
        });
    }
    if let Some(button) = close_button {
        let exit = exit.clone();
        on(button.as_ref(), "click", move |_| exit());
    }
    {
        let el = overlay.clone();
        let exit = exit.clone();
        on(overlay.as_ref(), "click", move |event| {
            if event
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .as_ref()
                == Some(&el)
            {
                exit();
            }
        });
    }
    let el = overlay.clone();
    on(document().as_ref(), "keydown", move |event| {
        if !el.class_list().contains("is-open") {
            return;
        }
        let Some(key) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if key.key() == "Escape" {
            event.prevent_default();
            exit();
        }
        if key.key() == "Tab" {
            let visible: Vec<_> = elements(&el, "a[href],button:not([disabled]),[tabindex='0']")
                .into_iter()
                .filter(|item| {
                    item.get_bounding_client_rect().width() > 0.0
                        && item
                            .closest("[aria-hidden='true']")
                            .ok()
                            .flatten()
                            .is_none()
                })
                .collect();
            let active = document().active_element();
            if key.shift_key() && active.as_ref() == visible.first() {
                if let Some(last) = visible.last() {
                    event.prevent_default();
                    focus(last);
                }
            } else if !key.shift_key() && active.as_ref() == visible.last() {
                if let Some(first) = visible.first() {
                    event.prevent_default();
                    focus(first);
                }
            }
        }
    });
}

fn mobile_tabs(button_key: &'static str, panel_key: &'static str) {
    let tabs = all(&format!("[{button_key}]"));
    let Some(strip) = tabs.first().and_then(|tab| tab.parent_element()) else {
        return;
    };
    let panels = all(&format!("[{panel_key}]"));
    let order = RefCell::new(tabs.clone());
    let transitions = RefCell::new(Vec::<Animation>::new());
    let sync = Rc::new(move |selected: &str, animate: bool| {
        let mut order = order.borrow_mut();
        let Some(index) = order
            .iter()
            .position(|tab| attr(tab, button_key) == selected)
        else {
            return;
        };
        let mut transitions = transitions.borrow_mut();
        for animation in transitions.drain(..) {
            animation.cancel();
        }
        for tab in &tabs {
            let active = attr(tab, button_key) == selected;
            toggle(tab, "is-active", active);
            set(tab, "aria-selected", if active { "true" } else { "false" });
        }
        if index > 0 {
            if animate {
                let next = order.remove(index);
                order.rotate_left(1);
                order.insert(0, next);
            } else {
                order.rotate_left(index);
            }
            let focused = document().active_element().filter(|el| tabs.contains(el));
            for tab in order.iter() {
                let _ = strip.append_child(tab);
            }
            if let Some(tab) = focused {
                focus(&tab);
            }
        }
        if animate {
            for (index, label) in elements(&strip, "button > span").iter().enumerate() {
                if let Some(animation) = motion::reveal_index(label, 0.68, 160, index as u16 * 24) {
                    transitions.push(animation);
                }
            }
        }
        for panel in &panels {
            let active = attr(panel, panel_key) == selected;
            toggle(panel, "is-active", active);
            set(panel, "aria-hidden", if active { "false" } else { "true" });
            if active && animate {
                panel.set_scroll_top(0);
                if let Some(animation) = motion::reveal_index(panel, 0.52, 190, 35) {
                    transitions.push(animation);
                }
            }
        }
    });
    let current = world();
    sync(&current, false);
    for tab in all(&format!("[{button_key}]")) {
        let (sync, tab) = (sync.clone(), tab.clone());
        on(tab.clone().as_ref(), "click", move |_| {
            if attr(&tab, "aria-selected") != "true" {
                sync(&attr(&tab, button_key), true);
            }
        });
    }
}

fn position_index_rule(switcher: &Element) {
    let Ok(Some(tab)) = switcher.query_selector(".world-tab.is-lead") else {
        return;
    };
    let rect = tab.get_bounding_client_rect();
    let left = rect.left() - switcher.get_bounding_client_rect().left();
    style(switcher, "--index-rule-left", &format!("{left:.2}px"));
    style(
        switcher,
        "--index-rule-width",
        &format!("{:.2}px", rect.width()),
    );
    set(switcher, "data-index-world", &attr(&tab, "data-world"));
}

fn desktop_tabs() {
    let Some(switcher) = query("#index-overlay .world-switcher") else {
        return;
    };
    on(window().as_ref(), "resize", {
        let switcher = switcher.clone();
        move |_| position_index_rule(&switcher)
    });
    let tabs = all("#index-overlay .world-tab");
    let panels = all("#index-overlay .world-panel");
    let animations = RefCell::new(Vec::<Animation>::new());
    let revision = Rc::new(Cell::new(0u32));
    let sync = Rc::new(move |selected: &str, animate: bool| {
        let current = revision.get().wrapping_add(1);
        revision.set(current);
        let opacities: Vec<_> = panels
            .iter()
            .map(|panel| {
                let classes = panel.class_list();
                if animate && (classes.contains("is-shown") || classes.contains("is-leaving")) {
                    motion::opacity(panel)
                } else {
                    0.0
                }
            })
            .collect();
        for animation in animations.borrow_mut().drain(..) {
            animation.cancel();
        }
        for tab in &tabs {
            let active = attr(tab, "data-world") == selected;
            toggle(tab, "is-lead", active);
            set(tab, "aria-selected", if active { "true" } else { "false" });
        }
        position_index_rule(&switcher);
        for (panel, opacity) in panels.iter().zip(opacities) {
            let active = attr(panel, "data-panel") == selected;
            toggle(panel, "is-shown", active);
            toggle(panel, "is-leaving", false);
            set(panel, "aria-hidden", if active { "false" } else { "true" });
            if active {
                let _ = panel.remove_attribute("inert");
                if animate
                    && let Some(animation) = motion::tween(panel, opacity, 1.0, None, [420, 420])
                {
                    animations.borrow_mut().push(animation);
                }
            } else {
                set(panel, "inert", "");
                if opacity > 0.0
                    && let Some(animation) = motion::tween(panel, opacity, 0.0, None, [250, 250])
                {
                    toggle(panel, "is-leaving", true);
                    let (panel, revision) = (panel.clone(), revision.clone());
                    let finish = Closure::once_into_js(move |event: Event| {
                        if revision.get() == current {
                            toggle(&panel, "is-leaving", false);
                        }
                        if let Some(animation) = event
                            .current_target()
                            .and_then(|target| target.dyn_into::<Animation>().ok())
                        {
                            animation.set_onfinish(None);
                            animation.set_oncancel(None);
                            animation.cancel();
                        }
                    });
                    animation.set_onfinish(finish.dyn_ref());
                    animation.set_oncancel(finish.dyn_ref());
                    animations.borrow_mut().push(animation);
                }
            }
        }
    });
    sync(&world(), false);
    for tab in all("#index-overlay .world-tab") {
        let sync = sync.clone();
        on(tab.clone().as_ref(), "click", move |_| {
            if attr(&tab, "aria-selected") != "true" {
                sync(&attr(&tab, "data-world"), true);
            }
        });
    }
}

#[wasm_bindgen(module = "/manifold_player.js")]
extern "C" {
    #[wasm_bindgen(js_name = playManifold)]
    fn play_scene(dialog: &HtmlDialogElement);
    #[wasm_bindgen(js_name = stopManifold)]
    fn stop_scene(dialog: &HtmlDialogElement);
}

pub fn dispose() {
    if let Some(dialog) = query("#manifold").and_then(|d| d.dyn_into::<HtmlDialogElement>().ok()) {
        stop_scene(&dialog);
        dialog.close();
    }
}

fn manifold() {
    let Some(dialog) = query("#manifold").and_then(|d| d.dyn_into::<HtmlDialogElement>().ok())
    else {
        return;
    };
    let Some(playback) = query("[data-manifold-playback]") else {
        return;
    };
    let media = window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten();
    let playing = Rc::new(Cell::new(false));
    let start = Rc::new({
        let (dialog, playback, media, playing) = (
            dialog.clone(),
            playback.clone(),
            media.clone(),
            playing.clone(),
        );
        move || {
            let _ = dialog.remove_attribute("data-scene-error");
            html(&playback).set_hidden(media.as_ref().is_some_and(|m| m.matches()));
            set(&playback, "aria-label", "Pause animation");
            let _ = playback.remove_attribute("data-paused");
            playing.set(true);
            play_scene(&dialog);
        }
    });
    let opener = Rc::new(RefCell::new(None::<Element>));
    let finalize = Rc::new({
        let (dialog, opener) = (dialog.clone(), opener.clone());
        move || {
            if dialog.open() {
                return;
            }
            stop_scene(&dialog);
            let _ = dialog.remove_attribute("data-closing");
            let _ = dialog.remove_attribute("data-scene-ready");
            if let Some(trigger) = opener.borrow_mut().take() {
                focus(&trigger);
            }
        }
    });
    let entrance = Rc::new(RefCell::new(None::<Animation>));
    let quote = query("#manifold blockquote").map(|quote| {
        let original = quote.text_content().unwrap_or_default();
        Rc::new(move || crate::grid::compose_quote(&quote, &original))
    });
    if let Some(compose) = quote.clone() {
        let dialog = dialog.clone();
        on(window().as_ref(), "resize", move |_| {
            if dialog.open() {
                compose();
            }
        });
    }
    let show = Rc::new({
        let (dialog, start, opener, entrance) = (
            dialog.clone(),
            start.clone(),
            opener.clone(),
            entrance.clone(),
        );
        move |trigger: Option<Element>| {
            if dialog.open() || dialog.has_attribute("data-closing") || dialog.show_modal().is_err()
            {
                return;
            }
            *opener.borrow_mut() = trigger;
            let _ = dialog.remove_attribute("data-scene-ready");
            if let Some(compose) = &quote {
                compose();
            }
            *entrance.borrow_mut() = motion::tween(&dialog, 0.0, 1.0, None, [360, 430]);
            start();
        }
    });
    for button in all("[data-manifold-open]") {
        let (show, button) = (show.clone(), button.clone());
        on(button.clone().as_ref(), "click", move |_| {
            show(Some(button.clone()))
        });
    }

    let exit = Rc::new({
        let (dialog, entrance, finalize) = (dialog.clone(), entrance.clone(), finalize.clone());
        move || {
            if !dialog.open() || dialog.has_attribute("data-closing") {
                return;
            }
            let opacity = motion::opacity(&dialog);
            if let Some(animation) = entrance.borrow_mut().take() {
                animation.cancel();
            }
            let Some(animation) = motion::tween(&dialog, opacity, 0.0, None, [250, 320]) else {
                dialog.close();
                finalize();
                return;
            };
            set(&dialog, "data-closing", "");
            let (dialog, finalize) = (dialog.clone(), finalize.clone());
            let finish = Closure::once_into_js(move |event: Event| {
                dialog.close();
                finalize();
                if let Some(animation) = event
                    .current_target()
                    .and_then(|e| e.dyn_into::<Animation>().ok())
                {
                    animation.set_onfinish(None);
                    animation.set_oncancel(None);
                    animation.cancel();
                }
            });
            animation.set_onfinish(finish.dyn_ref());
            animation.set_oncancel(finish.dyn_ref());
        }
    });
    if let Some(form) = query(".manifold-exit") {
        let exit = exit.clone();
        on(form.as_ref(), "submit", move |event| {
            event.prevent_default();
            exit();
        });
    }
    {
        let exit = exit.clone();
        on(dialog.as_ref(), "cancel", move |event| {
            event.prevent_default();
            exit();
        });
    }

    {
        let (dialog, start, playback) = (dialog.clone(), start.clone(), playback.clone());
        on(playback.clone().as_ref(), "click", move |_| {
            if !dialog.has_attribute("data-scene-ready") || dialog.has_attribute("data-closing") {
                return;
            }
            if playing.replace(false) {
                stop_scene(&dialog);
                set(&playback, "aria-label", "Play animation");
                set(&playback, "data-paused", "");
            } else {
                start();
            }
        });
    }
    if let Some(retry) = query("[data-manifold-retry]") {
        let (dialog, start) = (dialog.clone(), start.clone());
        on(retry.as_ref(), "click", move |_| {
            if dialog.has_attribute("data-closing") {
                return;
            }
            start();
            if let Some(close) = query(".manifold-close") {
                focus(&close);
            }
        });
    }
    if let Some(media) = media {
        let (dialog, start) = (dialog.clone(), start.clone());
        on(media.as_ref(), "change", move |_| {
            if dialog.open() && !dialog.has_attribute("data-closing") {
                start();
            }
        });
    }
    {
        let dlg = dialog.clone();
        on(dialog.as_ref(), "click", move |event| {
            if event.target().as_ref() == Some(dlg.as_ref()) {
                exit();
            }
        });
    }
    on(dialog.as_ref(), "close", move |_| finalize());
    if window().location().hash().ok().as_deref() == Some("#manifold") {
        show(None);
    }
}

pub fn init() {
    if let Some(index) = query("#index-overlay") {
        desktop_tabs();
        layer(index, all("#index-trigger"), "#index-close", "index-open");
    }
    if let Some(sheet) = query("[data-mobile-sheet='index']") {
        mobile_tabs("data-mobile-world", "data-mobile-world-panel");
        layer(
            sheet,
            all("[data-mobile-open='index']"),
            "[data-mobile-close]",
            "mobile-sheet-open",
        );
    }
    if let Some(sheet) = query("[data-mobile-site-index]") {
        mobile_tabs("data-mobile-site-world", "data-mobile-site-world-panel");
        layer(
            sheet,
            all("[data-mobile-site-index-open]"),
            "[data-mobile-site-index-close]",
            "mobile-site-index-open",
        );
    }
    manifold();
}
