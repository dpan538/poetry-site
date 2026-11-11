mod scene;

use scene::{Color, PERIOD, Scene};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{Clamped, JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

const CELL: usize = 5;
const BAYER: [f32; 16] = [
    0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.,
];
const RADII: [f32; 4] = [0.75, 1.20, 1.70, 2.25];

fn dot_mask(radius: f32, center: [f32; 2]) -> [f32; CELL * CELL] {
    std::array::from_fn(|i| {
        let px = (i % CELL) as f32 + 0.5 - center[0];
        let py = (i / CELL) as f32 + 0.5 - center[1];
        scene::smooth(radius + 0.42, radius - 0.42, (px * px + py * py).sqrt())
    })
}

fn context(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas 2D is unavailable"))?
        .dyn_into()
        .map_err(Into::into)
}

struct Renderer {
    ctx: CanvasRenderingContext2d,
    scene: Scene,
    pixels: Vec<u8>,
    previous: Vec<[u8; 4]>,
    masks: [[f32; CELL * CELL]; 4],
    cols: usize,
    rows: usize,
    entrance: bool,
}

impl Renderer {
    fn draw(&mut self, seconds: f32) -> Result<(), JsValue> {
        self.scene.prepare_frame(seconds);
        let (width, height) = (self.cols * CELL, self.rows * CELL);
        self.pixels.resize(width * height * 4, 255);
        let settle = if self.entrance {
            (1.0 - seconds / 1.2).max(0.0).powi(2)
        } else {
            0.0
        };
        for y in 0..self.rows {
            for x in 0..self.cols {
                let color = self.scene.color(
                    (x as f32 + 0.5) / self.cols as f32,
                    (y as f32 + 0.5) / self.rows as f32,
                    seconds,
                );
                let brightness = color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722;
                let dither = BAYER[(y & 3) * 4 + (x & 3)] / 16.0 - 0.47;
                let level = (brightness.sqrt() * 3.0 + dither).round().clamp(0.0, 3.0) as usize;
                let encoded = color.map(|c| (c * 255.0).clamp(0.0, 255.0).round() as u8);
                let color = encoded.map(|c| c as f32 / 255.0);
                let key = [
                    encoded[0],
                    encoded[1],
                    encoded[2],
                    level as u8 | (u8::from(settle > 0.0) << 7),
                ];
                let previous = &mut self.previous[y * self.cols + x];
                if settle == 0.0 && *previous == key {
                    continue;
                }
                *previous = key;
                let seed = scene::hash(x as u32, y as u32);
                let center_x = 2.5 + (seconds * 11.0 + seed * 17.0).sin() * 0.42 * settle;
                let center_y = 2.5 + (seconds * 9.0 + seed * 31.0).cos() * 0.42 * settle;
                let dots = if settle > 0.0 {
                    dot_mask(RADII[level], [center_x, center_y])
                } else {
                    self.masks[level]
                };
                let coverage = dots.iter().sum::<f32>() / (CELL * CELL) as f32;
                let peak = color.iter().copied().fold(0.0_f32, f32::max);
                let contrast = ((1.0 / peak.max(0.01) - 1.0) / (1.0 - coverage)).clamp(0.08, 0.95);
                for py in 0..CELL {
                    for px in 0..CELL {
                        let gain = 1.0 + (dots[py * CELL + px] - coverage) * contrast;
                        let index = ((y * CELL + py) * width + x * CELL + px) * 4;
                        for c in 0..3 {
                            self.pixels[index + c] =
                                (color[c] * gain * 255.0).clamp(0.0, 255.0).round() as u8;
                        }
                    }
                }
            }
        }
        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.pixels),
            width as u32,
            height as u32,
        )?;
        self.ctx.put_image_data(&image, 0.0, 0.0)
    }
}

type Frame = Closure<dyn FnMut(f64)>;

struct Player {
    renderer: RefCell<Renderer>,
    elapsed: Cell<f32>,
    paused: Cell<bool>,
    last: Cell<Option<f64>>,
    pending: Cell<Option<i32>>,
    callback: RefCell<Option<Frame>>,
}

impl Player {
    fn request(&self) {
        if self.paused.get() || self.pending.get().is_some() {
            return;
        }
        if let (Some(window), Some(callback)) = (web_sys::window(), self.callback.borrow().as_ref())
        {
            self.pending.set(
                window
                    .request_animation_frame(callback.as_ref().unchecked_ref())
                    .ok(),
            );
        }
    }

    fn pause(&self) {
        self.paused.set(true);
        self.last.set(None);
        if let (Some(window), Some(id)) = (web_sys::window(), self.pending.take()) {
            let _ = window.cancel_animation_frame(id);
        }
    }

    fn tick(&self, now: f64) {
        self.pending.set(None);
        if self.paused.get() {
            return;
        }
        let last = self.last.get().unwrap_or(now);
        if self.last.get().is_none() {
            self.last.set(Some(now));
        }
        let delta = now - last;
        if delta >= 1000.0 / 15.0 {
            self.last.set(Some(now));
            // Exclude background-tab time instead of jumping ahead when resuming.
            if delta < 500.0 {
                self.elapsed.set(self.elapsed.get() + delta as f32 / 1000.0);
            }
            let _ = self.renderer.borrow_mut().draw(self.elapsed.get());
        }
        self.request();
    }
}

#[wasm_bindgen]
pub struct OriginalStudy {
    player: Rc<Player>,
}

#[wasm_bindgen]
impl OriginalStudy {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, kind: u8, narrow: bool) -> Result<Self, JsValue> {
        if kind >= 6 {
            return Err(JsValue::from_str("Unknown original study"));
        }
        let (cols, rows) = if narrow { (102, 60) } else { (170, 100) };
        canvas.set_width((cols * CELL) as u32);
        canvas.set_height((rows * CELL) as u32);
        let player = Rc::new(Player {
            renderer: RefCell::new(Renderer {
                ctx: context(&canvas)?,
                scene: Scene::new(kind),
                pixels: Vec::new(),
                previous: vec![[255; 4]; cols * rows],
                masks: RADII.map(|radius| dot_mask(radius, [2.5, 2.5])),
                cols,
                rows,
                entrance: true,
            }),
            elapsed: Cell::new(0.0),
            paused: Cell::new(true),
            last: Cell::new(None),
            pending: Cell::new(None),
            callback: RefCell::new(None),
        });
        let weak = Rc::downgrade(&player);
        *player.callback.borrow_mut() = Some(Closure::new(move |now| {
            if let Some(player) = weak.upgrade() {
                player.tick(now);
            }
        }));
        player.renderer.borrow_mut().draw(0.0)?;
        Ok(Self { player })
    }

    pub fn play(&self) {
        self.player.paused.set(false);
        self.player.last.set(None);
        self.player.request();
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn seek(&self, seconds: f32) -> Result<(), JsValue> {
        if !seconds.is_finite() {
            return Err(JsValue::from_str("Invalid time"));
        }
        self.player.elapsed.set(seconds.clamp(0.0, PERIOD));
        self.player.last.set(None);
        let mut renderer = self.player.renderer.borrow_mut();
        renderer.entrance = false;
        renderer.draw(self.player.elapsed.get())
    }

    pub fn elapsed(&self) -> f32 {
        self.player.elapsed.get().rem_euclid(PERIOD)
    }

    pub fn source(&self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        let (width, height) = (680, 400);
        canvas.set_width(width);
        canvas.set_height(height);
        let mut renderer = self.player.renderer.borrow_mut();
        let scene = &mut renderer.scene;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        let time = if scene.kind == 4 { 8.0 } else { 0.0 };
        scene.prepare_frame(time);
        for y in 0..height {
            for x in 0..width {
                let color: Color = scene.color(
                    (x as f32 + 0.5) / width as f32,
                    (y as f32 + 0.5) / height as f32,
                    time,
                );
                pixels.extend(color.map(|c| (c * 255.0).clamp(0.0, 255.0) as u8));
                pixels.push(255);
            }
        }
        let image = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&pixels), width, height)?;
        context(&canvas)?.put_image_data(&image, 0.0, 0.0)
    }
}

impl Drop for OriginalStudy {
    fn drop(&mut self) {
        self.player.pause();
    }
}

fn published_kind(slug: &str) -> Option<u8> {
    match slug {
        "private-blue" => Some(1),
        "dusk-gauze" => Some(2),
        "crown-tiles" => Some(3),
        "drowned-water" => Some(5),
        _ => None,
    }
}

/// Only reviewed scenes can be mounted by the publication. Study numbers stay
/// local to the review gallery; the site uses stable catalogue identifiers.
#[wasm_bindgen]
pub fn mount_original(canvas: HtmlCanvasElement, slug: &str) -> Result<OriginalStudy, JsValue> {
    let kind = published_kind(slug)
        .ok_or_else(|| JsValue::from_str("Unknown published MANIFOLD original"))?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing window"))?;
    let narrow = window
        .match_media("(max-width: 700px)")?
        .is_some_and(|m| m.matches());
    let reduced = window
        .match_media("(prefers-reduced-motion: reduce)")?
        .is_some_and(|m| m.matches());
    let animation = OriginalStudy::new(canvas, kind, narrow)?;
    if reduced {
        animation.seek(0.0)?;
    } else {
        animation.play();
    }
    Ok(animation)
}

#[cfg(test)]
mod publication_tests {
    use super::published_kind;

    #[test]
    fn publication_exposes_only_the_four_approved_studies() {
        for (slug, kind) in [
            ("private-blue", 1),
            ("dusk-gauze", 2),
            ("crown-tiles", 3),
            ("drowned-water", 5),
        ] {
            assert_eq!(published_kind(slug), Some(kind));
        }
        for unreviewed in ["red-current", "night-room", "unknown"] {
            assert_eq!(published_kind(unreviewed), None);
        }
    }
}
