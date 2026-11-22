//! The halftone is a character field, not a source bitmap under a dot overlay.
//! Coverage/ink compensation and Bayer thresholds adapted from ascii.rest.
//! Copyright (c) 2026 bas3line; see ../ascii.rest-LICENSE.txt.
use crate::scene::{Color, NEUTRAL_INKS, Sample, hash, luminance, tint};

pub const CHARS: [char; 4] = [' ', '·', '•', '●'];
const COVER: [f32; 4] = [0.0, 0.30, 0.60, 1.0];
const BAYER: [f32; 16] = [
    0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.,
];
const HUES: usize = 20;
const SATURATIONS: [f32; 4] = [0.18, 0.42, 0.69, 1.];
const VALUES: [f32; 3] = [0.60, 0.80, 1.];

pub fn illustration_palette() -> Vec<Color> {
    let mut palette: Vec<_> = NEUTRAL_INKS.into_iter().map(tint).collect();
    for i in 0..HUES {
        let hue = i as f32 / HUES as f32;
        for saturation in SATURATIONS {
            for value in VALUES {
                palette.push(std::array::from_fn(|channel| {
                    let phase = (hue * 6. + [0., 4., 2.][channel]).rem_euclid(6.);
                    let ramp = (phase.min(6. - phase) - 1.).clamp(0., 1.);
                    value * (1. - saturation * ramp)
                }));
            }
        }
    }
    palette
}

fn tone_slot(values: &[f32], value: f32, threshold: f32) -> usize {
    values
        .windows(2)
        .take_while(|pair| value >= pair[0] + (pair[1] - pair[0]) * threshold)
        .count()
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Cell {
    pub glyph: u8,
    pub ink: u8,
}

pub struct Encoder {
    palette: Vec<Color>,
    lookup: Vec<u8>,
    columns: usize,
    tone: Vec<f32>,
    dither_strength: f64,
    illustration: bool,
}
impl Encoder {
    pub fn new(palette: &[Color], columns: usize) -> Self {
        Self {
            palette: palette.to_vec(),
            lookup: vec![255; 32768],
            columns,
            dither_strength: 1.,
            illustration: false,
            tone: (0..=1024)
                .map(|i| {
                    let v = i as f64 / 256.;
                    (if v < 0.75 {
                        v
                    } else {
                        0.75 + 0.25 * (1. - (-(v - 0.75) * 4.).exp())
                    }) as f32
                })
                .collect(),
        }
    }
    pub fn with_dither_strength(mut self, strength: f64) -> Self {
        self.dither_strength = strength;
        self
    }
    pub fn with_illustration_inks(mut self) -> Self {
        self.illustration = true;
        self
    }
    fn nearest(&mut self, color: Color) -> u8 {
        let peak = color.into_iter().fold(0.0_f32, f32::max);
        let low = color.into_iter().fold(1.0_f32, f32::min);
        // A neutral surface must stay neutral even beside a scene's blue/green
        // inks. Do this before the RGB-cube cache: a cube can straddle the
        // neutral threshold, so a coloured entry cannot contaminate this path.
        if peak - low < 0.025 {
            let value = luminance(color);
            return self
                .palette
                .iter()
                .enumerate()
                .filter(|(_, c)| (c[0] - c[1]).abs() < 0.0001 && (c[1] - c[2]).abs() < 0.0001)
                .min_by(|(_, a), (_, b)| (a[0] - value).abs().total_cmp(&(b[0] - value).abs()))
                .map(|(i, _)| i as u8)
                .unwrap_or(0);
        }
        let q = color.map(|c| (c.clamp(0., 1.) * 31.99) as usize);
        let key = (q[0] << 10) | (q[1] << 5) | q[2];
        if self.lookup[key] != 255 {
            return self.lookup[key];
        }
        let result = self
            .palette
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let distance = |p: &Color| {
                    (p[0] - color[0]).powi(2) * 0.3
                        + (p[1] - color[1]).powi(2) * 0.5
                        + (p[2] - color[2]).powi(2) * 0.2
                };
                distance(a).total_cmp(&distance(b))
            })
            .unwrap()
            .0 as u8;
        self.lookup[key] = result;
        result
    }
    fn transfer(&self, sample: Sample, x: usize, y: usize) -> (u8, Color) {
        // Complete Kyoto Dusk transfer, in its original order. No independent
        // global lift, forced-white threshold or partial ink compensation.
        let color = sample
            .color
            .map(|v| self.tone[((v.max(0.) as f64 * 256.) as usize).min(1024)] as f64);
        let peak = color.into_iter().fold(0.0001_f64, f64::max);
        let floor = sample.floor as f64;
        let level =
            (floor + (1. - floor) * peak.powf(1.1) * 1.02).clamp(0., 1.) * sample.opacity as f64;
        let k = y * self.columns + x;
        let n = (k as u32)
            .wrapping_mul(374761393)
            .wrapping_add(77_u32.wrapping_mul(668265263));
        let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        let h = (n ^ (n >> 16)) as f64 / 4294967296.;
        let ordered = BAYER[(y & 3) * 4 + (x & 3)] as f64 / 16. - 0.47;
        // Upstream stores the combined threshold in a Float32Array.
        let dither = (ordered * 0.4 + (h - 0.5) * 0.5) as f32;
        let step = (level * 3. + dither as f64 * self.dither_strength)
            .round()
            .clamp(0., 3.) as usize;
        if step == 0 {
            return (0, [0.; 3]);
        }
        let want = ((level + 0.06) / COVER[step] as f64).min(1.);
        let scale = (0.3 + 0.7 * want) / peak;
        (step as u8, color.map(|v| (v * scale).clamp(0., 1.) as f32))
    }
    pub fn encode(&mut self, sample: Sample, x: usize, y: usize) -> Cell {
        if self.illustration {
            return self.illustration_cell(sample, x, y);
        }
        let (glyph, color) = self.transfer(sample, x, y);
        if glyph == 0 {
            return Cell::default();
        }
        Cell {
            glyph,
            ink: self.nearest(color),
        }
    }

    fn illustration_cell(&mut self, sample: Sample, x: usize, y: usize) -> Cell {
        let [r, g, b] = sample.color;
        let peak = r.max(g).max(b);
        let low = r.min(g).min(b);
        let chroma = peak - low;
        let level = peak.clamp(0., 1.).powf(0.78) * sample.opacity;
        let threshold = BAYER[(y & 3) * 4 + (x & 3)] * 0.0625;
        let glyph = (level * 3. + 0.35 * (threshold - 0.47)).round() as usize;
        if glyph == 0 {
            return Cell::default();
        }
        let brightness = (0.62 + 0.38 * (level / COVER[glyph] as f32)).min(1.);
        let saturation = chroma / peak;
        let threshold = |seed| 0.20 + 0.60 * hash(x as i32 + seed, y as i32 - seed);
        if saturation < SATURATIONS[0] * threshold(73) {
            return Cell {
                glyph: glyph as u8,
                ink: self.nearest([brightness; 3]),
            };
        }
        let hue = if r >= g.max(b) {
            (g - b) / chroma
        } else if g >= b {
            2. + (b - r) / chroma
        } else {
            4. + (r - g) / chroma
        };
        let sector = hue.rem_euclid(6.) * HUES as f32 / 6.;
        // Screen-anchored thresholds mix neighboring inks spatially. Seeking
        // to the same frame always selects the same ink, regardless of history.
        let hue_index = ((sector + 1. - threshold(13)).floor() as usize) % HUES;
        let sat = tone_slot(&SATURATIONS, saturation, threshold(29));
        let val = tone_slot(&VALUES, brightness, threshold(53));
        let ink = NEUTRAL_INKS.len() + (hue_index * SATURATIONS.len() + sat) * VALUES.len() + val;
        Cell {
            glyph: glyph as u8,
            ink: ink as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    include!("reference_tone_cases.rs");

    #[test]
    fn illustrated_inks_keep_primary_colors_and_seek_stably() {
        let palette = illustration_palette();
        assert!(palette.len() < 255);
        let mut encoder = Encoder::new(&palette, 136).with_illustration_inks();
        let ink = |encoder: &mut Encoder, color| {
            let cell = encoder.encode(Sample::new(color, crate::scene::Material::Solid), 29, 41);
            assert!(cell.glyph >= 2);
            palette[cell.ink as usize]
        };
        let red = ink(&mut encoder, [0.82, 0.24, 0.16]);
        let blue = ink(&mut encoder, [0.22, 0.44, 0.84]);
        let neutral = ink(&mut encoder, [0.47; 3]);
        assert!(red[0] - red[1] > 0.37 && red[0] - red[2] > 0.47);
        assert!(blue[2] - blue[0] > 0.37 && blue[2] - blue[1] > 0.26);
        assert_eq!(neutral, [neutral[0]; 3]);
        assert_eq!(ink(&mut encoder, [0.82, 0.24, 0.16]), red);
    }

    #[test]
    fn matches_upstream_kyoto_reference_glyphs_and_compensated_ink() {
        let encoder = Encoder::new(&[[0.; 3], [1.; 3]], 200);
        for case in REFERENCE_CASES {
            for y in 0..4 {
                for x in 0..4 {
                    let mut sample = Sample::new(case.rgb, crate::scene::Material::Solid);
                    sample.floor = case.floor;
                    sample.opacity = case.fade;
                    let (glyph, rgb) = encoder.transfer(sample, x, y);
                    assert_eq!(glyph, case.glyphs[y * 4 + x], "{} at {x},{y}", case.name);
                    if glyph != 0 {
                        for (actual, expected) in rgb.into_iter().zip(case.inks[glyph as usize]) {
                            assert!(
                                (actual - expected).abs() < 0.00001,
                                "{} changes upstream ink compensation",
                                case.name
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn gradients_use_empty_and_three_different_dot_glyphs() {
        let palette = [[0., 0., 0.], [0.3, 0.5, 0.8], [1., 0.9, 0.6]];
        let mut encoder = Encoder::new(&palette, 128);
        let mut seen = [0; 4];
        for y in 0..32 {
            for x in 0..128 {
                let v = x as f32 / 127.;
                seen[encoder
                    .encode(Sample::new([v, v, v], crate::scene::Material::Air), x, y)
                    .glyph as usize] += 1;
            }
        }
        for count in &seen[..4] {
            assert!(*count > 100, "missing coverage tier {seen:?}");
        }
    }
    #[test]
    fn neutral_inputs_remain_neutral_in_every_scene_palette() {
        for kind in [0, 1, 3, 4, 5] {
            let scene = crate::scene::Scene::new(kind, 102, 60);
            let mut encoder = Encoder::new(&scene.palette, 102);
            let mut inks = std::collections::BTreeSet::new();
            for x in 0..128 {
                let value = x as f32 / 127.;
                for y in 0..4 {
                    let cell = encoder.encode(
                        Sample::new([value; 3], crate::scene::Material::Solid),
                        x,
                        y,
                    );
                    let ink = scene.palette[cell.ink as usize];
                    assert_eq!(ink[0], ink[1], "scene {kind} tints a grey input");
                    assert_eq!(ink[1], ink[2], "scene {kind} tints a grey input");
                    inks.insert(cell.ink);
                }
            }
            assert!(inks.len() >= 6, "scene {kind} collapses the grey range");
        }
    }
}
