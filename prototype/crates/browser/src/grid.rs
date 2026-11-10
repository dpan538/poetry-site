use crate::{
    all, attr, document, element, elements, on_page as on, query, set, style, toggle, window, world,
};
use reader_core::layout::{
    Form, Layout, Rectangle, avoid_overlap, cadence, compose, fit_spread, mobile_breaks,
    mobile_pauses,
};
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    ops::Range,
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, Node};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Frame {
    col_start: f64,
    col_span: f64,
    row_start: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Preset {
    id: String,
    poem_types: Vec<String>,
    title_frame: Frame,
    body_frames: Vec<Frame>,
}

struct ReadingColumn {
    frame: Rectangle,
    blocks: Range<usize>,
    spread: usize,
}

fn block_heights(rack: &Element, width: f64, blocks: &[Element]) -> Vec<f64> {
    style(rack, "width", &px(width));
    let measuring: Vec<_> = blocks.iter().map(copy).collect();
    for block in &measuring {
        append(rack, block);
    }
    let heights = measuring
        .iter()
        .map(|block| block.get_bounding_client_rect().height())
        .collect();
    rack.set_text_content(None);
    heights
}

fn block_run_height(heights: &[f64], gaps: &[f64]) -> f64 {
    heights.iter().sum::<f64>()
        + gaps
            .iter()
            .take(heights.len().saturating_sub(1))
            .sum::<f64>()
}

fn line_offset(rack: &Element, width: f64, stanza: &Element, line: usize) -> f64 {
    style(rack, "width", &px(width));
    let stanza = copy(stanza);
    append(rack, &stanza);
    let top = stanza.get_bounding_client_rect().top();
    let offset = elements(&stanza, ".poem-line")
        .get(line)
        .map(|el| el.get_bounding_client_rect().top() - top)
        .unwrap_or(0.0);
    stanza.remove();
    offset
}

struct Grid {
    page: Element,
    canvas: Element,
    title: Element,
    title_text: Element,
    body: Element,
    blocks: Vec<Element>,
    lines: Vec<Vec<String>>,
    breaths: Vec<f64>,
    column_alignment: Option<[usize; 3]>,
    mobile: Vec<(Element, String)>,
    mobile_width: Cell<f64>,
    preset: Preset,
    title_size: &'static str,
    title_end: bool,
    border: bool,
    top_rule: bool,
    accent_first: bool,
    accent_extra: bool,
    prose: bool,
}

fn append(parent: &Element, child: &Element) {
    let _ = parent.append_child(child);
}
fn copy(source: &Element) -> Element {
    source
        .clone_node_with_deep(true)
        .expect("clone DOM block")
        .unchecked_into()
}
fn px(value: f64) -> String {
    format!("{value:.2}px")
}
fn number(text: &str, default: f64) -> f64 {
    text.trim()
        .trim_end_matches("px")
        .parse()
        .unwrap_or(default)
}

fn title_fits(title: &Element, lines: f64) -> bool {
    let Ok(text) = document().create_range() else {
        return true;
    };
    if text.select_node_contents(title).is_err() {
        return true;
    }
    let ink = text.get_bounding_client_rect();
    let frame = title.get_bounding_client_rect();
    let leading = window()
        .get_computed_style(title)
        .ok()
        .flatten()
        .and_then(|css| css.get_property_value("line-height").ok())
        .map_or(frame.height(), |value| number(&value, frame.height()));
    ink.left() >= frame.left() - 1.0
        && ink.right() <= frame.right() + 1.0
        && frame.height() <= leading * lines + 1.0
}

fn fit_title(title: &Element, lines: f64) {
    if title_fits(title, lines) {
        return;
    }
    let Some(size) = window()
        .get_computed_style(title)
        .ok()
        .flatten()
        .and_then(|css| css.get_property_value("font-size").ok())
    else {
        return;
    };
    let mut high = number(&size, 54.0);
    let mut low = 26.0_f64.min(high);
    while high - low > 0.5 {
        let size = (low + high) / 2.0;
        style(title, "font-size", &px(size));
        if title_fits(title, lines) {
            low = size;
        } else {
            high = size;
        }
    }
    style(title, "font-size", &px(low));
}

fn navigation_rails(canvas: &web_sys::DomRect) -> (f64, f64) {
    let padding = |el: &Element, property: &str| {
        window()
            .get_computed_style(el)
            .ok()
            .flatten()
            .and_then(|css| css.get_property_value(property).ok())
            .map(|v| number(&v, 0.0))
            .unwrap_or(0.0)
    };
    let left = query("#nav-prev")
        .map(|el| {
            let rect = el.get_bounding_client_rect();
            if rect.width() == 0.0 {
                return 0.0;
            }
            rect.right() - padding(&el, "padding-right") + 7.0 - canvas.left()
        })
        .unwrap_or(0.0);
    let right = query("#nav-next")
        .map(|el| {
            let rect = el.get_bounding_client_rect();
            if rect.width() == 0.0 {
                return canvas.width();
            }
            rect.left() + padding(&el, "padding-left") - 7.0 - canvas.left()
        })
        .unwrap_or(canvas.width());
    let rail = canvas.width() * 0.15;
    (
        left.clamp(0.0, rail),
        right.clamp(canvas.width() - rail, canvas.width()),
    )
}

struct Metrics {
    lines: usize,
    words: usize,
    chars: usize,
    stanzas: usize,
    class: &'static str,
}

fn metrics(lines: &[Vec<String>]) -> Metrics {
    let all: Vec<_> = lines.iter().flatten().map(String::as_str).collect();
    let n = all.len();
    let words: Vec<_> = all
        .iter()
        .flat_map(|l| l.split(|c: char| !c.is_ascii_alphabetic()))
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let word_count = words.len();
    let punctuation = |c: char| ".,;:!?—–".contains(c);
    let punct = all
        .iter()
        .flat_map(|l| l.chars())
        .filter(|&c| punctuation(c))
        .count();
    let ending = all
        .iter()
        .filter(|l| l.chars().last().is_some_and(punctuation))
        .count() as f64
        / n.max(1) as f64;
    let stops = "the a an and or but in on at to for of with is was i my your their this that it as by from into not be";
    let mut frequencies = BTreeMap::new();
    for w in &words {
        *frequencies.entry(w).or_insert(0) += 1;
    }
    let rep = frequencies
        .iter()
        .filter(|(w, c)| **c > 3 && !stops.split_whitespace().any(|s| s == w.as_str()))
        .count() as f64
        / 5.0;
    let unique = if word_count == 0 {
        1.0
    } else {
        frequencies.len() as f64 / word_count as f64
    };
    let pd = punct as f64 / word_count.max(1) as f64;
    let avg = word_count as f64 / n.max(1) as f64;
    let stanzas = lines.len();
    let stanza_avg = n as f64 / stanzas.max(1) as f64;
    let var = lines
        .iter()
        .map(|s| (s.len() as f64 - stanza_avg).powi(2))
        .sum::<f64>()
        / stanzas.max(1) as f64;
    let dialogue = all.iter().any(|l| l.contains(['“', '”', '"']));
    let stage = all.iter().any(|l| {
        l.split_once('(')
            .and_then(|(_, v)| v.split_once(')'))
            .is_some_and(|(inside, _)| inside.chars().count() >= 2)
    });
    let section = all.iter().any(|l| {
        l.trim()
            .strip_prefix('(')
            .and_then(|v| v.strip_suffix(')'))
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
    });
    let class = match () {
        _ if stage && stanzas > 3 => "collage",
        _ if section => "triptych",
        _ if dialogue && n > 15 && avg > 5.0 => "dramatic-narrative",
        _ if dialogue || stage => "dramatic-monologue",
        _ if ending > 0.6 && var < 2.0 && stanza_avg >= 3.0 => "hymn-structured",
        _ if rep > 0.5 && avg < 5.0 => "incantatory",
        _ if rep > 0.4 && var < 2.0 => "structured-repetitive",
        _ if n <= 8 && avg < 5.0 && unique > 0.8 => "minimal-aphoristic",
        _ if n <= 12 && avg < 5.0 => "minimal-fragment",
        _ if n <= 20 && pd < 0.1 && unique > 0.7 => "still-observation",
        _ if avg > 7.0 && n > 15 => "prose-lyric",
        _ if avg > 6.0 && stanzas <= 3 => "dense-linear",
        _ if pd > 0.15 && n > 20 => "lyric-dense",
        _ if unique > 0.75 && var > 3.0 => "lyric-fragmented",
        _ if n > 8 && n <= 25 && (4.0..=7.0).contains(&avg) => "meditative",
        _ if var > 5.0 => "hybrid",
        _ if var < 3.0 && stanza_avg >= 3.0 => "structured",
        _ => "prose-lyric",
    };
    Metrics {
        lines: n,
        words: word_count,
        chars: all.iter().map(|l| l.encode_utf16().count()).sum(),
        stanzas,
        class,
    }
}

fn world_time(zone: &str) -> u32 {
    use js_sys::{Array, Date, Intl, Object, Reflect};
    let options = Object::new();
    for (key, value) in [
        ("timeZone", zone),
        ("hour", "2-digit"),
        ("minute", "2-digit"),
        ("hourCycle", "h23"),
    ] {
        let _ = Reflect::set(&options, &JsValue::from_str(key), &JsValue::from_str(value));
    }
    let languages = Array::of1(&JsValue::from_str("en-GB"));
    let format = Intl::DateTimeFormat::new(&languages, &options);
    let text = format
        .format()
        .call1(&JsValue::UNDEFINED, &Date::new_0())
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_default();
    let Some((h, m)) = text.split_once(':') else {
        return 0;
    };
    (h.parse::<u32>().unwrap_or(0) * 60 + m.parse::<u32>().unwrap_or(0)) * 10_000 / 1440
}

fn seed(world: &str, id: &str, m: &Metrics) -> u32 {
    let date = js_sys::Date::new_0();
    let now = js_sys::Date::now();
    let key = format!("tw-layout-time:{world}:{id}");
    let time = crate::navigation::storage()
        .and_then(|s| s.get_item(&key).ok().flatten())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .filter(|v| v["exp"].as_f64().is_some_and(|x| x > now))
        .and_then(|v| v["v"].as_u64())
        .map(|v| v as u32)
        .unwrap_or_else(|| {
            let zone = match world {
                "their-world" => "America/New_York",
                "bless-you" => "Asia/Shanghai",
                _ => "Australia/Brisbane",
            };
            let t = world_time(zone);
            if let Some(store) = crate::navigation::storage() {
                let exp = now + (15.0 + js_sys::Math::random() * 15.0) * 60_000.0;
                let _ = store.set_item(&key, &serde_json::json!({"v":t,"exp":exp}).to_string());
            }
            t
        });
    let input = format!(
        "{world}|{id}|{}-{:02}-{:02}|{}|{}|{}|{}|{time}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        m.lines,
        m.words,
        m.chars,
        m.stanzas
    );
    input
        .encode_utf16()
        .fold(5381u32, |hash, c| hash.wrapping_mul(33) ^ u32::from(c))
}

fn mulberry(mut state: u32) -> impl FnMut() -> f64 {
    move || {
        state = state.wrapping_add(0x6d2b79f5);
        let mut z = (state ^ (state >> 15)).wrapping_mul(state | 1);
        z ^= z.wrapping_add((z ^ (z >> 7)).wrapping_mul(z | 61));
        f64::from(z ^ (z >> 14)) / 4_294_967_296.0
    }
}

fn canvas_context(source: &Element) -> Option<(CanvasRenderingContext2d, f64)> {
    let canvas: HtmlCanvasElement = element("canvas", "").unchecked_into();
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()?
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let computed = window().get_computed_style(source).ok().flatten()?;
    let size = computed.get_property_value("font-size").ok()?;
    let family = computed.get_property_value("font-family").ok()?;
    let font = number(&size, 20.0);
    ctx.set_font(&format!("{font}px {family}"));
    Some((ctx, font))
}

fn measure(context: &CanvasRenderingContext2d, text: &str) -> f64 {
    context.measure_text(text).map(|x| x.width()).unwrap_or(0.0)
}

pub fn compose_quote(quote: &Element, source: &str) {
    let width = quote.get_bounding_client_rect().width() - 1.0;
    let Some((context, _)) = canvas_context(quote).filter(|_| width > 0.0) else {
        return;
    };
    quote.set_text_content(None);
    for (index, line) in source.split('\n').enumerate() {
        if index != 0 {
            let _ = quote.append_child(&document().create_text_node("\n"));
        }
        let row = element("span", "manifold-quote-line");
        let tail = line.split_whitespace().rev().nth(1).map(|word| {
            let offset = word.as_ptr() as usize - line.as_ptr() as usize;
            (offset, &line[offset..])
        });
        if let Some((offset, tail)) = tail.filter(|(offset, tail)| {
            *offset > 0 && measure(&context, line) > width && measure(&context, tail) <= width
        }) {
            row.set_text_content(Some(&line[..offset]));
            let phrase = element("span", "manifold-quote-tail");
            phrase.set_text_content(Some(tail));
            append(&row, &phrase);
        } else {
            row.set_text_content(Some(line));
        }
        append(quote, &row);
    }
}

fn break_cost(word: &str, next: &str) -> f32 {
    let word = word.split_whitespace().next_back().unwrap_or_default();
    let word = word.trim_end_matches(['”', '’', '"', ')']);
    let next = next.split_whitespace().next().unwrap_or_default();
    if word.ends_with(['.', '?', '!', '…']) {
        return 0.0;
    }
    if word.ends_with([',', '，', ';', ':', '—']) {
        return 1.0;
    }
    if "a an the in on at of to by for from with without into and or my your his her their our"
        .split_whitespace()
        .any(|w| word.eq_ignore_ascii_case(w))
    {
        return 7.0;
    }
    if "and but or as while from into before until where with when though if in to on at of for"
        .split_whitespace()
        .any(|w| next.eq_ignore_ascii_case(w))
    {
        return 2.0;
    }
    3.0
}

fn word_boundaries(text: &str) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if let Some(pos) = start.take() {
                words.push((pos, i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(pos) = start {
        words.push((pos, text.len()));
    }
    words
}

fn starts_phrase(word: &str) -> bool {
    let word = word.trim_start_matches(['“', '‘', '"', '\'', '(', '[']);
    ["a", "an", "i", "it"]
        .iter()
        .any(|start| word.eq_ignore_ascii_case(start))
}

fn phrase_boundaries(text: &str, fits: impl Fn(&str) -> bool) -> Vec<(usize, usize)> {
    let mut phrases = Vec::new();
    let mut words = word_boundaries(text).into_iter().peekable();
    while let Some((start, mut end)) = words.next() {
        let mut last = start;
        while let Some(&(next, next_end)) = words.peek() {
            if !starts_phrase(&text[last..end]) || !fits(&text[start..next_end]) {
                break;
            }
            words.next();
            last = next;
            end = next_end;
        }
        phrases.push((start, end));
    }
    phrases
}

fn protect_inline_phrases(paragraph: &Element, fits: &impl Fn(&str) -> bool) {
    let nodes = paragraph.child_nodes();
    let children: Vec<_> = (0..nodes.length()).filter_map(|i| nodes.item(i)).collect();
    for node in children {
        if let Some(child) = node.dyn_ref::<Element>() {
            protect_inline_phrases(child, fits);
            continue;
        }
        if node.node_type() != Node::TEXT_NODE {
            continue;
        }
        let Some(text) = node.node_value() else {
            continue;
        };
        let mut offset = 0;
        for (start, end) in phrase_boundaries(&text, fits) {
            let phrase = &text[start..end];
            if !phrase.contains(char::is_whitespace) {
                continue;
            }
            if start > offset {
                let before = document().create_text_node(&text[offset..start]);
                let _ = paragraph.insert_before(&before, Some(&node));
            }
            let group = element("span", "reading-phrase");
            group.set_text_content(Some(phrase));
            let _ = paragraph.insert_before(&group, Some(&node));
            offset = end;
        }
        if offset > 0 {
            node.set_node_value(Some(&text[offset..]));
        }
    }
}

impl Grid {
    fn from_dom() -> Option<Self> {
        let page = query("#poem-page")?;
        let canvas = query("#grid-canvas")?;
        let title = query("#title-frame")?;
        let title_text = query("#poem-title")?;
        let body = query("#body-frames")?;
        let blocks = elements(&body, ".poem-stanza, .prose-section");
        let prose = attr(&page, "data-kind") == "prose";
        let lines: Vec<Vec<String>> = blocks
            .iter()
            .map(|block| {
                if prose {
                    vec![block.text_content().unwrap_or_default()]
                } else {
                    elements(block, ".poem-line")
                        .into_iter()
                        .filter_map(|e| e.text_content())
                        .collect()
                }
            })
            .collect();
        let mobile = all(".mobile-poem-reader [data-line]")
            .into_iter()
            .map(|el| {
                let text = el.text_content().unwrap_or_default();
                (el, text)
            })
            .collect();
        let m = metrics(&lines);
        let face = match m.class {
            _ if prose => "book",
            "dense-linear" | "prose-lyric" => "book",
            "hymn-structured" | "incantatory" | "minimal-aphoristic" | "minimal-fragment"
            | "still-observation" => "lyric",
            _ => "classic",
        };
        if let Some(root) = query("html") {
            set(&root, "data-reading-type", face);
        }
        let mut rand = mulberry(seed(&world(), &attr(&page, "data-work"), &m));
        let presets: Vec<Preset> =
            serde_json::from_str(include_str!("../../../../site/data/frame-layouts.json")).ok()?;
        let mut pool: Vec<_> = presets
            .into_iter()
            .filter(|p| {
                if prose {
                    p.id == "FL-dense-A"
                } else {
                    p.poem_types.iter().any(|kind| kind == m.class)
                }
            })
            .collect();
        if pool.is_empty() {
            return None;
        }
        let index = (rand() * pool.len() as f64) as usize;
        let preset = pool.swap_remove(index);
        let title_size = if prose || m.lines > 20 || (m.lines > 10 && rand() < 0.5) {
            "var(--type-display)"
        } else {
            "var(--type-hero)"
        };
        let title_end = rand() >= 0.5;
        rand();
        let border = rand() < 0.62;
        let top_rule = rand() < 0.52;
        let accent_extra = rand() < 0.38;
        let accent_first = rand() < 0.5;
        let column_alignment = serde_json::from_str(&attr(&page, "data-column-alignment")).ok();
        Some(Self {
            page,
            canvas,
            title,
            title_text,
            body,
            blocks,
            breaths: cadence(&lines),
            column_alignment,
            lines,
            mobile,
            mobile_width: Cell::new(0.0),
            preset,
            title_size,
            title_end,
            border,
            top_rule,
            accent_first,
            accent_extra,
            prose,
        })
    }

    fn profile(&self, context: &CanvasRenderingContext2d, font: f64) -> Layout {
        let widths: Vec<_> = self
            .lines
            .iter()
            .flatten()
            .map(|line| (measure(context, line) / font) as f32)
            .collect();
        let counts: Vec<_> = self.lines.iter().map(Vec::len).collect();
        let words = self
            .lines
            .iter()
            .flatten()
            .map(|line| line.split_whitespace().count())
            .sum();
        compose(Form {
            line_widths_em: &widths,
            block_line_counts: &counts,
            words,
        })
    }

    fn mobile(&self) {
        let Some(body) = query(".mobile-poem-body") else {
            return;
        };
        let width = body.get_bounding_client_rect().width();
        if width > 0.0 && (self.mobile_width.get() - width).abs() < 0.01 {
            set(&body, "data-layout-viewport", &viewport_key());
            return;
        }
        let blocks = elements(&body, ".mobile-poem-stanza, .prose-section");
        style(&body, "--mobile-verse-size", "");
        let Some((mut context, mut font)) = blocks.first().and_then(canvas_context) else {
            return;
        };
        let wraps = self
            .mobile
            .iter()
            .filter(|(_, text)| measure(&context, text) > width - 1.0)
            .count();
        // Reserve the hanging indent for occasional wraps so dense verse has one clear edge.
        let indent = if wraps >= 4 && wraps * 4 >= self.mobile.len() {
            0.0
        } else {
            0.75
        };
        style(&body, "--mobile-wrap-indent", &format!("{indent}em"));
        if self.mobile.len() >= 12 && wraps * 3 >= self.mobile.len() {
            let compact = (font * 0.94).max(16.8);
            style(&body, "--mobile-verse-size", &px(compact));
            if let Some((new_context, new_font)) = blocks.first().and_then(canvas_context) {
                context = new_context;
                font = new_font;
            }
        }
        let profile = self.profile(&context, font);
        let accents = [
            "var(--highlight-blue-text)",
            "var(--highlight-red-text)",
            "var(--highlight-orange-text)",
            "#426653",
            "#5c682c",
        ];
        let accent = attr(&self.page, "data-work")
            .bytes()
            .fold(0usize, |hash, byte| {
                hash.wrapping_mul(31).wrapping_add(usize::from(byte))
            })
            % accents.len();
        style(&body, "--reading-opening", accents[accent]);
        let leading = if self.prose {
            1.61
        } else {
            f64::from(profile.leading) - 0.09
        };
        style(&body, "--reading-leading", &leading.to_string());
        let block_gap = f64::from(profile.block_gap_em) + if self.prose { 0.34 } else { 0.45 };
        for (block, breath) in blocks.iter().zip(&self.breaths) {
            style(
                block,
                "--reading-breath",
                &format!("{:.3}em", block_gap * breath),
            );
        }
        let mut rows = Vec::with_capacity(self.mobile.len());
        for (line, text) in &self.mobile {
            let _ = line.remove_attribute("data-reading-pause");
            let width = line.get_bounding_client_rect().width() - 1.0;
            if width <= 0.0 {
                rows.push(1);
                continue;
            }
            let words = phrase_boundaries(text, |phrase| {
                measure(&context, phrase) <= width - font * indent
            });
            let count = words.len();
            let mut cuts = Vec::new();
            if count > 1 && measure(&context, text) > width {
                let mut widths = vec![f32::INFINITY; count * count];
                let mut costs = vec![0.0; count];
                for (first, &(start, end)) in words.iter().enumerate() {
                    let from = if first == 0 { 0 } else { start };
                    let room = if first == 0 {
                        width as f32
                    } else {
                        (width as f32 - (font * indent) as f32).max(1.0)
                    };
                    for (last, &(_, to)) in words.iter().enumerate().skip(first) {
                        let measured = measure(&context, &text[from..to]) as f32;
                        widths[first * count + last] = measured;
                        if measured > room {
                            break;
                        }
                    }
                    costs[first] = break_cost(
                        &text[start..end],
                        words
                            .get(first + 1)
                            .map(|&(a, b)| &text[a..b])
                            .unwrap_or(""),
                    );
                }
                cuts = mobile_breaks(&widths, &costs, width as f32, (font * indent) as f32);
            }
            rows.push(cuts.len() + 1);
            set(line, "data-composed", "");
            if cuts.is_empty() {
                line.set_text_content(Some(text));
                continue;
            }
            let mut previous = 0;
            line.set_text_content(None);
            for end in cuts
                .into_iter()
                .map(|i| words[i].0)
                .chain(std::iter::once(text.len()))
            {
                let part = element("span", "line-part");
                part.set_text_content(Some(&text[previous..end]));
                append(line, &part);
                previous = end;
            }
        }
        if !self.prose {
            let mut start = 0;
            let mut marker = 0;
            for stanza in &self.lines {
                let end = start + stanza.len();
                for pause in mobile_pauses(&rows[start..end], stanza) {
                    let line = &self.mobile[start + pause].0;
                    set(line, "data-reading-pause", "");
                    marker += 1;
                    style(
                        line,
                        "--reading-marker",
                        accents[(accent + marker) % accents.len()],
                    );
                }
                start = end;
            }
        }
        self.mobile_width.set(width);
        set(&body, "data-grid-ready", "");
        set(&body, "data-layout-viewport", &viewport_key());
    }

    fn aligned_columns(
        &self,
        rack: &Element,
        boxes: &[Rectangle],
        title_bottom: f64,
        typography: (f64, f64),
        span: f64,
        gap_after: &mut [f64],
    ) -> Option<Vec<ReadingColumn>> {
        let [left, right] = boxes else {
            return None;
        };
        let [left_block, left_line, split] = self.column_alignment?;
        let (leading, gap) = typography;
        let (left_blocks, right_blocks) = self.blocks.split_at(split);

        // Keep the editorial alignment when both halves can still fit on one spread.
        for (leading_scale, gap_scale) in [
            (1.0, 1.0),
            (1.0, 0.88),
            (0.965, 0.82),
            (0.94, 0.75),
            (0.92, 0.70),
        ] {
            style(
                &self.page,
                "--reading-leading",
                &(leading * leading_scale).to_string(),
            );
            let left_heights = block_heights(rack, left.width, left_blocks);
            let right_heights = block_heights(rack, right.width, right_blocks);
            let top = title_bottom + gap * 0.68 * gap_scale;
            let left_offset = left_heights[..left_block].iter().sum::<f64>()
                + gap_after[..left_block].iter().sum::<f64>() * gap_scale
                + line_offset(rack, left.width, &left_blocks[left_block], left_line);
            let right_offset = line_offset(rack, right.width, &right_blocks[0], 0);
            let right_top = top + left_offset - right_offset;
            let gaps: Vec<_> = gap_after.iter().map(|gap| gap * gap_scale).collect();
            let left_end = top + block_run_height(&left_heights, &gaps);
            let right_end = right_top + block_run_height(&right_heights, &gaps[split..]);
            if right_top >= top && left_end.max(right_end) <= span - 24.0 {
                gap_after.copy_from_slice(&gaps);
                return Some(vec![
                    ReadingColumn {
                        frame: Rectangle {
                            top,
                            height: left_end - top,
                            ..*left
                        },
                        blocks: 0..split,
                        spread: 1,
                    },
                    ReadingColumn {
                        frame: Rectangle {
                            top: right_top,
                            height: right_end - right_top,
                            ..*right
                        },
                        blocks: split..self.blocks.len(),
                        spread: 1,
                    },
                ]);
            }
        }

        style(&self.page, "--reading-leading", &leading.to_string());
        None
    }

    fn desktop(&self) {
        let width = self.canvas.get_bounding_client_rect().width();
        if width <= 0.0 || self.blocks.is_empty() {
            return;
        }
        style(&self.page, "--reading-body-size", "");
        let font_sample = query(".poem-page .poem-line, .poem-page .prose-paragraph")
            .unwrap_or_else(|| self.body.clone());
        let Some((context, font)) = canvas_context(&font_sample) else {
            return;
        };
        let profile = self.profile(&context, font);
        let canvas_rect = self.canvas.get_bounding_client_rect();
        let (safe_left, safe_right) = navigation_rails(&canvas_rect);
        let measure = safe_right - safe_left;
        let shortness = if self.prose || self.column_alignment.is_some() {
            0.0
        } else {
            f64::from(profile.shortness) * ((measure - 670.0) / 430.0).clamp(0.0, 1.0)
        };
        let font = if shortness > 0.0 {
            let boosted = font * (1.0 + shortness * 0.19);
            let measure_limit = measure * 0.46 / f64::from(profile.column_em);
            let size = boosted.min(measure_limit).max(font);
            style(&self.page, "--reading-body-size", &px(size));
            size
        } else {
            font
        };
        let leading = if self.prose {
            1.65
        } else {
            f64::from(profile.leading)
        };
        let gap = f64::from(profile.block_gap_em) * font;
        let mut gap_after: Vec<_> = self.breaths.iter().map(|breath| gap * breath).collect();
        let cell = width / 24.0;
        let row = window()
            .get_computed_style(&self.canvas)
            .ok()
            .flatten()
            .and_then(|s| s.get_property_value("--cell-h").ok())
            .map(|s| number(&s, 90.0))
            .unwrap_or(90.0);
        style(&self.page, "--reading-leading", &leading.to_string());
        toggle(&self.page, "is-paginated", false);
        set(&self.page, "data-grid-ready", &self.preset.id);
        toggle(&self.page, "no-top-rule", !self.top_rule);
        let tf = self.preset.title_frame;
        let title_left = ((tf.col_start - 1.0) * cell).max(safe_left);
        let title_width = (tf.col_span * cell).min(safe_right - title_left).max(cell);
        style(&self.title, "left", &px(title_left));
        style(&self.title, "top", &px((tf.row_start - 1.0) * row));
        style(&self.title, "width", &px(title_width));
        style(
            &self.title,
            "text-align",
            if self.title_end { "end" } else { "start" },
        );
        if !self.border {
            for key in ["border-left", "padding-left", "margin-left"] {
                style(&self.title, key, "0");
            }
        }
        style(
            &self.title_text,
            "font-size",
            &format!(
                "min({}, {}px)",
                self.title_size,
                (title_width * 0.37).max(36.0)
            ),
        );
        let title_rect = self.title.get_bounding_client_rect();
        let mut title = Rectangle {
            left: title_rect.left() - canvas_rect.left(),
            top: title_rect.top() - canvas_rect.top(),
            width: title_rect.width(),
            height: title_rect.height(),
        };
        let viewport = window()
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(900.0);
        let nav_bottom = query(".site-nav")
            .map(|nav| nav.get_bounding_client_rect().bottom())
            .unwrap_or(0.0);
        let available_first = || {
            let document_y = self.canvas.get_bounding_client_rect().top()
                + window().scroll_y().unwrap_or_default();
            (viewport - document_y).max(120.0)
        };
        let first_span = available_first();
        let next_span = (viewport - nav_bottom - 24.0).max(120.0);
        let tail = 54.0; // Room for the continuation link beneath the last stanza.

        let rack = element("div", "body-frame grid-measure");
        append(&self.canvas, &rack);
        let full = self.blocks.len();
        let (mut columns, canvas_height) = if self.prose {
            let centered = attr(&self.page, "data-work") == "church";
            let width = if centered {
                (measure * 0.83).min(font * 36.0)
            } else {
                (measure * 0.75).min(font * 48.0)
            };
            let rail = (measure * 0.05).clamp(28.0, 64.0);
            let left = if centered {
                (canvas_rect.width() - width) / 2.0
            } else {
                safe_left + ((measure - width) * 0.58).max(rail + 8.0)
            };
            let title_top = if centered {
                nav_bottom + (viewport * 0.065).clamp(36.0, 64.0)
                    - canvas_rect.top()
                    - window().scroll_y().unwrap_or_default()
            } else {
                (first_span * 0.07).clamp(24.0, 66.0)
            };
            if centered {
                style(&self.title, "border-left", "0");
                style(&self.title, "margin-left", "0");
                style(&self.title, "padding-left", "0");
                for (index, gap) in gap_after
                    .iter_mut()
                    .enumerate()
                    .take(full.saturating_sub(1))
                {
                    *gap = if self.blocks[index].class_list().contains("prose-section") {
                        font * 1.5
                    } else if self.blocks[index + 1]
                        .class_list()
                        .contains("prose-section")
                    {
                        (*gap * 1.3).max(font * 2.9)
                    } else {
                        (*gap * 1.26).max(font * 1.55)
                    };
                }
            }
            style(&self.page, "--prose-rail", &px(rail));
            style(&self.title, "left", &px(left));
            style(&self.title, "top", &px(title_top));
            style(&self.title, "width", &px(width));
            style(
                &self.title,
                "text-align",
                if centered { "center" } else { "start" },
            );
            style(
                &self.title_text,
                "font-size",
                &format!("min({}, {}px)", self.title_size, font * 3.05),
            );
            let top = title_top
                + self.title.get_bounding_client_rect().height()
                + font * if centered { 2.0 } else { 2.6 };
            let heights = block_heights(&rack, width, &self.blocks);
            let height = block_run_height(&heights, &gap_after);
            (
                vec![ReadingColumn {
                    frame: Rectangle {
                        left,
                        top,
                        width,
                        height,
                    },
                    blocks: 0..full,
                    spread: 1,
                }],
                first_span.max(top + height + tail),
            )
        } else {
            let frame = self.preset.body_frames[0];
            let mut left = ((frame.col_start - 1.0) * cell).max(safe_left);
            let body_top = (frame.row_start - 1.0) * row;
            let desired = font * f64::from(profile.column_em);
            let gutter = cell * 0.55;
            if self.column_alignment.is_none()
                && self.preset.body_frames.len() == 1
                && !title_fits(&self.title_text, 2.0)
            {
                // Give a constrained side title the open rail before reducing its type.
                let body_left = safe_left + cell * 0.7;
                let body_width = desired.max(cell * 8.0);
                let title_start = body_left + body_width + gutter * 2.0;
                let title_room = safe_right - title_start;
                if title_room > title_width {
                    left = body_left;
                    style(&self.title, "left", &px(title_start));
                    style(&self.title, "width", &px(title_room));
                    style(&self.title, "top", &px(body_top));
                    style(&self.title, "text-align", "start");
                }
                fit_title(&self.title_text, 2.0);
                let rect = self.title.get_bounding_client_rect();
                title = Rectangle {
                    left: rect.left() - canvas_rect.left(),
                    top: rect.top() - canvas_rect.top(),
                    width: rect.width(),
                    height: rect.height(),
                };
            }
            let beside_title = title.left > left && title.top + title.height > body_top;
            let shift = left + desired + gutter - title.left;
            if beside_title
                && self.preset.body_frames.len() == 1
                && shift > 0.0
                && shift <= cell * 2.0
                && safe_right - title_left - shift >= cell * 4.0
            {
                // Give a nearly fitting verse line the space beside the title.
                style(&self.title, "left", &px(title_left + shift));
                style(
                    &self.title,
                    "width",
                    &px(title_width.min(safe_right - title_left - shift)),
                );
                let moved = self.title.get_bounding_client_rect();
                if moved.height() <= title.height + 0.5 && title_fits(&self.title_text, 2.0) {
                    title.left = moved.left() - canvas_rect.left();
                    title.width = moved.width();
                } else {
                    style(&self.title, "left", &px(title_left));
                    style(&self.title, "width", &px(title_width));
                }
            }
            let right_limit = if beside_title { title.left } else { safe_right };
            let max = (right_limit - left - gutter).max(cell);
            let target = (frame.col_span * cell * 0.88).max(desired).min(max);
            let inset = ((max - target) * 0.18).min(f64::from(profile.inset_em) * 0.17 * font);
            let measured = block_heights(&rack, target, &self.blocks);
            let height = block_run_height(&measured, &gap_after);
            let original = avoid_overlap(
                Rectangle {
                    left: left + inset,
                    top: body_top,
                    width: target,
                    height,
                },
                &[title],
                gap * 0.65,
            );

            let compact_column =
                shortness >= 0.62 && original.width >= desired && height <= first_span * 0.54;
            if self.column_alignment.is_none()
                && (self.preset.body_frames.len() == 1 || compact_column)
                && original.top + height <= first_span - 28.0
                && title.top + title.height <= first_span - 28.0
                && title_fits(&self.title_text, 2.0)
            {
                (
                    vec![ReadingColumn {
                        frame: original,
                        blocks: 0..full,
                        spread: 1,
                    }],
                    first_span,
                )
            } else {
                toggle(&self.page, "is-paginated", true);
                let first_span = available_first();
                let edge = ((safe_right - safe_left) * 0.045).min(2.2 * font);
                let left = safe_left + edge;
                let right = safe_right - edge;
                let gutter = cell.max(1.75 * font);
                let usable = right - left - gutter;
                let first_width = usable * 0.52;
                let right_start = left + first_width + gutter;
                let mut boxes = vec![Rectangle {
                    left,
                    top: 0.0,
                    width: first_width,
                    height: 0.0,
                }];
                if usable * 0.48 >= 13.0 * font {
                    boxes.push(Rectangle {
                        left: right_start,
                        width: usable * 0.48,
                        ..boxes[0]
                    });
                } else {
                    boxes[0].width = right - left;
                }
                for column in &mut boxes {
                    let max_width = column.width;
                    column.width = max_width.min(desired.max(17.0 * font));
                    column.left += (max_width - column.width) * 0.15;
                }

                let title_top = title.top.clamp(12.0, 22.0).min(first_span * 0.12);
                style(&self.title, "top", &px(title_top));
                style(&self.title, "left", &px(boxes[0].left));
                style(
                    &self.title,
                    "width",
                    &px((right - boxes[0].left).min(boxes[0].width * 1.35)),
                );
                style(&self.title, "text-align", "start");
                style(
                    &self.title_text,
                    "font-size",
                    &format!("min({}, {}px)", self.title_size, (font * 2.9).max(36.0)),
                );
                fit_title(&self.title_text, 3.0);
                let title_bottom = title_top + self.title.get_bounding_client_rect().height();
                if let Some(columns) = self.aligned_columns(
                    &rack,
                    &boxes,
                    title_bottom,
                    (leading, gap),
                    first_span,
                    &mut gap_after,
                ) {
                    (columns, first_span)
                } else {
                    let first_top = title_bottom + gap * 0.68;
                    let heights: Vec<_> = boxes
                        .iter()
                        .map(|column| block_heights(&rack, column.width, &self.blocks))
                        .collect();

                    let mut columns = Vec::new();
                    let mut start = 0;
                    let mut page_top = 0.0;
                    let mut spread = 1;
                    let canvas_height;
                    loop {
                        let (span, start_y) = if spread == 1 {
                            (first_span, first_top)
                        } else {
                            (next_span, 20.0)
                        };
                        let capacities = vec![span - start_y - 24.0; boxes.len()];
                        // A short ending follows the previous right column across the page break.
                        let ranges = if spread > 1
                            && boxes.len() == 2
                            && block_run_height(&heights[1][start..], &gap_after[start..])
                                <= capacities[1] * 0.6
                        {
                            vec![start..start, start..full]
                        } else {
                            let final_ranges =
                                fit_spread(&heights, &capacities, start, &gap_after, 0.0);
                            if final_ranges.last().is_some_and(|range| range.end == full) {
                                final_ranges
                            } else {
                                fit_spread(&heights, &capacities, start, &gap_after, tail - 24.0)
                            }
                        };
                        let mut used_span = span;
                        if ranges.iter().all(Range::is_empty) {
                            // A long stanza keeps one wider, uninterrupted measure.
                            let width = (right - left).min(desired.max(font * 32.0));
                            let tall = block_heights(&rack, width, &self.blocks);
                            let end = fit_spread(
                                std::slice::from_ref(&tall),
                                &[span - start_y - tail],
                                start,
                                &gap_after,
                                0.0,
                            )[0]
                            .end
                            .max(start + 1);
                            let height = block_run_height(&tall[start..end], &gap_after[start..]);
                            columns.push(ReadingColumn {
                                frame: Rectangle {
                                    left,
                                    top: page_top + start_y,
                                    width,
                                    height,
                                },
                                blocks: start..end,
                                spread,
                            });
                            used_span = span.max(start_y + height + tail);
                            start = end;
                        } else {
                            for ((range, frame), measured) in
                                ranges.into_iter().zip(&boxes).zip(&heights)
                            {
                                if range.is_empty() {
                                    continue;
                                }
                                start = range.end;
                                let height = block_run_height(
                                    &measured[range.clone()],
                                    &gap_after[range.start..],
                                );
                                columns.push(ReadingColumn {
                                    frame: Rectangle {
                                        top: page_top + start_y,
                                        height,
                                        ..*frame
                                    },
                                    blocks: range,
                                    spread,
                                });
                            }
                        }
                        if start == full {
                            // Let the final continuation land in the viewport without reserving a blank screen.
                            canvas_height = if spread == 1 {
                                first_span
                            } else {
                                page_top + start_y + viewport * 0.46
                            };
                            break;
                        }
                        page_top += used_span;
                        spread += 1;
                    }
                    (columns, canvas_height)
                }
            }
        };
        if !self.prose {
            let continues = columns.last().is_some_and(|column| column.spread > 1);
            let title_rect = self.title.get_bounding_client_rect();
            let title_top = title_rect.top() - self.canvas.get_bounding_client_rect().top();
            let title_bottom = title_top + title_rect.height();
            let bottom = columns
                .iter()
                .take_while(|column| column.spread == 1)
                .map(|column| column.frame.top + column.frame.height)
                .fold(title_bottom, f64::max);
            let span = available_first();
            let slack = (span - bottom - tail).max(0.0);
            let title_gap = columns[0].frame.top - title_bottom;
            let extra_gap = if title_gap >= 0.0 {
                (font * 2.45 - title_gap).clamp(0.0, slack * 0.38)
            } else {
                0.0
            };
            let offset = if continues {
                ((slack - extra_gap) * 0.40).min(font * 3.4)
            } else {
                // Use measured spread height even when the two columns have an
                // editorial alignment. Moving both together preserves that baseline.
                let unfilled = ((slack / font - 3.0) / 8.0).clamp(0.0, 1.0);
                let balance = ((slack - extra_gap) * 0.44 * unfilled).min(span * 0.14);
                balance.max((span * shortness * 0.30).min((slack - extra_gap) * 0.54))
            };
            if offset > 0.0 {
                style(&self.title, "top", &px(title_top + offset));
            }
            for column in columns.iter_mut().take_while(|column| column.spread == 1) {
                column.frame.top += offset + extra_gap;
            }
        }
        rack.remove();

        self.body.set_text_content(None);
        let mut accent_seen = 0;
        let title_box = self.title.get_bounding_client_rect();
        let canvas_box = self.canvas.get_bounding_client_rect();
        let mut occupied = vec![Rectangle {
            left: title_box.left() - canvas_box.left(),
            top: title_box.top() - canvas_box.top(),
            width: title_box.width(),
            height: title_box.height(),
        }];
        for (i, column) in columns.iter().enumerate() {
            let ReadingColumn {
                frame,
                blocks,
                spread,
            } = column;
            let body = element("div", "body-frame");
            set(&body, "id", &format!("reading-column-{}", i + 1));
            set(&body, "data-reading-spread", &spread.to_string());
            set(&body, "tabindex", "-1");
            style(&body, "width", &px(frame.width));
            style(&body, "left", &px(frame.left));
            style(&body, "top", &px(frame.top));
            append(&self.body, &body);
            for index in blocks.clone() {
                let stanza = copy(&self.blocks[index]);
                if index + 1 < blocks.end {
                    toggle(&stanza, "reading-break", true);
                    style(&stanza, "--reading-breath", &px(gap_after[index]));
                }
                set(&stanza, "aria-hidden", "true");
                for line in elements(&stanza, ".poem-line") {
                    if (accent_seen == 0 && self.accent_first)
                        || (accent_seen == 1 && self.accent_extra)
                    {
                        toggle(&line, "poem-line--accent", true);
                        toggle(&line, "poem-line--accent-soft", accent_seen == 1);
                    }
                    accent_seen += 1;
                }
                append(&body, &stanza);
            }
            occupied.push(Rectangle {
                height: body.get_bounding_client_rect().height(),
                ..*frame
            });
            if let Some(next) = columns.get(i + 1).filter(|next| next.spread != *spread) {
                let link = element("a", "reading-continue");
                set(&link, "href", &format!("#reading-column-{}", i + 2));
                let label = format!("Continue to page {}", next.spread);
                set(&link, "aria-label", &label);
                set(&link, "title", &label);
                link.set_text_content(Some(&format!("{:02} ↓", next.spread)));
                if next.frame.left + next.frame.width / 2.0 <= width / 2.0 {
                    let spread_bottom = columns[..=i]
                        .iter()
                        .rev()
                        .take_while(|column| column.spread == *spread)
                        .map(|column| column.frame.top + column.frame.height)
                        .fold(
                            frame.top + body.get_bounding_client_rect().height(),
                            f64::max,
                        );
                    style(
                        &link,
                        "inset-inline-start",
                        &px(next.frame.left - frame.left),
                    );
                    style(&link, "inset-inline-end", "auto");
                    style(
                        &link,
                        "inset-block-start",
                        &px(spread_bottom - frame.top + 2.0),
                    );
                }
                append(&body, &link);
            }
        }
        let end = |areas: &[Rectangle]| {
            areas
                .iter()
                .map(|area| area.top + area.height)
                .fold(0.0, f64::max)
        };
        let mut height = canvas_height.max(end(&occupied) + 24.0);
        if let Some(meta) = query("#poem-meta") {
            style(&meta, "width", "max-content");
            let size = meta.get_bounding_client_rect();
            let side_end = |right: bool| {
                occupied
                    .iter()
                    .filter(|area| (area.left + area.width / 2.0 > width / 2.0) == right)
                    .map(|area| area.top + area.height)
                    .fold(0.0, f64::max)
            };
            let index_right = !self.prose && side_end(true) + 8.0 < side_end(false);
            let index = query("#index-trigger").map(|index| {
                toggle(&index, "index-trigger--right", index_right);
                index.get_bounding_client_rect()
            });
            let mut left = if self.prose || index_right {
                (safe_right - size.width()).max(width / 2.0)
            } else {
                (safe_left + cell).min(width / 2.0 - size.width())
            };
            if let Some(index) = index.filter(|index| index.width() > 0.0) {
                if index_right {
                    left = left.min(index.left() - canvas_box.left() - size.width() - 12.0);
                } else if !self.prose {
                    left = left.max(index.right() - canvas_box.left() + 12.0);
                }
            }
            let clear = occupied
                .iter()
                .filter(|area| area.left < left + size.width() && area.left + area.width > left)
                .map(|area| area.top + area.height + 12.0)
                .fold(0.0, f64::max);
            let edge_clearance = 26.0;
            let top = (height - size.height() - edge_clearance).max(clear);
            style(&meta, "left", &px(left.max(0.0)));
            style(&meta, "top", &px(top));
            height = height.max(top + size.height() + edge_clearance);
        }
        style(&self.canvas, "height", &px(height));
        set(&self.page, "data-layout-viewport", &viewport_key());
    }

    fn layout(&self) {
        if crate::mobile() {
            self.mobile();
        } else {
            self.desktop();
        }
    }
}

fn viewport_key() -> String {
    let width = window()
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or_default();
    let height = window()
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or_default();
    format!("{width}x{height}")
}

pub fn init() {
    for paragraph in
        all(".mobile-poem-body .prose-paragraph, .about-group[aria-label='Biography'] > p")
    {
        if let Some((context, _)) = canvas_context(&paragraph) {
            // At the narrowest reader width, leave room for both 22 px side margins.
            let width = paragraph.get_bounding_client_rect().width();
            let width = if width > 0.0 { width.min(274.0) } else { 274.0 };
            protect_inline_phrases(&paragraph, &|phrase| measure(&context, phrase) <= width);
        }
    }
    let Some(grid) = Grid::from_dom().map(Rc::new) else {
        return;
    };
    grid.layout();
    let pending = Rc::new(Cell::new(false));
    let callback = Rc::new(RefCell::new(None));
    let paint = Closure::<dyn FnMut()>::new({
        let pending = pending.clone();
        let grid = grid.clone();
        move || {
            pending.set(false);
            grid.layout();
        }
    });
    *callback.borrow_mut() = Some(paint);
    on(window().as_ref(), "resize", move |_| {
        if pending.replace(true) {
            return;
        }
        if let Some(cb) = callback.borrow().as_ref() {
            let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
    });
}
