use anyhow::{Result, anyhow, ensure};
use std::fmt::Write;
use typst::{
    Library, LibraryExt, World,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_layout::PagedDocument;
use typst_render::RenderOptions;

const WIDTH: f32 = 350.0;
const INSET_X: f32 = (WIDTH - 240.0) / 2.0;
const HEIGHT: f32 = 150.0;

struct ArtWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    source: Source,
}

impl World for ArtWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.source.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        (id == self.main())
            .then(|| self.source.clone())
            .ok_or(FileError::AccessDenied)
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.source(id)
            .map(|source| Bytes::from_string(source.text().to_owned()))
    }
    fn font(&self, _: usize) -> Option<Font> {
        None
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

struct Plate(String);
type Point = (f32, f32);

impl Plate {
    fn new() -> Self {
        Self(format!(
            "#set page(width: {WIDTH}pt, height: {HEIGHT}pt, margin: 0pt, fill: black)\n"
        ))
    }

    fn polygon(&mut self, points: &[Point], ink: u8) {
        let x = points.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let y = points.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let _ = write!(
            self.0,
            "#place(top + left, dx: {:.2}pt, dy: {y:.2}pt, polygon(",
            x + INSET_X
        );
        for &(px, py) in points {
            let _ = write!(self.0, "({:.2}pt, {:.2}pt), ", px - x, py - y);
        }
        let _ = writeln!(self.0, "fill: luma({ink}), stroke: none))");
    }

    fn ellipse(&mut self, (x, y): Point, rx: f32, ry: f32, ink: u8) {
        let _ = writeln!(
            self.0,
            "#place(top + left, dx: {:.2}pt, dy: {:.2}pt, ellipse(width: {:.2}pt, height: {:.2}pt, fill: luma({ink}), stroke: none))",
            x + INSET_X - rx,
            y - ry,
            rx * 2.0,
            ry * 2.0
        );
    }

    fn ring(&mut self, center: Point, rx: f32, ry: f32, width: f32, ink: u8) {
        self.ellipse(center, rx, ry, ink);
        self.ellipse(center, rx - width, ry - width, 0);
    }

    fn stroke(&mut self, a: Point, b: Point, width: f32, ink: u8) {
        let length = (b.0 - a.0).hypot(b.1 - a.1);
        if length < 0.01 {
            return;
        }
        let dx = (a.1 - b.1) * width / length / 2.0;
        let dy = (b.0 - a.0) * width / length / 2.0;
        self.polygon(
            &[
                (a.0 + dx, a.1 + dy),
                (b.0 + dx, b.1 + dy),
                (b.0 - dx, b.1 - dy),
                (a.0 - dx, a.1 - dy),
            ],
            ink,
        );
    }

    fn arc(&mut self, center: Point, radii: Point, from: f32, to: f32, width: f32, ink: u8) {
        let mut previous = None;
        for i in 0..=30 {
            let a = (from + (to - from) * i as f32 / 30.0).to_radians();
            let point = (center.0 + radii.0 * a.cos(), center.1 + radii.1 * a.sin());
            if let Some(last) = previous {
                self.stroke(last, point, width, ink);
            }
            previous = Some(point);
        }
    }
}

fn rand(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f32 / 16_777_216.0
}

fn plate(motif: usize, seed: u32) -> String {
    let mut p = Plate::new();
    let mut rng = seed;
    let balance = 108.0 + (rand(&mut rng) - 0.5) * 23.0 + 8.0;
    match motif {
        0 => {
            for (radius, ink) in [
                (29.0, 24),
                (23.0, 45),
                (17.0, 105),
                (14.0, 184),
                (12.0, 239),
            ] {
                p.ellipse((balance - 21.0, 53.0), radius, radius, ink);
            }
            p.stroke((21.0, 83.0), (222.0, 83.0), 1.4, 89);
            for i in 0..77 {
                let y = 88.0 + rand(&mut rng) * 49.0;
                let spread = (y - 79.0) * 1.4;
                let x = balance - 21.0 + (rand(&mut rng) - 0.5) * spread * 2.0;
                let end = (x + 3.0 + rand(&mut rng) * spread * 0.57).min(228.0);
                p.stroke(
                    (x, y),
                    (end, y),
                    0.55 + rand(&mut rng) * 1.8,
                    if i % 5 == 0 { 207 } else { 71 + (i % 64) as u8 },
                );
            }
        }
        1 => {
            let center = (balance, 70.0);
            p.ring(center, 74.0, 38.0, 1.5, 98);
            p.arc(center, (67.0, 33.0), 188.0, 346.0, 2.0, 142);
            p.arc(center, (83.0, 45.0), 12.0, 164.0, 1.7, 91);
            p.ring((balance + 5.0, 70.0), 30.0, 26.0, 1.8, 163);
            p.ring((balance + 5.0, 70.0), 22.0, 19.0, 1.3, 93);
            p.ellipse((balance + 5.0, 70.0), 12.0, 11.0, 48);
            p.ellipse((balance + 1.0, 67.0), 6.0, 5.0, 226);
            p.ellipse((balance + 14.0, 78.0), 2.4, 2.4, 139);
            for i in 0..32 {
                let angle = i as f32 * 0.19635;
                let x = balance + angle.cos() * 48.0;
                let y = 70.0 + angle.sin() * 31.0;
                p.ellipse((x, y), 1.0, 0.8, 42 + (i % 6) * 12);
            }
        }
        2 | 3 => {
            let center = balance + 7.0;
            for (rx, ry, ink) in [(69.0, 57.0, 47), (54.0, 46.0, 65), (39.0, 34.0, 83)] {
                if motif == 2 {
                    p.arc((center, 83.0), (rx, ry), 180.0, 360.0, 1.8, ink);
                } else {
                    p.stroke((center - rx, 82.0 - ry), (center + rx, 82.0 - ry), 1.7, ink);
                }
                for x in [center - rx, center + rx] {
                    p.stroke(
                        (x, if motif == 2 { 83.0 } else { 82.0 - ry }),
                        (x, 121.0),
                        1.4,
                        ink,
                    );
                }
            }
            for i in 0..14 {
                let offset = i as f32 * 2.15;
                let ink = 41 + i * 10;
                p.polygon(
                    &[
                        (center + 12.0 + offset * 0.24, 33.0),
                        (center + 16.0 + offset * 0.24, 33.0),
                        (center - 30.0 + offset, 123.0),
                        (center - 35.0 + offset, 123.0),
                    ],
                    ink,
                );
            }
            p.stroke((center - 43.0, 123.0), (center + 7.0, 123.0), 1.2, 157);
            for x in [21.0, 44.0, 66.0, 90.0, 145.0, 170.0, 194.0, 219.0] {
                p.stroke((center + (x - center) * 0.5, 116.0), (x, 143.0), 1.05, 65);
            }
        }
        4 => {
            p.ellipse((balance - 11.0, 49.0), 9.0, 9.0, 134);
            p.stroke((18.0, 77.0), (223.0, 77.0), 1.4, 98);
            for x in [32.0, 74.0, 169.0, 211.0] {
                p.stroke((balance + (x - balance) * 0.11, 77.0), (x, 141.0), 1.6, 160);
            }
            for i in 1..18 {
                let d = (i as f32 / 18.0).powf(2.0);
                p.stroke(
                    (balance - 7.0 - 64.0 * d, 78.0 + d * 63.0),
                    (balance + 9.0 + 64.0 * d, 78.0 + d * 63.0),
                    0.8 + d,
                    91,
                );
            }
        }
        5 => {
            for _ in 0..130 {
                let x = 21.0 + rand(&mut rng) * 195.0;
                let y = 18.0 + rand(&mut rng) * 108.0;
                let d = 3.0 + rand(&mut rng) * 13.0;
                p.stroke(
                    (x, y),
                    (x - d * 0.4, y + d),
                    0.9,
                    32 + (rand(&mut rng) * 131.0) as u8,
                );
            }
            p.stroke((13.0, 104.0), (226.0, 104.0), 1.6, 130);
        }
        6 => {
            p.stroke((16.0, 61.0), (224.0, 61.0), 1.4, 137);
            for i in 0..30 {
                let x = 18.0 + rand(&mut rng) * 206.0;
                let mid = (balance * 0.55 + x * 0.45, 75.0 + rand(&mut rng) * 28.0);
                p.stroke((balance + (x - balance) * 0.16, 61.0), mid, 1.2, 125);
                p.stroke(mid, (x, 111.0 + rand(&mut rng) * 26.0), 0.75, 70 + i * 4);
            }
        }
        _ => {
            p.polygon(
                &[
                    (balance - 6.0, 45.0),
                    (balance + 17.0, 45.0),
                    (balance + 32.0, 65.0),
                    (balance - 21.0, 65.0),
                ],
                196,
            );
            p.stroke((balance + 6.0, 65.0), (balance + 6.0, 109.0), 2.3, 153);
            p.polygon(
                &[
                    (balance - 20.0, 67.0),
                    (balance + 32.0, 67.0),
                    (balance + 57.0, 109.0),
                    (balance - 51.0, 109.0),
                ],
                43,
            );
            p.stroke((28.0, 111.0), (213.0, 111.0), 2.0, 144);
            p.stroke((56.0, 111.0), (49.0, 140.0), 1.6, 93);
            p.stroke((190.0, 111.0), (199.0, 140.0), 1.6, 93);
        }
    }
    for _ in 0..200 {
        let x = 10.0 + rand(&mut rng) * 220.0;
        let y = 10.0 + rand(&mut rng) * 128.0;
        let radius = 0.2 + rand(&mut rng) * 0.58;
        p.ellipse((x, y), radius, radius, 40 + (rand(&mut rng) * 54.0) as u8);
    }
    p.0
}

pub fn build(motif: usize, output: &std::path::Path) -> Result<()> {
    let seed = 2_166_136_261_u32.wrapping_add(motif as u32 * 167_776);
    let src = plate(motif, seed);
    let id = RootedPath::new(VirtualRoot::Project, VirtualPath::new("plate.typ")?).intern();
    let world = ArtWorld {
        library: LazyHash::new(Library::builder().build()),
        book: LazyHash::new(FontBook::new()),
        source: Source::new(id, src),
    };
    let doc = typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|errors| {
            anyhow!(
                "Typst plate: {}",
                errors
                    .iter()
                    .map(|e| e.message.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    ensure!(doc.pages().len() == 1, "expected a one-page Typst plate");
    let image = typst_render::render(&doc.pages()[0], &RenderOptions::default());
    image.save_png(output)?;
    Ok(())
}
