//! Short local studies. Artwork is composed before the shared character encoder.
#[path = "cottage_study.rs"]
mod cottage;
#[path = "train_study.rs"]
mod train;
#[path = "umbrella_study.rs"]
mod umbrella;

use super::plate_pixels::plate_pixel;
use super::{BORING_CLOUDS, COTTAGE, Color, Material, Sample, UMBRELLA};

const TRAIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/boring-clouds-distance.rgb"));
const STREET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/umbrella-cafe.rgb"));
type Point = [f32; 2];

enum Scene {
    Train(train::Train),
    Umbrella(umbrella::Umbrella),
    Cottage(cottage::Cottage),
}

pub(super) struct Artwork {
    scene: Scene,
    cols: usize,
    rows: usize,
    background: Vec<Color>,
}

impl Artwork {
    pub(super) fn new(kind: u8, cols: usize, rows: usize) -> Self {
        let (plate, scene) = match kind {
            BORING_CLOUDS => (TRAIN, Scene::Train(train::Train::at(0.))),
            UMBRELLA => (STREET, Scene::Umbrella(umbrella::Umbrella::at(0.))),
            COTTAGE => (cottage::PLATE, Scene::Cottage(cottage::Cottage::at(0.))),
            _ => unreachable!("unknown preview kind"),
        };
        let background = (0..cols * rows)
            .map(|i| {
                let mut rgb = [0.; 3];
                for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    let x = (i % cols) as f32 + ox;
                    let y = (i / cols) as f32 + oy;
                    let pixel = plate_pixel(plate, x / cols as f32, y / rows as f32);
                    for c in 0..3 {
                        rgb[c] += pixel[c] * 0.25;
                    }
                }
                rgb.map(|channel| channel.powf(0.87))
            })
            .collect();
        Self {
            scene,
            cols,
            rows,
            background,
        }
    }

    pub(super) fn update(&mut self, seconds: f32) {
        match &mut self.scene {
            Scene::Train(frame) => *frame = train::Train::at(seconds),
            Scene::Umbrella(frame) => *frame = umbrella::Umbrella::at(seconds),
            Scene::Cottage(frame) => *frame = cottage::Cottage::at(seconds),
        }
    }

    pub(super) fn sample(&self, x: usize, y: usize) -> Sample {
        let base = self.background[y * self.cols + x];
        let mut color = [0.; 3];
        for (ox, oy) in [(0.28, 0.30), (0.72, 0.30), (0.28, 0.70), (0.72, 0.70)] {
            let u = (x as f32 + ox) / self.cols as f32;
            let v = (y as f32 + oy) / self.rows as f32;
            let shade = match &self.scene {
                Scene::Train(frame) => frame.shade(base, u, v),
                Scene::Umbrella(frame) => frame.shade(base, u, v),
                Scene::Cottage(frame) => frame.shade(base, u, v),
            };
            for c in 0..3 {
                color[c] += shade[c] * 0.25;
            }
        }
        let mut sample = Sample::new(color, Material::Solid);
        sample.floor = 0.033;
        sample
    }
}
