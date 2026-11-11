mod glyph;
mod scene;

use glyph::{CHARS, Cell, Encoder};
use scene::Scene;
use std::{
    cell::{Cell as Flag, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
fn context(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas 2D unavailable"))?
        .dyn_into()
        .map_err(Into::into)
}
fn css(color: scene::Color) -> String {
    format!(
        "rgb({},{},{})",
        (color[0] * 255.) as u8,
        (color[1] * 255.) as u8,
        (color[2] * 255.) as u8
    )
}
struct Renderer {
    ctx: CanvasRenderingContext2d,
    atlas: HtmlCanvasElement,
    scene: Scene,
    encoder: Encoder,
    previous: Vec<Cell>,
    frame: Vec<Cell>,
    counts: [u32; 4],
    entrance: bool,
    x_edges: Vec<u32>,
    y_edges: Vec<u32>,
    slot_w: u32,
    slot_h: u32,
}
impl Renderer {
    fn new(canvas: HtmlCanvasElement, kind: u8, narrow: bool) -> Result<Self, JsValue> {
        let (cols, rows) = match (kind == 5, narrow) {
            (true, true) => (119, 70),
            (true, false) => (204, 120),
            (false, true) => (136, 80),
            (false, false) => (255, 150),
        };
        let scene = Scene::new(kind, cols, rows);
        let encoder = Encoder::new(&scene.palette, cols);
        let dpr = web_sys::window()
            .map(|w| w.device_pixel_ratio())
            .unwrap_or(1.);
        let css_width = canvas
            .client_width()
            .max(
                canvas
                    .parent_element()
                    .map(|e| e.client_width())
                    .unwrap_or(0),
            )
            .max(1) as f64;
        let width = (css_width * dpr).round() as u32;
        let height = (width as f64 * rows as f64 / cols as f64).round() as u32;
        canvas.set_width(width);
        canvas.set_height(height);
        let cell_w = width as f64 / cols as f64;
        let cell_h = height as f64 / rows as f64;
        let slot_w = cell_w.ceil() as u32;
        let slot_h = cell_h.ceil() as u32;
        let x_edges = (0..=cols)
            .map(|x| (x as f64 * cell_w).round() as u32)
            .collect();
        let y_edges = (0..=rows)
            .map(|y| (y as f64 * cell_h).round() as u32)
            .collect();
        let ctx = context(&canvas)?;
        ctx.set_image_smoothing_enabled(false);
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| JsValue::from_str("Missing document"))?;
        let atlas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        atlas.set_width(CHARS.len() as u32 * slot_w);
        atlas.set_height(scene.palette.len() as u32 * slot_h);
        let ac = context(&atlas)?;
        ac.set_fill_style_str(&css(scene.ground));
        ac.fill_rect(0., 0., atlas.width() as f64, atlas.height() as f64);
        for (ink, color) in scene.palette.iter().enumerate() {
            ac.set_fill_style_str(&css(*color));
            for glyph in 1..CHARS.len() {
                let (x, y) = ((glyph as u32 * slot_w) as f64, (ink as u32 * slot_h) as f64);
                // The actual upstream monospace masks, rather than hand-drawn
                // circles. Preserve font baseline and device-pixel cell sizing.
                ac.set_font(&format!(
                    "{}px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
                    cell_w / 0.6
                ));
                ac.set_text_align("center");
                ac.set_text_baseline("middle");
                ac.save();
                ac.begin_path();
                ac.rect(x, y, slot_w as f64, slot_h as f64);
                ac.clip();
                ac.fill_text(
                    &CHARS[glyph].to_string(),
                    x + slot_w as f64 / 2.,
                    y + slot_h as f64 / 2.,
                )?;
                ac.restore();
            }
        }
        ctx.set_fill_style_str(&css(scene.ground));
        ctx.fill_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
        Ok(Self {
            ctx,
            atlas,
            scene,
            encoder,
            previous: vec![
                Cell {
                    glyph: 255,
                    ink: 255
                };
                cols * rows
            ],
            frame: vec![Cell::default(); cols * rows],
            counts: [0; 4],
            entrance: true,
            x_edges,
            y_edges,
            slot_w,
            slot_h,
        })
    }
    fn draw(&mut self, t: f32) -> Result<(), JsValue> {
        self.scene.prepare(t);
        let settle = if self.entrance {
            (1. - t / 1.2).max(0.)
        } else {
            0.
        };
        self.counts.fill(0);
        for y in 0..self.scene.rows {
            for x in 0..self.scene.cols {
                let mut sample = self.scene.sample(x, y, t);
                if settle > 0. {
                    let stagger = ((x * 13 + y * 7) % 31) as f32 / 31.;
                    sample.opacity *= 1. - settle.powi(2) * (0.6 + stagger * 0.4);
                }
                let cell = self.encoder.encode(sample, x, y);
                let at = y * self.scene.cols + x;
                self.frame[at] = cell;
                self.counts[cell.glyph as usize] += 1;
                if self.previous[at] == cell {
                    continue;
                }
                self.previous[at] = cell;
                let dx = self.x_edges[x];
                let dy = self.y_edges[y];
                let dw = self.x_edges[x + 1] - dx;
                let dh = self.y_edges[y + 1] - dy;
                if dw == 0 || dh == 0 {
                    continue;
                }
                // Copy whole device pixels, without resizing the full Canvas
                // texture in CSS. Fractional resampling used to create moire.
                self.ctx
                    .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &self.atlas,
                        (cell.glyph as u32 * self.slot_w + (self.slot_w - dw) / 2) as f64,
                        (cell.ink as u32 * self.slot_h + (self.slot_h - dh) / 2) as f64,
                        dw as f64,
                        dh as f64,
                        dx as f64,
                        dy as f64,
                        dw as f64,
                        dh as f64,
                    )?;
            }
        }
        Ok(())
    }
}
type Frame = Closure<dyn FnMut(f64)>;
struct Player {
    renderer: RefCell<Renderer>,
    elapsed: Flag<f32>,
    paused: Flag<bool>,
    last: Flag<Option<f64>>,
    pending: Flag<Option<i32>>,
    callback: RefCell<Option<Frame>>,
}
impl Player {
    fn request(&self) {
        if self.paused.get() || self.pending.get().is_some() {
            return;
        }
        if let (Some(w), Some(cb)) = (web_sys::window(), self.callback.borrow().as_ref()) {
            self.pending
                .set(w.request_animation_frame(cb.as_ref().unchecked_ref()).ok());
        }
    }
    fn pause(&self) {
        self.paused.set(true);
        self.last.set(None);
        if let (Some(w), Some(id)) = (web_sys::window(), self.pending.take()) {
            let _ = w.cancel_animation_frame(id);
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
        let dt = now - last;
        if dt >= 1000. / 15. {
            self.last.set(Some(now));
            if dt < 500. {
                self.elapsed.set(self.elapsed.get() + dt as f32 / 1000.);
            }
            let _ = self.renderer.borrow_mut().draw(self.elapsed.get());
        }
        self.request();
    }
}
#[wasm_bindgen]
pub struct SceneStudy {
    player: Rc<Player>,
}
#[wasm_bindgen]
impl SceneStudy {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, kind: u8, narrow: bool) -> Result<Self, JsValue> {
        if kind >= 6 || kind == 2 {
            return Err(JsValue::from_str("Unknown study"));
        }
        let player = Rc::new(Player {
            renderer: RefCell::new(Renderer::new(canvas, kind, narrow)?),
            elapsed: Flag::new(0.),
            paused: Flag::new(true),
            last: Flag::new(None),
            pending: Flag::new(None),
            callback: RefCell::new(None),
        });
        let weak = Rc::downgrade(&player);
        *player.callback.borrow_mut() = Some(Closure::new(move |now| {
            if let Some(p) = weak.upgrade() {
                p.tick(now)
            }
        }));
        player.renderer.borrow_mut().draw(0.)?;
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
    pub fn seek(&self, t: f32) -> Result<(), JsValue> {
        if !t.is_finite() {
            return Err(JsValue::from_str("Invalid time"));
        }
        self.player.elapsed.set(t.max(0.));
        self.player.last.set(None);
        let mut r = self.player.renderer.borrow_mut();
        r.entrance = false;
        r.draw(self.player.elapsed.get())
    }
    pub fn elapsed(&self) -> f32 {
        self.player.elapsed.get()
    }
    pub fn stats(&self) -> String {
        format!("{:?}", self.player.renderer.borrow().counts)
    }
    pub fn ascii(&self) -> String {
        let r = self.player.renderer.borrow();
        let mut out = String::with_capacity(r.frame.len() * 2);
        for row in r.frame.chunks(r.scene.cols) {
            for c in row {
                out.push(CHARS[c.glyph as usize]);
            }
            out.push('\n');
        }
        out
    }
    pub fn atlas(&self, target: HtmlCanvasElement) -> Result<(), JsValue> {
        let r = self.player.renderer.borrow();
        target.set_width(r.atlas.width());
        target.set_height(r.atlas.height());
        let ctx = context(&target)?;
        ctx.set_image_smoothing_enabled(false);
        ctx.draw_image_with_html_canvas_element(&r.atlas, 0., 0.)
    }
    pub fn palette(&self) -> String {
        self.player
            .renderer
            .borrow()
            .scene
            .palette
            .iter()
            .copied()
            .map(css)
            .collect::<Vec<_>>()
            .join(";")
    }
}
impl Drop for SceneStudy {
    fn drop(&mut self) {
        self.player.pause();
    }
}

fn published_kind(slug: &str) -> Option<u8> {
    match slug {
        "homeward-bridge" => Some(0),
        "observer-supper" => Some(1),
        "silver-lake" => Some(3),
        "sleeping-courtyard" => Some(4),
        "iron-shrine" => Some(5),
        _ => None,
    }
}

/// Stable publication identifiers exclude the withdrawn study 03.
#[wasm_bindgen]
pub fn mount_atmosphere(canvas: HtmlCanvasElement, slug: &str) -> Result<SceneStudy, JsValue> {
    let kind = published_kind(slug)
        .ok_or_else(|| JsValue::from_str("Unknown published MANIFOLD atmosphere"))?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing window"))?;
    let narrow = window
        .match_media("(max-width: 700px)")?
        .is_some_and(|m| m.matches());
    let reduced = window
        .match_media("(prefers-reduced-motion: reduce)")?
        .is_some_and(|m| m.matches());
    let animation = SceneStudy::new(canvas, kind, narrow)?;
    if reduced {
        animation.seek(0.)?;
    } else {
        animation.play();
    }
    Ok(animation)
}

#[cfg(test)]
mod publication_tests {
    use super::published_kind;

    #[test]
    fn only_the_five_approved_atmospheres_have_publication_ids() {
        for (slug, kind) in [
            ("homeward-bridge", 0),
            ("observer-supper", 1),
            ("silver-lake", 3),
            ("sleeping-courtyard", 4),
            ("iron-shrine", 5),
        ] {
            assert_eq!(published_kind(slug), Some(kind));
        }
        for unknown in ["03", "withdrawn", "unknown"] {
            assert_eq!(published_kind(unknown), None);
        }
    }
}
