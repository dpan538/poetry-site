use crate::{mobile, query, window};
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Animation, Element};

fn reduced() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|media| media.matches())
}

pub fn opacity(target: &Element) -> f64 {
    window()
        .get_computed_style(target)
        .ok()
        .flatten()
        .and_then(|style| style.get_property_value("opacity").ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(1.0)
}

pub fn tween(
    target: &Element,
    from: f64,
    to: f64,
    rise: Option<u8>,
    durations: [u16; 2],
) -> Option<Animation> {
    if reduced() {
        return None;
    }
    let frames = Object::new();
    let opacity = Array::of2(&from.into(), &to.into());
    let _ = Reflect::set(&frames, &"opacity".into(), &opacity);
    if let Some(rise) = rise {
        let start = JsValue::from_str(&format!("translateY({rise}px)"));
        let transform = Array::of2(&start, &"translateY(0px)".into());
        let _ = Reflect::set(&frames, &"transform".into(), &transform);
    }
    let (duration, easing) = if mobile() {
        (durations[1], "cubic-bezier(.32,.12,.22,1)")
    } else {
        (durations[0], "cubic-bezier(.25,.46,.45,.94)")
    };
    let fill = if to == 0.0 { "both" } else { "backwards" };
    animate(target, &frames, duration, 0, easing, fill)
}

pub fn reveal_index(target: &Element, from: f64, duration: u16, delay: u16) -> Option<Animation> {
    if reduced() {
        return None;
    }
    let frames = Object::new();
    let opacity = Array::of2(&from.into(), &1.0.into());
    let _ = Reflect::set(&frames, &"opacity".into(), &opacity);
    animate(target, &frames, duration, delay, "linear", "backwards")
}

fn animate(
    target: &Element,
    frames: &Object,
    duration: u16,
    delay: u16,
    easing: &str,
    fill: &str,
) -> Option<Animation> {
    let animate = Reflect::get(target, &"animate".into())
        .ok()?
        .dyn_into::<Function>()
        .ok()?;
    let timing = Object::new();
    let _ = Reflect::set(&timing, &"duration".into(), &duration.into());
    let _ = Reflect::set(&timing, &"delay".into(), &delay.into());
    let _ = Reflect::set(&timing, &"easing".into(), &easing.into());
    let _ = Reflect::set(&timing, &"fill".into(), &fill.into());
    animate.call2(target, frames, &timing).ok()?.dyn_into().ok()
}

pub fn enter_page() {
    if let Some(page) = query(".world-index") {
        let _ = tween(&page, 0.72, 1.0, None, [460, 480]);
    }
}
