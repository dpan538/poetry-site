mod photo;
mod scenes;

use scenes::{COLS, PAINTINGS, ROWS, Study};
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, MediaQueryList, Window,
};

const CELL: u32 = 5;
const MARKS: [f64; 5] = [0.47, 0.83, 1.20, 1.64, 2.24];
const HUES: usize = 24;
const SATURATIONS: usize = 9;
const VALUES: usize = 24;
const TONE_POWER: f32 = 1.45;
const ATLAS_COLUMNS: usize = 126;
const SPRITES: usize = HUES * SATURATIONS * VALUES * MARKS.len();

fn context(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas 2D is unavailable"))?
        .dyn_into()
        .map_err(Into::into)
}

fn atlas_cell(sprite: usize) -> (f64, f64) {
    let cell = f64::from(CELL);
    (
        (sprite % ATLAS_COLUMNS) as f64 * cell,
        (sprite / ATLAS_COLUMNS) as f64 * cell,
    )
}

fn glyph_atlas(document: &Document, max_contrast: f64) -> Result<HtmlCanvasElement, JsValue> {
    let atlas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
    atlas.set_width(ATLAS_COLUMNS as u32 * CELL);
    atlas.set_height(SPRITES.div_ceil(ATLAS_COLUMNS) as u32 * CELL);
    let ctx = context(&atlas)?;
    let cell = f64::from(CELL);
    let half = cell / 2.0;
    for hue in 0..HUES {
        for saturation in 0..SATURATIONS {
            for value in 0..VALUES {
                let pigment = |tone: f64| {
                    let [r, g, b] = scenes::hsv_to_rgb(
                        hue as f32 / HUES as f32,
                        saturation as f32 / (SATURATIONS - 1) as f32,
                        tone as f32,
                    );
                    format!("#{r:02x}{g:02x}{b:02x}")
                };
                let color = (hue * SATURATIONS + saturation) * VALUES + value;
                let tone = f64::from((value as f32 / (VALUES - 1) as f32).powf(TONE_POWER));
                for (mark, radius) in MARKS.iter().enumerate() {
                    let (x, y) = atlas_cell(color * MARKS.len() + mark);
                    // Balance the bright mark and tinted ground by their areas so dot size
                    // can animate without changing the painting's overall exposure.
                    let coverage = std::f64::consts::PI * (radius / cell).powi(2);
                    let contrast = max_contrast
                        .min(0.42 * tone / coverage)
                        .min((1.0 - tone) / (1.0 - coverage));
                    ctx.set_fill_style_str(&pigment(tone - coverage * contrast));
                    ctx.fill_rect(x, y, cell, cell);
                    ctx.set_fill_style_str(&pigment(tone + (1.0 - coverage) * contrast));
                    ctx.begin_path();
                    ctx.arc(x + half, y + half, *radius, 0.0, std::f64::consts::TAU)?;
                    ctx.fill();
                }
            }
        }
    }
    Ok(atlas)
}

thread_local! {
    static ATLAS: OnceCell<HtmlCanvasElement> = const { OnceCell::new() };
    static SOFT_ATLAS: OnceCell<HtmlCanvasElement> = const { OnceCell::new() };
}

fn shared_atlas(document: &Document, soft: bool) -> Result<HtmlCanvasElement, JsValue> {
    let (atlas, contrast) = if soft {
        (&SOFT_ATLAS, 0.23)
    } else {
        (&ATLAS, 0.46)
    };
    atlas.with(|slot| {
        if let Some(atlas) = slot.get() {
            return Ok(atlas.clone());
        }
        let atlas = glyph_atlas(document, contrast)?;
        let _ = slot.set(atlas.clone());
        Ok(atlas)
    })
}

fn sprite(ink: scenes::Ink) -> u16 {
    let hue = (ink.hue * HUES as f32).round() as usize % HUES;
    let saturation = (ink.saturation * (SATURATIONS - 1) as f32).round() as usize;
    let value = (ink.value.powf(TONE_POWER.recip()) * (VALUES - 1) as f32).round() as usize;
    let mark = ((ink.density * MARKS.len() as f32) as usize).min(MARKS.len() - 1);
    (((hue * SATURATIONS + saturation) * VALUES + value) * MARKS.len() + mark) as u16
}

struct Renderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    atlas: HtmlCanvasElement,
    study: Study,
    previous: Vec<u16>,
    grain: Vec<f32>,
    narrow: bool,
    photo_pixels: Vec<u8>,
    entrance_motion: bool,
}

impl Renderer {
    fn draw(&mut self, elapsed: f32) {
        let painting = self.study.painting();
        let (cols, rows) = painting.grid(self.narrow);
        if self.canvas.width() != cols as u32 * CELL
            || self.canvas.height() != rows as u32 * CELL
            || self.previous.len() != cols * rows
        {
            self.canvas.set_width(cols as u32 * CELL);
            self.canvas.set_height(rows as u32 * CELL);
            self.previous.resize(cols * rows, u16::MAX);
            self.previous.fill(u16::MAX);
        }
        if painting.source.is_empty() {
            self.draw_photo(cols, rows, elapsed);
            return;
        }
        let cell = f64::from(CELL);
        let scale_x = COLS as f32 / cols as f32;
        let scale_y = painting.rows() as f32 / rows as f32;
        let exposure = self.study.exposure(elapsed);
        for (index, previous) in self.previous.iter_mut().enumerate() {
            let (x, y) = (index % cols, index / cols);
            let x = (x as f32 + 0.5) * scale_x - 0.5;
            let y = (y as f32 + 0.5) * scale_y - 0.5;
            let strength = self.study.motion_strength(x, y);
            if strength == 0.0 && *previous != u16::MAX {
                continue;
            }
            let next = sprite(
                self.study
                    .ink(x, y, elapsed, exposure, self.grain[index], strength),
            );
            if next == *previous {
                continue;
            }
            let (sx, sy) = atlas_cell(next as usize);
            let _ = self
                .ctx
                .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &self.atlas,
                    sx,
                    sy,
                    cell,
                    cell,
                    (index % cols) as f64 * cell,
                    (index / cols) as f64 * cell,
                    cell,
                    cell,
                );
            *previous = next;
        }
    }
}

type FrameCallback = Closure<dyn FnMut(f64)>;

struct Player {
    window: Window,
    renderer: RefCell<Renderer>,
    elapsed: Cell<f32>,
    paused: Cell<bool>,
    pending: Cell<Option<i32>>,
    last_frame: Cell<Option<f64>>,
    callback: RefCell<Option<FrameCallback>>,
}

impl Player {
    fn request_frame(&self) {
        if self.paused.get() || self.pending.get().is_some() {
            return;
        }
        if let Some(callback) = self.callback.borrow().as_ref() {
            self.pending.set(
                self.window
                    .request_animation_frame(callback.as_ref().unchecked_ref())
                    .ok(),
            );
        }
    }

    fn set_paused(&self, paused: bool) {
        self.paused.set(paused);
        self.last_frame.set(None);
        if paused {
            if let Some(frame) = self.pending.take() {
                let _ = self.window.cancel_animation_frame(frame);
            }
        } else {
            self.request_frame();
        }
    }

    fn tick(&self, timestamp: f64) {
        self.pending.set(None);
        if self.paused.get() {
            return;
        }
        let interval = timestamp - self.last_frame.get().unwrap_or(timestamp);
        if interval >= 62.5 {
            self.last_frame.set(Some(timestamp));
            self.elapsed
                .set(self.elapsed.get() + interval.min(100.) as f32 / 1000.);
            self.renderer.borrow_mut().draw(self.elapsed.get());
        } else if self.last_frame.get().is_none() {
            self.last_frame.set(Some(timestamp));
        }
        self.request_frame();
    }

    fn select(
        &self,
        document: &Document,
        painting: &'static scenes::Painting,
    ) -> Result<(), JsValue> {
        let atlas = shared_atlas(document, painting.soft)?;
        self.elapsed.set(0.);
        self.last_frame.set(None);
        let mut renderer = self.renderer.borrow_mut();
        renderer.atlas = atlas;
        renderer.study = Study::new(painting);
        renderer.previous.fill(u16::MAX);
        renderer.draw(0.);
        Ok(())
    }
}

#[wasm_bindgen]
pub struct PaintingPlayback {
    player: Rc<Player>,
    narrow: MediaQueryList,
    resize: Closure<dyn FnMut(Event)>,
}

impl Drop for PaintingPlayback {
    fn drop(&mut self) {
        self.player.set_paused(true);
        let _ = self
            .narrow
            .remove_event_listener_with_callback("change", self.resize.as_ref().unchecked_ref());
    }
}

impl PaintingPlayback {
    fn new(
        canvas: HtmlCanvasElement,
        painting: &'static scenes::Painting,
    ) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("Window is unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("Document is unavailable"))?;
        let paused = window
            .match_media("(prefers-reduced-motion: reduce)")?
            .is_some_and(|query| query.matches());
        let narrow = window
            .match_media("(max-width: 600px)")?
            .ok_or_else(|| JsValue::from_str("Viewport media query is unavailable"))?;
        let player = Rc::new(Player {
            window,
            renderer: RefCell::new(Renderer {
                ctx: context(&canvas)?,
                canvas,
                atlas: shared_atlas(&document, painting.soft)?,
                study: Study::new(painting),
                previous: Vec::with_capacity(COLS * ROWS),
                grain: (0..(COLS * ROWS) as u32).map(scenes::noise).collect(),
                narrow: narrow.matches(),
                photo_pixels: Vec::new(),
                entrance_motion: !paused,
            }),
            elapsed: Cell::new(0.),
            paused: Cell::new(paused),
            pending: Cell::new(None),
            last_frame: Cell::new(None),
            callback: RefCell::new(None),
        });
        let weak = Rc::downgrade(&player);
        *player.callback.borrow_mut() = Some(Closure::new(move |timestamp| {
            if let Some(player) = weak.upgrade() {
                player.tick(timestamp);
            }
        }));
        let resize = Closure::<dyn FnMut(Event)>::new({
            let (player, narrow) = (player.clone(), narrow.clone());
            move |_| {
                let mut renderer = player.renderer.borrow_mut();
                renderer.narrow = narrow.matches();
                renderer.draw(player.elapsed.get());
            }
        });
        narrow.add_event_listener_with_callback("change", resize.as_ref().unchecked_ref())?;
        player.renderer.borrow_mut().draw(0.);
        player.request_frame();
        Ok(Self {
            player,
            narrow,
            resize,
        })
    }
}

#[wasm_bindgen]
pub fn mount_painting(canvas: HtmlCanvasElement, slug: &str) -> Result<PaintingPlayback, JsValue> {
    let painting = PAINTINGS
        .iter()
        .find(|painting| painting.piece == slug)
        .ok_or_else(|| JsValue::from_str("Unknown MANIFOLD painting"))?;
    PaintingPlayback::new(canvas, painting)
}

fn lab_credit(description: &Element, original: &Element, painting: &scenes::Painting) {
    description.set_text_content(Some(painting.caption));
    if painting.source.is_empty() {
        let _ = original.remove_attribute("href");
        let _ = original.set_attribute("hidden", "");
    } else {
        let _ = original.set_attribute("href", painting.source);
        let _ = original.remove_attribute("hidden");
    }
}

#[wasm_bindgen]
pub fn start_lab() -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("Document is unavailable"))?;
    let element = |id| {
        document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(id))
    };
    let canvas: HtmlCanvasElement = element("scene")?.dyn_into()?;
    let description = element("description")?;
    let original = element("original")?;
    let playback = element("playback")?;
    let studies = element("study-scenes")?;
    let first = PAINTINGS
        .first()
        .ok_or_else(|| JsValue::from_str("Missing painting catalogue"))?;
    let animation = Rc::new(PaintingPlayback::new(canvas, first)?);
    let selected = Rc::new(Cell::new(0));
    let buttons: Vec<Element> = PAINTINGS
        .iter()
        .enumerate()
        .map(|(i, painting)| {
            let button = document.create_element("button")?;
            button.set_attribute("type", "button")?;
            button.set_attribute("data-scene", &i.to_string())?;
            button.set_attribute("aria-pressed", if i == 0 { "true" } else { "false" })?;
            button.set_text_content(Some(&format!(
                "{:02} · {}",
                painting.number, painting.label
            )));
            studies.append_child(&button)?;
            Ok(button)
        })
        .collect::<Result<_, JsValue>>()?;
    for (i, button) in buttons.iter().enumerate() {
        let (animation, buttons, description, original, selected, document) = (
            animation.clone(),
            buttons.clone(),
            description.clone(),
            original.clone(),
            selected.clone(),
            document.clone(),
        );
        let select = Closure::<dyn FnMut(Event)>::new(move |_| {
            let previous = selected.get();
            let painting = &PAINTINGS[i];
            if previous == i || animation.player.select(&document, painting).is_err() {
                return;
            }
            selected.set(i);
            let _ = buttons[previous].set_attribute("aria-pressed", "false");
            let _ = buttons[i].set_attribute("aria-pressed", "true");
            lab_credit(&description, &original, painting);
        });
        button.add_event_listener_with_callback("click", select.as_ref().unchecked_ref())?;
        select.forget();
    }
    let toggle = Closure::<dyn FnMut(Event)>::new({
        let (animation, playback) = (animation.clone(), playback.clone());
        move |_| {
            let paused = !animation.player.paused.get();
            animation.player.set_paused(paused);
            playback.set_text_content(Some(if paused { "Play" } else { "Pause" }));
        }
    });
    playback.add_event_listener_with_callback("click", toggle.as_ref().unchecked_ref())?;
    toggle.forget();

    lab_credit(&description, &original, first);
    playback.set_text_content(Some(if animation.player.paused.get() {
        "Play"
    } else {
        "Pause"
    }));
    Ok(())
}
